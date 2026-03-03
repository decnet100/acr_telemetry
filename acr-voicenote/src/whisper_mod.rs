//! Candle-Whisper Integration (angelehnt an candle-examples/whisper-microphone)

pub mod multilingual;

use anyhow::{Error as E, Result};
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::ops::softmax;
use candle_transformers::models::whisper::{self as m, audio, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};
use rand::{distr::Distribution, distr::weighted::WeightedIndex, SeedableRng};
use tokenizers::Tokenizer;

#[allow(unused_imports)]
pub use multilingual::detect_language;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhichModel {
    Tiny,
    TinyEn,
    Base,
    BaseEn,
    Small,
    SmallEn,
}

impl WhichModel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tiny" => Some(Self::Tiny),
            "tiny.en" | "tiny_en" => Some(Self::TinyEn),
            "base" => Some(Self::Base),
            "base.en" | "base_en" => Some(Self::BaseEn),
            "small" => Some(Self::Small),
            "small.en" | "small_en" => Some(Self::SmallEn),
            _ => None,
        }
    }

    pub fn is_multilingual(&self) -> bool {
        matches!(self, Self::Tiny | Self::Base | Self::Small)
    }

    fn model_and_revision(&self) -> (&'static str, &'static str) {
        match self {
            Self::Tiny => ("openai/whisper-tiny", "main"),
            Self::TinyEn => ("openai/whisper-tiny.en", "refs/pr/15"),
            Self::Base => ("openai/whisper-base", "refs/pr/22"),
            Self::BaseEn => ("openai/whisper-base.en", "refs/pr/13"),
            Self::Small => ("openai/whisper-small", "main"),
            Self::SmallEn => ("openai/whisper-small.en", "refs/pr/10"),
        }
    }
}

pub enum Model {
    Normal(m::model::Whisper),
    Quantized(m::quantized_model::Whisper),
}

impl Model {
    pub fn config(&self) -> &Config {
        match self {
            Self::Normal(m) => &m.config,
            Self::Quantized(m) => &m.config,
        }
    }

    pub fn encoder_forward(&mut self, x: &Tensor, flush: bool) -> candle_core::Result<Tensor> {
        match self {
            Self::Normal(m) => m.encoder.forward(x, flush),
            Self::Quantized(m) => m.encoder.forward(x, flush),
        }
    }

    pub fn decoder_forward(
        &mut self,
        x: &Tensor,
        xa: &Tensor,
        flush: bool,
    ) -> candle_core::Result<Tensor> {
        match self {
            Self::Normal(m) => m.decoder.forward(x, xa, flush),
            Self::Quantized(m) => m.decoder.forward(x, xa, flush),
        }
    }

    pub fn decoder_final_linear(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        match self {
            Self::Normal(m) => m.decoder.final_linear(x),
            Self::Quantized(m) => m.decoder.final_linear(x),
        }
    }

    pub fn reset_kv_cache(&mut self) {
        match self {
            Self::Normal(m) => m.reset_kv_cache(),
            Self::Quantized(m) => m.reset_kv_cache(),
        }
    }
}

pub fn token_id(tokenizer: &Tokenizer, token: &str) -> candle_core::Result<u32> {
    match tokenizer.token_to_id(token) {
        None => candle_core::bail!("no token-id for {token}"),
        Some(id) => Ok(id),
    }
}

#[derive(Debug, Clone)]
struct DecodingResult {
    text: String,
    avg_logprob: f64,
    no_speech_prob: f64,
}

struct Decoder {
    model: Model,
    rng: rand::rngs::StdRng,
    timestamps: bool,
    tokenizer: Tokenizer,
    suppress_tokens: Tensor,
    sot_token: u32,
    transcribe_token: u32,
    eot_token: u32,
    no_speech_token: u32,
    no_timestamps_token: u32,
    language_token: Option<u32>,
}

