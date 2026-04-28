# ACR Recorder – Windows Installer

Der Installer legt alle Binaries und Konfigurationsvorlagen ins gewählte Verzeichnis. Pfade in den TOML-Dateien sind **relativ zum Installationsverzeichnis** (Option-3-Verhalten).

## Voraussetzungen

1. **Rust** – Release-Build:
   ```cmd
   cargo build --release
   ```
2. **Inno Setup 6** – [jrsoftware.org](https://jrsoftware.org/ishell.php) (kostenlos). Bei der Installation „Inno Setup Preprocessor“ mit auswählen (für spätere Erweiterungen).

## Installer bauen

Von **Projektroot** aus:

```cmd
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\ACR_Recorder.iss
```

Oder Inno Setup öffnen, `installer\ACR_Recorder.iss` laden und **Build → Compile** (F9).

Die fertige Setup-Datei liegt in:

- `target\installer\ACR_Recorder_0.1.0_setup.exe`

## Was der Installer macht

- Installiert in `%LOCALAPPDATA%\Programs\ACR_Recorder` (oder gewähltes Verzeichnis):
  - `acr_recorder.exe`, `acr_export.exe`, `acr_telemetry_bridge.exe`, `acr_analysis_export.exe`
  - `acr_recorder.toml` (nur wenn noch nicht vorhanden, aus Vorlage mit relativen Pfaden)
  - `acr_telemetry_bridge.toml`, `telemetry_color.toml` (nur wenn noch nicht vorhanden)
  - `batch\` mit `acr_stop.bat`, `acr_marker_good.bat`, etc.
- Startmenü-Einträge: ACR Recorder, ACR Telemetry Bridge, Deinstallieren
- Optional: Desktop- und Quick-Launch-Icons
- Nach der Installation optional: „ACR Recorder starten“ bzw. „Telemetry Bridge starten“

## Version anpassen

In `installer\ACR_Recorder.iss` die Zeile anpassen:

```iss
#define MyAppVersion "0.1.0"
```

Optional: Version aus `Cargo.toml` über ein kleines Script oder den Preprocessor übernehmen.
