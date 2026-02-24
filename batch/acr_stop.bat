@echo off
:: Signals acr_recorder to stop by creating the stop file. Does not touch acr_notes. Bind to a game controller button for in-game stop.
:: Path: %APPDATA%\acr_telemetry\ (e.g. C:\Users\<user>\AppData\Roaming\acr_telemetry\)
mkdir "%APPDATA%\acr_telemetry" 2>nul
echo. > "%APPDATA%\acr_telemetry\acr_stop"
