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

3. **Record**: Run `target/release/acr_recorder.exe` while ACC/AC Rally is running. **Ctrl+C** to stop, or run `acr_stop.bat` (bind to game controller for in-game stop).

4. **Export**: `target/release/acr_export telemetry_raw --sqlite` to create a SQLite database for Grafana.

5. **Bridge** (live dashboard): `target/release/acr_telemetry_bridge` – serves the web UI, config in `acr_telemetry_bridge.toml`.

6. **Dashboard**: See `grafanimate/DASHBOARD_SETUP.md` for Grafana setup.

## Documentation
- **[docs/FIELDS.md](docs/FIELDS.md)** – Available telemetry fields (data variables) with small description of what they do.
- **[docs/BRIDGE.md](docs/BRIDGE.md)** – Bridge web dashboard readme.
- **[grafana/DASHBOARD_SETUP.md](grafana/DASHBOARD_SETUP.md)** – Grafana installation and dashboard setup
- **[vendor/acc_shared_memory_rs/](vendor/acc_shared_memory_rs/)** – ACC shared memory library

## License

PolyForm Noncommercial License 1.0.0
=======
# acr_telemetry
Telemetry tools for Assetto Corsa Rally

