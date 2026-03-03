//! acr-voicenote – Voice-to-text for conferences
//!
//! Record audio with timestamps; append transcriptions to file or send via UDP.

mod config;
mod whisper_mod;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use config::Config;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
#[derive(Parser)]
#[command(name = "acr-voicenote")]
#[command(about = "Voice-to-text for conferences: append with timestamps to file or UDP")]
struct Cli {
    /// Path to config.toml (default: config.toml, ~/.config/acr-voicenote/config.toml)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// List audio devices and exit
    #[arg(long)]
    list_devices: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_devices {
        list_audio_devices()?;
        return Ok(());
    }

    let config_path = cli
        .config
        .or_else(config::Config::discover)
        .ok_or_else(|| anyhow::anyhow!("config.toml not found. Use --config <path>."))?;

    let config = Config::load(&config_path)?;

    let device = select_audio_device()?;
    println!("Using device: {}", device.name().unwrap_or_default());

    let file_path = if let Some(ref notes_dir) = config.output.notes_dir {
        notes_dir.join("acr_notes")
    } else {
        config.output.file_path.clone()
    };
    let udp_config = config.output.udp.clone();
    let language = config.speech.language.clone();
    let model_name = config.whisper.model.clone();

    run_voicenote_loop(device, file_path, udp_config, language, model_name)?;

    Ok(())
}

fn list_audio_devices() -> Result<()> {
    let host = cpal::default_host();
    let devices = host.input_devices()?;
    println!("Available input devices (microphones):\n");
    for (i, dev) in devices.enumerate() {
        println!("  [{}] {}", i, dev.name().unwrap_or_else(|_| "<unnamed>".into()));
    }
    Ok(())
}

/// Linear interpolation: arbitrary sample rate → 16 kHz (Whisper standard)
fn resample_to_16k(samples: &[f32], in_rate: u32) -> Vec<f32> {
    const OUT_RATE: u32 = 16000;
    if in_rate == OUT_RATE {
        return samples.to_vec();
    }
    let ratio = in_rate as f64 / OUT_RATE as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    (0..out_len)
        .map(|i| {
            let src_idx = i as f64 * ratio;
            let lo = src_idx.floor() as usize;
            let hi = (lo + 1).min(samples.len().saturating_sub(1));
            let t = (src_idx - lo as f64) as f32;
            samples[lo] * (1.0 - t) + samples[hi] * t
        })
        .collect()
}

fn select_audio_device() -> Result<cpal::Device> {
    let host = cpal::default_host();
    let devices: Vec<_> = host.input_devices()?.collect();
    if devices.is_empty() {
        anyhow::bail!("No microphone found.");
    }

    println!("Select microphone:");
    for (i, dev) in devices.iter().enumerate() {
        println!("  [{}] {}", i, dev.name().unwrap_or_else(|_| "<unnamed>".into()));
    }
    print!("Enter number [0]: ");
    std::io::stdout().flush()?;

    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    let idx: usize = buf.trim().parse().unwrap_or(0);
    devices
        .into_iter()
        .nth(idx)
        .ok_or_else(|| anyhow::anyhow!("Invalid device number"))
}

fn run_voicenote_loop(
    device: cpal::Device,
    file_path: PathBuf,
    udp_config: Option<config::UdpConfig>,
    language: String,
    model_name: String,
) -> Result<()> {
    let udp_enabled = udp_config
        .as_ref()
        .map(|u| u.enabled)
        .unwrap_or(false);

    let udp_socket = if udp_enabled {
        Some(std::net::UdpSocket::bind("0.0.0.0:0")?)
    } else {
        None
    };

    let audio_config = device.default_input_config()?;
    let channel_count = audio_config.channels() as usize;
    let in_sample_rate = audio_config.sample_rate().0 as usize;

    let (tx, rx) = std::sync::mpsc::channel::<Vec<f32>>();

    let stream = device.build_input_stream(
        &audio_config.config(),
        move |pcm: &[f32], _: &cpal::InputCallbackInfo| {
            let mono: Vec<f32> = pcm.iter().step_by(channel_count).copied().collect();
            if !mono.is_empty() {
                let _ = tx.send(mono);
            }
        },
        move |err| eprintln!("Audio error: {err}"),
        None,
    )?;
    stream.play()?;

    let lang = if language == "auto" {
        None
    } else {
        Some(language.as_str())
    };

    println!("Recording started (Ctrl+C to stop). Transcribes every ~5 seconds.\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let mut buffered_pcm = Vec::new();
    const BUF_SECONDS: usize = 5;

    while running.load(Ordering::SeqCst) {
        let pcm = match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(p) => p,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        };
        buffered_pcm.extend_from_slice(&pcm);

        if buffered_pcm.len() < BUF_SECONDS * in_sample_rate {
            continue;
        }

        let resampled = resample_to_16k(&buffered_pcm, in_sample_rate as u32);
        buffered_pcm.clear();

        match whisper_mod::transcribe_pcm(&resampled, &model_name, lang) {
            Ok(texts) => {
                let full_text = texts.join(" ").trim().to_string();
                if full_text.is_empty()
                    || full_text == "..."
                    || full_text.chars().all(|c| c == '.' || c.is_whitespace())
                {
                    continue;
                }
                let ts = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");

                {
                    if let Some(parent) = file_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let mut f = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&file_path)?;
                    writeln!(f, "{}\t{}", ts, full_text)?;
                }
                println!("[{}] {}", ts, full_text);

                if let (Some(ref sock), Some(ref dest)) = (&udp_socket, &udp_config) {
                    if dest.enabled {
                        let payload = format!("{}\t{}\n", ts, full_text);
                        let addr = format!("{}:{}", dest.host, dest.port);
                        let _ = sock.send_to(payload.as_bytes(), &addr);
                    }
                }
            }
            Err(e) => eprintln!("Transcription error: {e}"),
        }
    }

    Ok(())
}
