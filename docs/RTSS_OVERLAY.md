# RTSS OSD: erkannte Strecke / freier Text

Dieses Repo kann Text **direkt in RTSS** schreiben, ohne dass der RTSS *Overlay Editor* eine „Datei-Data-Source“ braucht.

RTSS stellt dafür eine **Shared Memory**-Schnittstelle bereit (typischer Name: `RTSSSharedMemoryV2`). Unsere Implementierung folgt dem üblichen Muster aus RTSS-Beispielen/SDK:

- Mapping öffnen: `OpenFileMappingW("RTSSSharedMemoryV2")`
- Signatur prüfen: `RTSS`
- OSD-Einträge über Offsets: `dwOSDArrOffset`, `dwOSDEntrySize`, `dwOSDArrSize`
- Text:
  - RTSS **< 2.7**: `szOSD` (kurz)
  - RTSS **>= 2.7**: `szOSDEx` (lang; typisch bis 4095 Zeichen + NUL)
- Update auslösen: `dwOSDFrame++`
  - Wichtig: In der RTSS v2 Header-Struktur liegt `dwOSDArrSize` bei Byte-Offset **28** und `dwOSDFrame` bei **32**. `acr_recorder` incrementiert `dwOSDFrame` bei Offset 32.

## Voraussetzungen

- **RivaTuner Statistics Server (RTSS)** läuft (Tray/Service).
- RTSS OSD ist im Spiel sichtbar (so wie bei FPS/Afterburner-OSD).

## Binaries

Nach Build findest du die Tools unter `target/release/`:

- `acr_rtss_osd.exe` — generischer RTSS-Text-Pusher
- `acr_track_match.exe` — Track-Matching; optional `--rtss`

Build:

```powershell
cargo build --release --bin acr_rtss_osd --bin acr_track_match
```

## `acr_rtss_osd` (manuell / Scripting)

### Einmaliger Text

```powershell
.\target\release\acr_rtss_osd.exe --owner acr_demo --text "hello from acr"
```

### Text aus Datei

```powershell
.\target\release\acr_rtss_osd.exe --owner acr_demo --file .\note.txt
```

### Datei „followen“ (poll)

```powershell
.\target\release\acr_rtss_osd.exe --owner acr_demo --file "$env:APPDATA\acr_telemetry\acr_detected_track.txt" --follow --poll-ms 200
```

### Slot erzwingen (optional)

Wenn du Kollisionen mit anderen Tools vermeiden willst:

```powershell
.\target\release\acr_rtss_osd.exe --owner acr_demo --text "..." --slot 3
```

`--slot 0` bedeutet: automatisch freien Slot suchen / Owner wiederfinden (Default).

### Aufräumen / freigeben

```powershell
.\target\release\acr_rtss_osd.exe --owner acr_demo --release
```

## `acr_track_match` + RTSS (empfohlen für „detected track …“)

`acr_track_match` kann parallel zur Textdatei auch RTSS updaten:

```powershell
.\target\release\acr_track_match.exe --refs .\reference_tracks --live --rtss --rtss-owner acr_track_match
```

Optional Slot erzwingen:

```powershell
.\target\release\acr_track_match.exe --refs .\reference_tracks --live --rtss --rtss-owner acr_track_match --rtss-slot 3
```

Beim Beenden wird versucht, den Owner-Slot per `release` zu leeren.

## Textdatei (Fallback / andere Overlays)

Standardpfad (wenn nicht überschrieben):

- `%APPDATA%\acr_telemetry\acr_detected_track.txt`

`acr_track_match` schreibt dort **atomisch** (temp + rename), damit Reader nicht „halb“ lesen.

`acr_telemetry_bridge` kann den Text zusätzlich als JSON-Feld bereitstellen:

- `detected_track_message`

## Grenzen / Realitätscheck

- RTSS OSD ist **Text/Markup** (je nach RTSS-Version). Kein „beliebiges Binary“ im Sinne von eingebetteten Objekten über den einfachen Textpfad.
- Unsere Strings gehen über **ANSI `CString`** (wie viele RTSS-Samples): **keine NUL-Bytes** im Text; Umlaute können je nach Codepage/RTSS/OSD anders aussehen.
- RTSS rendert Updates **nicht zwingend pro Frame**; `dwOSDFrame++` ist der übliche „bitte neu zeichnen“-Trigger.

## Troubleshooting

- **`OpenFileMappingW` schlägt fehl**: RTSS läuft nicht / keine Shared Memory Session.
- **Signatur != `RTSS` / „header didn't validate“**:
  - In `acr_recorder` wird beim Öffnen nacheinander versucht:
    - `RTSSSharedMemoryV2`
    - `Global\\RTSSSharedMemoryV2`
    - `Local\\RTSSSharedMemoryV2`
  - Wenn trotzdem keine gültige Signatur gelesen wird, ist typischerweise **kein RTSS‑Shared‑Memory‑Objekt** in deiner Session sichtbar (oder es wird von Policies/AV blockiert), oder es gibt ein **Session/Isolation**-Thema (selten, aber möglich).
  - Praktischer Check: starte RTSS einmal neu und stelle sicher, dass OSD im Spiel wirklich aktiv ist (RTSS zeigt ja sonst auch nichts an).
- **Kein Slot frei**: andere Tools belegen OSD-Slots; `--slot` setzen oder Owner wechseln.
- **`dwOSDEntrySize` wirkt „zu klein“ (z.B. 256)**:
  - Das Feld ist bei manchen RTSS-Versionen **nicht zuverlässig** als „tatsächliche Speicherbreite“ eines Slots zu lesen.
  - `acr_recorder` nutzt deshalb einen **konservativen Mindest‑Stride** (Textfelder + ggf. großer Buffer ab v2.12) und nimmt `max(dwOSDEntrySize, mindestens_nötig)`.
- **Text erscheint, aber „komisch“ formatiert**: RTSS-Markup/Tags prüfen (RTSS-Doku/Foren), Länge prüfen (4095+).
