# ACR Bridge – Live Telemetry Dashboard

Web dashboard for monitoring ACC/AC Rally temperatures and telemetry on a second device (phone, tablet, laptop).

## Requirements

- **acr_telemetry_bridge** running on the gaming PC (`cargo build --release` from repo root)
- Gaming PC and receiver on the same network (or localhost for testing)

## Usage

1. **On the gaming PC**, start the bridge:
   ```bash
   acr_telemetry_bridge --http 0.0.0.0:8080 --rate 5
   ```
   Config: `acr_telemetry_bridge.toml` next to the executable (or CWD, or `~/.config/acr_recorder/`).

2. **On phone or second device**, open in browser:
   ```
   http://<GAMING_PC_IP>:8080
   ```
   Example: `http://192.168.1.42:8080`

3. The dashboard auto-refreshes and shows configurable fields (speed, temps, fuel, etc.).

## Configuration

`acr_telemetry_bridge.toml`:

```toml
rate_hz = 5
http_addr = "0.0.0.0:8080"
temperature_unit = "c"   # "c", "f", "k"
dashboard_slots = ["water_temp", "road_temp", "tyre_fl", "speed_kmh", "gear", "rpm"]
```

### telemetry_color.toml

Threshold-based coloring. Place next to the executable or in `~/.config/acr_recorder/`.

- **[colors]** – Hex colors for levels: very_low, low, normal, high, very_high, ignore
- **[fields.field_id]** – Per-field thresholds.

## CLI Options

| Option | Description |
|--------|-------------|
| `--rate N` | Update rate in Hz |
| `--http [ADDR]` | Serve HTTP dashboard |
| `--udp HOST:PORT` | Send JSON over UDP |
| `--unit c\|f\|k` | Temperature unit |