impl Decoder {
    fn new(
        model: Model,
        tokenizer: Tokenizer,
        seed: u64,
        device: &Device,
        language_token: Option<u32>,
        timestamps: bool,
    ) -> Result<Self> {
        let no_timestamps_token = token_id(&tokenizer, m::NO_TIMESTAMPS_TOKEN)?;
        let suppress_tokens: Vec<f32> = (0..model.config().vocab_size as u32)
            .map(|i| {
                if model.config().suppress_tokens.contains(&i)
                    || timestamps && i == no_timestamps_token
                {
                    f32::NEG_INFINITY
                } else {
                    0f32
                }
            })
            .collect();
        let suppress_tokens = Tensor::new(suppress_tokens.as_slice(), device)?;
        let sot_token = token_id(&tokenizer, m::SOT_TOKEN)?;
        let transcribe_token = token_id(&tokenizer, m::TRANSCRIBE_TOKEN)?;
        let eot_token = token_id(&tokenizer, m::EOT_TOKEN)?;
        let no_speech_token = m::NO_SPEECH_TOKENS
            .iter()
            .find_map(|token| token_id(&tokenizer, token).ok())
            .ok_or_else(|| E::msg("unable to find any non-speech token"))?;

        Ok(Self {
            model,
            rng: rand::rngs::StdRng::seed_from_u64(seed),
            timestamps,
            tokenizer,
            suppress_tokens,
            sot_token,
            transcribe_token,
            eot_token,
            no_speech_token,
            language_token,
            no_timestamps_token,
        })
    }

    fn decode(&mut self, mel: &Tensor, t: f64) -> Result<DecodingResult> {
        let audio_features = self.model.encoder_forward(mel, true)?;
        let sample_len = self.model.config().max_target_positions / 2;
        let mut sum_logprob = 0f64;
        let mut no_speech_prob = f64::NAN;
        let mut tokens = vec![self.sot_token];
        if let Some(language_token) = self.language_token {
            tokens.push(language_token);
        }
        tokens.push(self.transcribe_token);
        if !self.timestamps {
            tokens.push(self.no_timestamps_token);
        }

        for i in 0..sample_len {
            let tokens_t = Tensor::new(tokens.as_slice(), mel.device())?.unsqueeze(0)?;
            let ys = self
                .model
                .decoder_forward(&tokens_t, &audio_features, i == 0)?;

            if i == 0 {
                let logits = self
                    .model
                    .decoder_final_linear(&ys.i(..1)?)?
                    .i(0)?
                    .i(0)?;
                no_speech_prob = softmax(&logits, 0)?
                    .i(self.no_speech_token as usize)?
                    .to_scalar::<f32>()? as f64;
            }

            let (_, seq_len, _) = ys.dims3()?;
            let logits = self
                .model
                .decoder_final_linear(&ys.i((..1, seq_len - 1..))?)?
                .i(0)?
                .i(0)?;
            let logits = logits.broadcast_add(&self.suppress_tokens)?;

            let next_token = if t > 0f64 {
                let prs = softmax(&(&logits / t)?, 0)?;
                let logits_v: Vec<f32> = prs.to_vec1()?;
                let distr = WeightedIndex::new(&logits_v)?;
                distr.sample(&mut self.rng) as u32
            } else {
                let logits_v: Vec<f32> = logits.to_vec1()?;
                logits_v
                    .iter()
                    .enumerate()
                    .max_by(|(_, u), (_, v)| u.total_cmp(v))
                    .map(|(i, _)| i as u32)
                    .unwrap()
            };
            tokens.push(next_token);
            let prob = softmax(&logits, candle_core::D::Minus1)?
                .i(next_token as usize)?
                .to_scalar::<f32>()? as f64;
            if next_token == self.eot_token || tokens.len() > self.model.config().max_target_positions
            {
                break;
            }
            sum_logprob += prob.ln();
        }

        let text = self.tokenizer.decode(&tokens, true).map_err(E::msg)?;
        Ok(DecodingResult {
            text,
            avg_logprob: sum_logprob / tokens.len().max(1) as f64,
            no_speech_prob,
        })
    }

