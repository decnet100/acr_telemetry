# ACR Telemetry Bridge – Live Telemetry Dashboard

Web dashboard for monitoring ACC/AC Rally temperatures and telemetry on a second device (phone, tablet, laptop).

# Requirements

- **acr_telemetry_bridge** running on the gaming PC (Use binary release .exe, or build yourself using `cargo build --release` from repo root)
- Gaming PC and receiver on the same WLAN/LAN (simplest for testing: use browser on gaming computer, keep game window visible and alt+tab open the browser.

## Usage

1. **On the gaming PC**, start the game. It needs to have loaded the main menu when you start the bridge, for this to connect successfully. With this example command, you are creating a http server connection at all IP addresses, at port 8080, with an update rate of 5 per second:
   ```bash
   acr_telemetry_bridge --http 0.0.0.0:8080 --rate 5
   ```
   Create a config: `acr_telemetry_bridge.toml` next to the executable in the same directory (or CWD, or `~/.config/acr_recorder/`). If this is set, you can simply call acr_telemetry_bridge.exe without arguments.

2. **On phone or second device**, open in browser:
   ```
   http://<GAMING_PC_IP>:8080 (if using on the same PC, you can simply use http://localhost:8080. If you want to connect from another device on your local network (WLAN or LAN), you can press Win+R keys, enter cmd, type "ipconfig". This should show a listing which contains your IPv4 address - four numbers separated by periods, i.e. 192.168.1.42).
   ```
   Example: `http://192.168.1.42:8080`. 


3. The dashboard auto-refreshes and shows configurable fields (speed, temps, fuel, etc.). You can control the content and order of fields in the `acr_telemetry_bridge.toml` and coloring in the `telemetry_color.toml`.

## Configuration

`acr_telemetry_bridge.toml`:

Full field list: see **[FIELDS.md](FIELDS.md)**.

```toml
rate_hz = 5
http_addr = "0.0.0.0:8080"
temperature_unit = "c"   # "c", "f", "k"
dashboard_slots = ["water_temp", "road_temp", "tyre_fl", "speed_kmh", "gear", "rpm"]
```

### telemetry_color.toml

Threshold-based coloring. Place next to the executable or in `~/.config/acr_recorder/`.

- **[colors]** – Hex colors for levels: very_low, low, normal, high, very_high, ignore/unknown. 
- **[fields.field_id]** – Per-field thresholds. If no threshold is set, or the value for the field is below the very_low threshold, the color set for ignore/unknown will be used.

## CLI Options

| Option | Description |
|--------|-------------|
| `--rate N` | Update rate in Hz |
| `--http [ADDR]` | Serve HTTP dashboard |
| `--udp HOST:PORT` | Send JSON over UDP |
| `--unit c\|f\|k` | Temperature unit |


### Troubleshooting:

If that port 8080 is taken (it's the standard port for web-based services like ours, but also many other similar tools), the executable will give you an error. Then you can just pick another number that isn't take (i.e. try 8081, 8082...). Of course, also use this chosen port number when connecting to the bridge.
