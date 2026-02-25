# ACR – Assetto Corsa Recorder & Grafana Telemetry

Introductory video:
https://youtu.be/IYoPnljtn9o

Telemetry recording and analysis for **Assetto Corsa Competizione (ACC)** and **Assetto Corsa Rally**. Records physics data at ~333 Hz and exports to CSV, MoTeC LD (not working yet!! help needed by someone who knows that data format), or SQLite for Grafana dashboards.

## Project Structure

| Path | Contents |
|------|----------|
| **src/** | Rust source (acr_recorder, acr_export, acr_telemetry_bridge) |
| **config-examples/** | Example TOML configs: `acr_recorder.toml`, `acr_telemetry_bridge.toml`, `telemetry_color.toml` – copy to CWD or `bin/` as needed. |
| **batch/** | Helper scripts: `acr_stop.bat`, `acr_marker_good.bat`, `acr_marker_bad.bat`, `acr_note_aborted.bat` – write into the notes directory and/or signal stop. |
| **acr_receiver/** | Web dashboard for live telemetry on phone/second device |
| **vendor/** | acc_shared_memory_rs (ACC shared memory) |
| **grafanimate/** | Grafana dashboard, animation tooling |

## Quick Start

1. **Build** from repo root:
   ```bash
   cargo build --release
   ```
   Binaries are in `target/release/`: `acr_recorder.exe`, `acr_export.exe`, `acr_telemetry_bridge.exe`.

2. **Configure**
   - **Recorder / Export:** `acr_recorder.toml` in the current working directory or `~/.config/acr_recorder/config.toml`. Copy from **`config-examples/acr_recorder.toml`** and adjust paths.
   - **Bridge:** `acr_telemetry_bridge.toml` next to the bridge executable, or in CWD, or `~/.config/acr_recorder/acr_telemetry_bridge.toml`. Example: **`config-examples/acr_telemetry_bridge.toml`**.

3. **Run from repo root or from bin/**  
   If you use **bin/** with symlinks to the release binaries and configs, run from `bin/` (e.g. `bin\acr_recorder.exe`). Then configs in `bin/` are used when you run from there. Otherwise run `target/release/acr_recorder.exe` etc. from the repo root (config from root or global config dir).

4. **Record**: Run `acr_recorder.exe` (from `bin/` or `target/release/`) while ACC/AC Rally is running. **Ctrl+C** to stop, or run `batch\acr_stop.bat` (bind to game controller for in-game stop). The stop file is created in the **notes directory** (default `%APPDATA%\acr_telemetry` on Windows).

5. **Export**: `acr_export telemetry_raw --sqlite` (or `acr_export --rawDir --sqlite` to use the raw dir from config). Creates/updates the SQLite database for Grafana.

6. **Bridge** (live dashboard): Run `acr_telemetry_bridge.exe` – serves the web UI; config in `acr_telemetry_bridge.toml` (see [docs/BRIDGE.md](docs/BRIDGE.md)).

7. **Dashboard**: See `grafana/DASHBOARD_SETUP.md` for Grafana setup.

### Bin directory (optional)

You can put **symlinks** in `bin/` pointing to the release binaries and config files so everything is runnable from one folder:

- `acr_recorder.exe` → `../target/release/acr_recorder.exe`
- `acr_export.exe` → `../target/release/acr_export.exe`
- `acr_telemetry_bridge.exe` → `../target/release/acr_telemetry_bridge.exe`
- `acr_recorder.toml`, `acr_telemetry_bridge.toml` (copy from **`config-examples/`** or link to your config)

Then run from `bin/` (e.g. `cd bin` and `.\acr_recorder.exe`). The tools look for config in the current working directory, so keep or link the relevant `.toml` files in `bin/`.

## Telemetry bridge (live monitoring)

The **acr_telemetry_bridge** is a separate program for **live** telemetry: it reads ACC/AC Rally shared memory at low rate (1–10 Hz) and sends data via UDP and/or a small HTTP server. Use it to view temperatures (or other values - it supports every variable the game makes available) on a phone or second device while driving. Config: `acr_telemetry_bridge.toml` (next to the executable or in `~/.config/acr_recorder/`). See **[docs/BRIDGE.md](docs/BRIDGE.md)** for setup and the **acr_receiver/** web UI.

## Recording notes (voice / manual)

You can attach a short description to each recording (e.g. “comparison of ABS levels”, “test run aborted”) so you can tell them apart later in Grafana or in the SQLite DB. For example, anything that will be entered as text into  `%APPDATA%\acr_telemetry\acr_note` will be saved as part of the dataset when the acr_recorder finishes.



- **Voice / external tool**: To avoid switching away from the game (handy in VR or when the game is slow to alt-tab), an external tool can write notes into a file that the recorder reads when you stop.

  **Stop vs notes**: The stop signal is the file `acr_stop`; notes are in `acr_notes` and `acr_<field>`. These files live in the **notes directory** (default `%APPDATA%\acr_telemetry` on Windows, `~/.config/acr_telemetry` on Linux). Configure via **notes_dir** in `acr_recorder.toml` if needed. You can append to `acr_notes` during a run; the recorder only stops when `acr_stop` appears. On start the recorder resets `acr_notes`, `acr_elapsed_secs`, and `acr_<field>` in that directory.

  **When**: Your tool should append lines to `acr_notes` whenever the user speaks a note (e.g. “telemetry note, comparison of abs level efficiency”) or a stop reason (e.g. “telemetry stop, test run aborted, crash”). Including a timestamp in each line is recommended.

  **On stop**: The recorder reads `acr_notes` and any `acr_<field>` from the notes directory, writes a single **`<stem>.notes.json`** (notes, fields, and parsed annotations from `#marker` lines) **next to the `.rkyv`** (in the raw output directory), then removes the source files from the notes directory. During recording it writes `acr_elapsed_secs` (current recording time in seconds) there so batch scripts can add elapsed time to markers.
  - **Manual**: Edit the SQLite database (e.g. with [DB Browser for SQLite](https://sqlitebrowser.org/)). The **`recording_notes`** table has one row per recording (`recording_id`) with TEXT fields you can fill: `notes`, `laptime`, `result`, `driver_impression`, `tested_parameters`, `conditions`, `setup_notes`, `session_goal`, `incident`. All are optional and default to empty.

  **Batch helpers** (in `batch/`): They write into the notes directory (default `%APPDATA%\acr_telemetry`). `acr_stop.bat` only creates `acr_stop`. `acr_marker_good.bat` appends a marker "good" to this timestamp and does not stop; `acr_marker_bad.bat` appends a “bad” marker; `acr_note_aborted.bat` appends then stops. These can be run from anywhere (e.g. bound to game controller) and appear in the grafana visualisation as vertical lines.

  **Export**: `acr_export ... --sqlite` reads each recording’s **`<stem>.notes.json`** (written by the recorder on stop) and fills the `recording_notes` and `annotations` tables.

Content is stored only as plain text (no execution or interpretation). Safe to use with third-party voice-to-text tools.

## Documentation
- **[docs/EXPORT.md](docs/EXPORT.md)** – acr_export: options, batch mode, CSV vs SQLite, sidecars.
- **[docs/FIELDS.md](docs/FIELDS.md)** – Available telemetry fields (data variables) with short descriptions.
- **[docs/BRIDGE.md](docs/BRIDGE.md)** – Bridge web dashboard readme.
- **[grafana/DASHBOARD_SETUP.md](grafana/DASHBOARD_SETUP.md)** – Grafana installation and dashboard setup.
- **[grafana/ANNOTATIONS.md](grafana/ANNOTATIONS.md)** – Using recording markers as Grafana annotations.
- **[vendor/acc_shared_memory_rs/](vendor/acc_shared_memory_rs/)** – ACC shared memory library.

## License

PolyForm Noncommercial License 1.0.0