    fn decode_with_fallback(&mut self, segment: &Tensor) -> Result<DecodingResult> {
        for (i, &t) in m::TEMPERATURES.iter().enumerate() {
            let dr = self.decode(segment, t);
            if i == m::TEMPERATURES.len() - 1 {
                return dr;
            }
            match dr {
                Ok(dr) => {
                    let needs_fallback = dr.avg_logprob < m::LOGPROB_THRESHOLD;
                    if !needs_fallback || dr.no_speech_prob > m::NO_SPEECH_THRESHOLD {
                        return Ok(dr);
                    }
                }
                Err(err) => eprintln!("Decode error at {t}: {err}"),
            }
        }
        unreachable!()
    }

    fn run(&mut self, mel: &Tensor) -> Result<Vec<String>> {
        let (_, _, content_frames) = mel.dims3()?;
        let mut seek = 0;
        let mut texts = Vec::new();
        while seek < content_frames {
            let segment_size = usize::min(content_frames - seek, m::N_FRAMES);
            let mel_segment = mel.narrow(2, seek, segment_size)?;
            let dr = self.decode_with_fallback(&mel_segment)?;
            seek += segment_size;
            if dr.no_speech_prob > m::NO_SPEECH_THRESHOLD && dr.avg_logprob < m::LOGPROB_THRESHOLD {
                continue;
            }
            let text = dr.text.trim();
            if !text.is_empty() {
                texts.push(text.to_string());
            }
        }
        Ok(texts)
    }

    fn set_language_token(&mut self, language_token: Option<u32>) {
        self.language_token = language_token;
    }

    fn model_mut(&mut self) -> &mut Model {
        &mut self.model
    }
}

/// Lädt Modell und Tokenizer vom HuggingFace Hub
pub fn load_model(model_name: &str, device: &Device) -> Result<(Model, Tokenizer, Config)> {
    let which = WhichModel::from_str(model_name)
        .ok_or_else(|| E::msg(format!("Unbekanntes Modell: {model_name}")))?;
    let (model_id, revision) = which.model_and_revision();

    let api = Api::new()?;
    let repo = api.repo(Repo::with_revision(
        model_id.to_string(),
        RepoType::Model,
        revision.to_string(),
    ));

    let config_path = repo.get("config.json")?;
    let tokenizer_path = repo.get("tokenizer.json")?;
    let weights_path = repo.get("model.safetensors")?;

    let config: Config = serde_json::from_str(&std::fs::read_to_string(config_path)?)?;
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(E::msg)?;

    let model = {
        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(&[weights_path], m::DTYPE, device)?
        };
        Model::Normal(m::model::Whisper::load(&vb, config.clone())?)
    };

    Ok((model, tokenizer, config))
}

fn load_mel_filters() -> Vec<f32> {
    let mel_bytes = include_bytes!("../melfilters.bytes");
    mel_bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Transkribiert PCM-Audio (16 kHz, f32 mono) zu Text
pub fn transcribe_pcm(
    pcm: &[f32],
    model: &str,
    language: Option<&str>,
) -> Result<Vec<String>> {
    let device = Device::Cpu;
    let (mut whisper_model, tokenizer, config) = load_model(model, &device)?;
    let mel_filters = load_mel_filters();

    let mel = audio::pcm_to_mel(&config, pcm, &mel_filters);
    let mel_len = mel.len();
    let mel_t = Tensor::from_vec(
        mel,
        (1, config.num_mel_bins, mel_len / config.num_mel_bins),
        &device,
    )?;

    let language_token = match (WhichModel::from_str(model).map(|m| m.is_multilingual()), language)
    {
        (Some(true), None) => Some(multilingual::detect_language(
            &mut whisper_model,
            &tokenizer,
            &mel_t,
        )?),
        (Some(true), Some(lang)) => token_id(&tokenizer, &format!("<|{lang}|>")).ok(),
        _ => None,
    };

    let mut decoder = Decoder::new(
        whisper_model,
        tokenizer,
        299792458,
        &device,
        language_token,
        false,
    )?;

    decoder.run(&mel_t)
}
