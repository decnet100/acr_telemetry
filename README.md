# ACR – Assetto Corsa Recorder & Grafana Telemetry

Introductory video:
https://youtu.be/IYoPnljtn9o

Telemetry recording and analysis for **Assetto Corsa Competizione (ACC)** and **Assetto Corsa Rally**. Records physics data at ~333 Hz and exports to CSV, MoTeC LD (minimal working PoC), or SQLite for Grafana dashboards.

### MoTeC LD status (current)

- The LD export is currently a **minimal compatible implementation** (validated with MoTeC i2 + RBR Motec v105 workspace).
- Core channels and several RBR workspace-compatible aliases are exported (e.g. `speed`, `throttle`, `brake`, `steering`, `engineRotation`, `gear_ok`, G-force and selected suspension/tyre channels).
- This is still a staged rollout; not all workspace channels are mapped yet.
- Feedback is very welcome: if a channel looks wrong or missing, please open an issue and include the `.rkyv` filename and workspace used.

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

This compilation uses Rust (https://rust-lang.org/tools/install/). Please install this before, including the Visual Studio C++ build tools, and make sure your cargo - tool (Rust's package manager) can be reached in your command line by typing 
   ```cmd
   cargo
   ```

   This should return the related help.

1. **Build** from repo root:
   ```bash
   cargo build --release
   ```
   This will automatically download all required libraries. Binaries will be created in `target/release/`: `acr_recorder.exe`, `acr_export.exe`, `acr_telemetry_bridge.exe`.

2. **Configure**
   - **Recorder / Export:** `acr_recorder.toml` in the current working directory or `~/.config/acr_recorder/config.toml`. Copy from **`config-examples/acr_recorder.toml`** and adjust paths.
   - **Bridge:** `acr_telemetry_bridge.toml` next to the bridge executable, or in CWD, or `~/.config/acr_recorder/acr_telemetry_bridge.toml`. Example: **`config-examples/acr_telemetry_bridge.toml`**.
   - **Please adapt the paths in these configuration files**

3. **Run from target/release/ or copy elsewhere/**  
   If you use the executables in a different folder for convenience, copy the toml config files from config-examples along with them so they're in the same folder. 

4. **Record**: Run `acr_recorder.exe` (from `bin/` or `target/release/`) while ACC/AC Rally is running. By default, physics (~333 Hz) and **graphics** (~60 Hz, for Grafana e.g. `distance_traveled`) are recorded. Disable graphics in `acr_recorder.toml` with `record_graphics = false` or via `--no-graphics`. **Ctrl+C** to stop, or run `batch\acr_stop.bat` (bind to game controller for in-game stop). The stop file is created in the **notes directory** (default `%APPDATA%\acr_telemetry` on Windows).

5. **Export**: `acr_export telemetry_raw --sqlite` (or `acr_export --rawDir --sqlite` to use the raw dir from config). Creates/updates the SQLite database for Grafana.

6. **Bridge** (live dashboard): Run `acr_telemetry_bridge.exe` – serves the web UI; config in `acr_telemetry_bridge.toml` (see [docs/BRIDGE.md](docs/BRIDGE.md)).

7. **Dashboard**: See `grafana/DASHBOARD_SETUP.md` for Grafana setup. I will later add more dashboards to allow better analysis of your telemetry (currently working on a way to display tyre performance over other related variables such as temperature and camber).

## Telemetry bridge (live monitoring)

The **acr_telemetry_bridge** is a separate program for **live** telemetry: it reads ACC/AC Rally shared memory at low rate (1–10 Hz) and sends data via UDP and/or a small HTTP server. Use it to view temperatures (or other values - it supports every variable the game makes available) on a phone or second device while driving. Config: `acr_telemetry_bridge.toml` (next to the executable or in `~/.config/acr_recorder/`). See **[docs/BRIDGE.md](docs/BRIDGE.md)** for setup and the **acr_receiver/** web UI.

## Recording notes (voice / manual)

Since it's easy to record dozens of tracks and AC Rallys internal description is currently still lacking even basic info such as location, it's very useful to attach a short description to each recording (e.g. “comparison of ABS levels”, “test run aborted”) so you can tell them apart later in Grafana or in the SQLite DB. For example, anything that will be entered as text into  `%APPDATA%\acr_telemetry\acr_note` will be saved as part of the dataset when the acr_recorder finishes.

This folder `%APPDATA%\acr_telemetry` will serve as the location for temporary notes.



- **Suggestion: Voice / external tool while recording**: To avoid switching away from the game (handy in VR or when the game is slow to alt-tab), an external tool – such as voice-to-text – can write notes into `acr_notes`, which acr_export reads on export. **acr-voicenote** (in `acr-voicenote/`) writes to `acr_notes`; set its `output.notes_dir` to the same path as acr_recorder’s `notes_dir`. On export, acr_export prompts to edit notes and suggests a recording label from the first five voice notes.

  **Stop vs notes**: The stop signal is the existence of `acr_stop`; notes are in `acr_notes` and `acr_<field>`. These files live in the **notes directory** (default `%APPDATA%\acr_telemetry` on Windows, `~/.config/acr_telemetry` on Linux). Configure via **notes_dir** in `acr_recorder.toml` if needed. You can append to `acr_notes` during a run; the recorder only stops when `acr_stop` appears. The recorder does **not** reset `acr_notes`; the file persists across recordings.

  **When**: Your tool should append lines to `acr_notes` whenever the user speaks a note (e.g. “telemetry note, comparison of abs level efficiency”) or a stop reason (e.g. “telemetry stop, test run aborted, crash”). Including a timestamp in each line is recommended.

  **Recorder on stop:** Writes only recording start and end times to `<stem>.notes.json`. It does **not** read or delete `acr_notes`. During recording it writes `acr_elapsed_secs` (current recording time in seconds) for batch scripts.
  - **Manual**: Edit the SQLite database (e.g. with [DB Browser for SQLite](https://sqlitebrowser.org/)). The **`recording_notes`** table has one row per recording (`recording_id`) with TEXT fields you can fill: `notes`, `laptime`, `result`, `driver_impression`, `tested_parameters`, `conditions`, `setup_notes`, `session_goal`, `incident`. All are optional and default to empty.

  **Batch helpers** (in `batch/`): They write into the notes directory (default `%APPDATA%\acr_telemetry`). `acr_stop.bat` only creates `acr_stop`. `acr_marker_good.bat` appends a marker "good" to this timestamp and does not stop; `acr_marker_bad.bat` appends a “bad” marker; `acr_note_aborted.bat` appends then stops. These can be run from anywhere (e.g. bound to game controller) and appear in the grafana visualisation as vertical lines.

  **Export**: `acr_export ... --sqlite` reads `acr_notes` from the notes directory, filters by recording time (10 s padding before/after), and interactively prompts: include notes, edit/delete, set recording label (suggested from first 5 voice notes), add tags. Writes to `<stem>.notes.json`, `recording_notes`, `annotations`, and tag tables (`acr_telemetry_tags`, `acr_tag_lookup`). Sync annotations from physics (e.g. `sync_air_temp_gt_0`, `sync_speed_gt_0`) appear as vertical lines on dashboards.



## Documentation
- **[docs/EXPORT.md](docs/EXPORT.md)** – acr_export: options, batch mode, CSV vs SQLite, sidecars.
- **[docs/FIELDS.md](docs/FIELDS.md)** – Available telemetry fields (data variables) with short descriptions.
- **[docs/GRAPHICS_AND_STATISTICS.md](docs/GRAPHICS_AND_STATISTICS.md)** – Details about graphics/statistics outputs (at 0.3: very minimal content).
- **[docs/BRIDGE.md](docs/BRIDGE.md)** – Bridge web dashboard readme.
- **[grafana/DASHBOARD_SETUP.md](grafana/DASHBOARD_SETUP.md)** – Grafana installation and dashboard setup.
- **[grafana/ANNOTATIONS.md](grafana/ANNOTATIONS.md)** – Using recording markers as Grafana annotations.
- **[grafana/ANALYSIS_RANGES.md](grafana/ANALYSIS_RANGES.md)** – Analysis segments from Grafana annotations (`acr_analysis_export`).
- **[vendor/acc_shared_memory_rs/](vendor/acc_shared_memory_rs/)** – ACC shared memory library.

## License

PolyForm Noncommercial License 1.0.0
