# Analysis-Segmente aus Grafana-Annotations

Workflow: Annotations in Grafana mit Tag `rid_<recording_id>` versehen → per Link `acr_analysis_export --serve` aufrufen → Tool schreibt in **analysis.db** (recordings, statics, graphics, analysis mit geschnittenen physics).

## Ablauf

1. **acr_analysis_export --serve** starten (einmalig im Hintergrund).
2. **In Grafana** (Single-Recording-Dashboard): Annotations anlegen (Strg+Ziehen) und Tag `rid_55` setzen (für Recording 55).
3. **Dashboard-Link** (Button): `http://localhost:9876/export?recording_id=${recording_id}`
4. Klick auf den Link → Tool liest `grafana.db`, erstellt Backup `analysis.db.bak`, schreibt in **analysis.db**.

## acr_analysis_export

Schreibt in **analysis.db** (gleiches Verzeichnis wie telemetry.db, oder `--analysis-db PATH`). Vor dem Schreiben wird `analysis.db` nach `analysis.db.bak` gesichert.

**Server-Modus (für Grafana-Links):**
```
acr_analysis_export --serve [--port 9876] [--grafana-db PATH] [--telemetry-db PATH] [--analysis-db PATH]
```

**CLI-Modus:**
```
acr_analysis_export <recording_id> [--grafana-db PATH] [--telemetry-db PATH] [--analysis-db PATH]
```

Pfade: `--grafana-db` (oder `GRAFANA_DB`), `--telemetry-db` (oder acr_recorder.toml), `--analysis-db` (Default: Verzeichnis von telemetry.db + `analysis.db`).

## Inhalt von analysis.db

- **recordings**: Zeilen für die verwendeten recording_ids (Spalte `id`)
- **statics**: Zeilen mit passender recording_id
- **graphics**: nach Zeitbereichen der Annotations geschnitten
- **analysis**: physics geschnitten + annotation_id

## Grafana-Dashboard-Link

1. Dashboard bearbeiten → **Dashboard settings** (Zahnrad) → **Links** → **New link**
2. **Title**: z.B. `In Analysis exportieren`
3. **URL**: `http://localhost:9876/export?recording_id=${recording_id}`
4. **Open in new tab**: aktivieren
5. Speichern

Die Variable `recording_id` muss im Dashboard existieren (z.B. Recording-Dropdown). In **AC Rally full** ist der Link bereits eingebaut.

