# ACR Telemetry Bridge – Live Telemetry Dashboard

Web dashboard for monitoring ACC/AC Rally temperatures and telemetry on a second device (phone, tablet, laptop).

# Requirements

- **acr_telemetry_bridge** running on the gaming PC. Use the binary from `target/release/acr_telemetry_bridge.exe` after `cargo build --release`, or run from **bin/** if you have symlinks there pointing to the release binaries.
- Gaming PC and receiver on the same WLAN/LAN (simplest for testing: use browser on gaming computer, keep game window visible and alt+tab open the browser.

## Usage

1. **On the gaming PC**, start the game. It needs to have loaded the main menu when you start the bridge, for this to connect successfully. With this example command, you are creating a http server connection at all IP addresses, at port 8080, with an update rate of 5 per second:
   ```bash
   acr_telemetry_bridge --http 0.0.0.0:8080 --rate 5
   ```
   Create a config: `acr_telemetry_bridge.toml` next to the executable (e.g. in `bin/` when using the bin symlinks), or in CWD, or `~/.config/acr_recorder/`. Copy from **`config-examples/acr_telemetry_bridge.toml`** and adjust. If this is set, you can simply call `acr_telemetry_bridge.exe` without arguments.

2. **On phone or second device**, open in browser:
   ```
   http://<GAMING_PC_IP>:8080 (if using on the same PC, you can simply use http://localhost:8080. If you want to connect from another device on your local network (WLAN or LAN), you can press Win+R keys, enter cmd, type "ipconfig". This should show a listing which contains your IPv4 address - four numbers separated by periods, i.e. 192.168.1.42).
   ```
   Example: `http://192.168.1.42:8080`. 


3. The dashboard auto-refreshes and shows configurable fields (speed, temps, fuel, etc.). Configure fields and coloring in **`acr_telemetry_bridge.toml`** (all in one file).

## Recording Status Indicator

The dashboard displays a **red indicator light** in the top-right corner that shows when `acr_recorder` is actively recording:

- **Red pulsing light + "Recording"**: The recorder is currently running and writing telemetry data
- **Gray light + "Recorder: Inactive"**: No active recording session

The bridge automatically detects the recorder status by monitoring the `acr_elapsed_secs` file in the notes directory (default: `%APPDATA%\acr_telemetry` on Windows, `~/.config/acr_telemetry` on Linux). This file is updated every second while the recorder is running.

## Configuration

**`acr_telemetry_bridge.toml`** – bridge options and optional dashboard colors:

Full field list: see **[FIELDS.md](FIELDS.md)**.

```toml
rate_hz = 5
http_addr = "0.0.0.0:8080"
temperature_unit = "c"   # "c", "f", "k"
dashboard_slots = ["water_temp", "road_temp", "tyre_fl", "speed_kmh", "gear", "rpm"]

# Optional: [telemetry_colors] – threshold-based coloring (palette + per-field thresholds)
# Add [telemetry_colors.colors] and [telemetry_colors.fields.<field_id>]. If omitted, defaults are used or telemetry_color.toml is loaded as fallback.
```

### Dashboard colors (optional)

Either in the same file under **[telemetry_colors]**:

- **[telemetry_colors.colors]** – Hex colors for levels: very_low, low, normal, high, very_high, ignore.
- **[telemetry_colors.fields.field_id]** – Per-field thresholds (very_low, low, normal, high, very_high). Omit a field = use default or ignore.

Or in a separate **`telemetry_color.toml`** (same directory or `~/.config/acr_recorder/`) with **[colors]** and **[fields.field_id]** – used as fallback when [telemetry_colors] is not set in the bridge config. Example in repo: **`config-examples/telemetry_color.toml`**.

## CLI Options

| Option | Description |
|--------|-------------|
| `--rate N` | Update rate in Hz |
| `--http [ADDR]` | Serve HTTP dashboard |
| `--udp HOST:PORT` | Send JSON over UDP |
| `--unit c\|f\|k` | Temperature unit |


### Performance Considerations

The bridge is designed to minimize impact on game performance:

- **Low polling rate**: Default 5 Hz (configurable via `--rate`) is sufficient for monitoring and much lower than the recorder's 333 Hz
- **Reduced thread priority**: On Windows, the bridge automatically runs at below-normal priority to avoid interfering with game rendering
- **Cached status checks**: Recorder status is only checked every 2 seconds (not on every telemetry update) to minimize filesystem overhead
- **Separate process**: The bridge runs independently from the recorder

**If you experience micro-stutters (0.2s freezes):**

1. **Lower the update rate**: Try `--rate 2` or `--rate 1` instead of the default 5 Hz
   ```bash
   acr_telemetry_bridge --http 0.0.0.0:8080 --rate 1
   ```
2. **Disable when not needed**: Only run the bridge when you actually need live monitoring. The recorder works independently.
3. **Check system load**: The bridge uses minimal CPU (~0.1-0.5%), but combined with:
   - The recorder (333 Hz physics polling)
   - The game (rendering + physics)
   - Other background processes
   
   ...older systems or high-load scenarios may experience contention for shared memory access.
4. **Use UDP instead of HTTP**: If you're using a custom receiver, UDP mode (`--udp`) has lower overhead than the HTTP server (no web server thread needed)
5. **Monitor CPU affinity**: Consider running the bridge on a different CPU core than the game (use Task Manager > Details > Set Affinity)

**Technical details:**
- The bridge runs at **below-normal thread priority** (Windows) to yield to the game
- Recorder status checks happen only every 2 seconds (not on every telemetry update)
- Shared memory reads are non-blocking and use duplicate detection to skip unchanged data

### Troubleshooting:

If that port 8080 is taken (it's the standard port for web-based services like ours, but also many other similar tools), the executable will give you an error. Then you can just pick another number that isn't take (i.e. try 8081, 8082...). Of course, also use this chosen port number when connecting to the bridge.
