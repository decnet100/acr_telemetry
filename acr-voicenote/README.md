# acr_voicenote

Voice-to-text for conferences: record audio with timestamps; append transcriptions to file or send via UDP.

## Features

- **Windows CLI**: Command-line only
- **Configuration**: `config.toml` – output path, language (de/en/auto), UDP
- **Microphone selection**: Lists available devices at startup; select by number
- **Candle-Whisper**: Pure Rust – all dependencies via `cargo` (no CMake, no C++ tools)
- **Output**:
  - **File**: Line format `ISO8601\ttranscript` (tab-separated)
  - **ACR integration**: Set `output.notes_dir` to the same path as acr_recorder's `notes_dir` (default `%APPDATA%\acr_telemetry`). Writes to `acr_notes`; acr_export reads these on export, filters by recording time, and prompts to edit/set label and tags. The recorder does not read or delete `acr_notes`.
  - **UDP**: Optional – same format to another device

## Usage

```text
acr_voicenote [OPTIONS]

OPTIONS:
  -c, --config <PATH>   Path to config.toml
  --list-devices        List audio devices and exit
```

**On first run** the Whisper model is downloaded from HuggingFace (~150 MB for `tiny`).  
Transcriptions run every ~5 seconds; empty results (silence) are not written.

## Configuration

Copy `config.example.toml` to `config.toml` and adjust:

| Setting | Description |
|---------|-------------|
| `output.file_path` | Output file path (ignored when `notes_dir` is set) |
| `output.notes_dir` | For ACR integration: same path as acr_recorder `notes_dir` |
| `output.udp.enabled` | Enable UDP output |
| `output.udp.host` / `port` | UDP target |
| `speech.language` | `"de"`, `"en"` or `"auto"` |
| `whisper.model` | Model: `tiny`, `base`, `small`, `tiny.en`, `base.en`, `small.en` |

## Build

```bash
cd acr-voicenote
cargo build --release
```

All dependencies (including Candle-Whisper) are installed via `cargo` – no external build tools required.

## Project structure

```text
acr-voicenote/
├── Cargo.toml
├── config.example.toml
├── melfilters.bytes
├── README.md
└── src/
    ├── main.rs
    ├── config.rs
    └── whisper_mod/
        ├── mod.rs
        └── multilingual.rs
```
