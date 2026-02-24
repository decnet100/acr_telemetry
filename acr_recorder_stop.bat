@echo off
:: Creates stop file in %APPDATA%\acr_telemetry\ (e.g. C:\Users\<user>\AppData\Roaming\acr_telemetry\)
mkdir "%APPDATA%\acr_telemetry" 2>nul
echo. > "%APPDATA%\acr_telemetry\acr_stop"