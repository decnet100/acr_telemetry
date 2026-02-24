# ACR – Assetto Corsa Recorder & Grafana Telemetry

Introductory video:
https://youtu.be/IYoPnljtn9o

Telemetry recording and analysis for **Assetto Corsa Competizione (ACC)** and **Assetto Corsa Rally**. Records physics data at ~333 Hz and exports to CSV, MoTeC LD (not working yet!! help needed by someone who knows that data format), or SQLite for Grafana dashboards.

## Project Structure

| Path | Contents |
|------|----------|
| **src/** | Rust source (acr_recorder, acr_export, acr_telemetry_bridge) |
| **acr_receiver/** | Web dashboard for live telemetry on phone/second device |
| **vendor/** | acc_shared_memory_rs (ACC shared memory) |
| **grafanimate/** | Grafana dashboard, animation tooling |

## Quick Start

1. **Build** from repo root:
   ```bash
   cargo build --release
   ```

2. **Configure**: Copy `acr_recorder.toml.example` (if present) or create `acr_recorder.toml` for recorder/export. For the bridge, use `acr_telemetry_bridge.toml` next to the executable.

3. **Record**: Run `target/release/acr_recorder.exe` while ACC/AC Rally is running. **Ctrl+C** to stop, or run `batch\acr_stop.bat` (bind to game controller for in-game stop).

4. **Export**: `target/release/acr_export telemetry_raw --sqlite` to create a SQLite database for Grafana.

5. **Bridge** (live dashboard): `target/release/acr_telemetry_bridge` – serves the web UI, config in `acr_telemetry_bridge.toml`.

6. **Dashboard**: See `grafanimate/DASHBOARD_SETUP.md` for Grafana setup.

## Telemetry bridge (live monitoring)

The **acr_telemetry_bridge** is a separate program for **live** telemetry: it reads ACC/AC Rally shared memory at low rate (1–10 Hz) and sends data via UDP and/or a small HTTP server. Use it to view temperatures (or other values - it supports every variable the game makes available) on a phone or second device while driving. Config: `acr_telemetry_bridge.toml` (next to the executable or in `~/.config/acr_recorder/`). See **[docs/BRIDGE.md](docs/BRIDGE.md)** for setup and the **acr_receiver/** web UI.

## Recording notes (voice / manual)

You can attach a short description to each recording (e.g. “comparison of ABS levels”, “test run aborted”) so you can tell them apart later in Grafana or in the SQLite DB.

- **Manual**: Edit the SQLite database (e.g. with [DB Browser for SQLite](https://sqlitebrowser.org/)). The **`recording_notes`** table has one row per recording (`recording_id`) with TEXT fields you can fill: `notes`, `laptime`, `result`, `driver_impression`, `tested_parameters`, `conditions`, `setup_notes`, `session_goal`, `incident`. All are optional and default to empty.

- **Voice / external tool**: To avoid switching away from the game (handy in VR or when the game is slow to alt-tab), an external tool can write notes into a file that the recorder reads when you stop.

  **Stop vs notes**: The stop signal is the file `acr_stop`; notes are in `acr_notes` and `acr_<field>` in the raw output dir (default `telemetry_raw/`). You can append to `acr_notes` during a run; the recorder only stops when `acr_stop` appears. On start the recorder resets `acr_notes`, `acr_elapsed_secs`, and `acr_<field>` files.

  2. **When**: Your tool should append lines to that file whenever the user speaks a note (e.g. “telemetry note, comparison of abs level efficiency”) or a stop reason (e.g. “telemetry stop, test run aborted, crash”). Including a timestamp in each line is recommended.

  **On stop**: The recorder reads `acr_notes` and any `acr_<field>` files, writes a single **`<stem>.notes.json`** (notes, fields, and parsed annotations from `#marker` lines) next to the `.rkyv`, then removes the source files. During recording it writes `acr_elapsed_secs` (current recording time in seconds) so batch scripts can add elapsed time to markers.

  **Batch helpers** (in `batch/`): `acr_stop.bat` only creates `acr_stop`. `acr_note_good.bat` appends a marker and does not stop; `acr_note_aborted.bat` appends then stops. These should be able to run anywhere. 

  **Export**: `acr_export ... --sqlite` reads each recording’s `.notes` and `.notes_<field>` sidecars and fills the `recording_notes` table.

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
=======
# acr_telemetry
Telemetry tools for Assetto Corsa Rally

