# acr_export – Export rkyv telemetry to CSV, MoTeC LD, or SQLite

`acr_export` reads raw `.rkyv` files produced by **acr_recorder** and exports them to:

- **SQLite** – for use with Grafana (recommended for analysis). Fills `recordings`, `physics`, `statics`, `graphics`, `recording_notes`, and `annotations`.
- **CSV** – MoTeC-style CSV plus a separate graphics CSV if a `.graphics.rkyv` sidecar exists.
- **MoTeC LD** – always written when exporting to CSV (single-file mode). LD export is experimental and may not load correctly in MoTeC tools.

You must choose **either** `--csv` or `--sqlite`; they cannot be combined in one run.

---

## Usage

```text
acr_export [--rawDir] [<input.rkyv|directory>] [--csv | --sqlite [db_path]]
```

- **Single file:** pass a path to one `.rkyv` file.
- **Batch (directory):** pass a directory path; all `.rkyv` files inside it are exported (files named `*.graphics.rkyv` are ignored as input; they are used as sidecars when present).
- **Batch (config):** use `--rawDir` (or `--raw-dir`) to use the **raw_output_dir** from `acr_recorder.toml` as the directory. No path argument needed.

If you omit both `--csv` and `--sqlite`, the tool uses the **default_method** from config (`"csv"` or `"sqlite"`). If no config is found, the default is CSV.

---

## Options

| Option | Description |
|--------|-------------|
| `--rawDir` / `--raw-dir` | Use the configured **raw_output_dir** as the input directory (batch mode). Skips files that are already exported (see below). |
| `--csv` | Export to CSV and MoTeC LD. Single file: writes `<stem>.csv`, optionally `<stem>.graphics.csv`, and `<stem>.ld`. |
| `--sqlite [path]` | Export to SQLite. If **path** is omitted, uses **sqlite_db_path** from config (default `telemetry.db`). Path is relative to CWD or absolute. |

**Config:** `./acr_recorder.toml` or `~/.config/acr_recorder/config.toml`.

Relevant keys under `[export]`:

- **default_method** – `"csv"` or `"sqlite"` (used when you don’t pass `--csv` / `--sqlite`).
- **sqlite_db_path** – default SQLite path (e.g. `telemetry.db` or `c:\telemetry\telemetry.db`).

---

## Batch mode and skip rules

In batch mode (directory or `--rawDir`), the tool **skips** a file to avoid duplicate work:

- **SQLite:** skips if a recording with the same **source_file** (the `.rkyv` filename) already exists in the database.
- **CSV:** skips if a `<stem>.csv` file already exists next to the `.rkyv` file.

So you can re-run `acr_export --rawDir --sqlite` after new recordings; only new `.rkyv` files are exported.

---

## Input files and sidecars

For each `<stem>.rkyv` file, the exporter expects (optional):

- **`<stem>.json`** – format metadata written by the recorder; used for **statics** when exporting to SQLite.
- **`<stem>.graphics.rkyv`** – graphics recording; if present, exported to SQLite (graphics table) or to `<stem>.graphics.csv` when using CSV.
- **`<stem>.notes.json`** – notes and annotations written by the recorder on stop; if present, content is written to **recording_notes** and **annotations** in SQLite. See [Recording notes](../README.md#recording-notes-voice--manual) and [Grafana annotations](../grafana/ANNOTATIONS.md).

---

## Examples

Export a single file to SQLite (DB path from config or default):

```bash
acr_export telemetry_raw/acc_physics_1771667046.rkyv --sqlite
```

Export the same file to a specific database:

```bash
acr_export telemetry_raw/acc_physics_1771667046.rkyv --sqlite c:\telemetry\telemetry.db
```

Export all recordings in the configured raw directory to SQLite (batch; skips already exported):

```bash
acr_export --rawDir --sqlite
```

Export a directory to CSV (and LD) using default method from config:

```bash
acr_export telemetry_raw --csv
```

Export a single file to CSV/LD (default when no config or default_method is csv):

```bash
acr_export telemetry_raw/acc_physics_1771667046.rkyv
```

---

## Output summary

| Mode | Output |
|------|--------|
| **SQLite** | One database; each run **appends** new recordings. Tables: `recordings`, `physics`, `statics`, `graphics`, `recording_notes`, `annotations`. |
| **CSV** | Per `.rkyv`: `<stem>.csv`, optionally `<stem>.graphics.csv`, and `<stem>.ld` in the same directory as the input. |

The tool prints the **recording_id** for each new SQLite export so you can use it in Grafana.
