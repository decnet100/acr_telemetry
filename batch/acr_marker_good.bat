@echo off
:: Appends real time + recording elapsed time + "#marker good#" to acr_notes. Does NOT stop the recording. Use to mark good moments during a run.
:: Path: %APPDATA%\acr_telemetry\ (e.g. C:\Users\<user>\AppData\Roaming\acr_telemetry\)
set "NOTES_DIR=%APPDATA%\acr_telemetry"
set "NOTES_FILE=%NOTES_DIR%\acr_notes"
mkdir "%NOTES_DIR%" 2>nul
for /f "usebackq delims=" %%T in (`powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-dd HH:mm:ss'"`) do set RT=%%T
set "ELAPSED=0"
if exist "%NOTES_DIR%\acr_elapsed_secs" for /f "usebackq delims=" %%E in ("%NOTES_DIR%\acr_elapsed_secs") do set ELAPSED=%%E
echo [%RT%] [elapsed %ELAPSED%s] #marker good#>> "%NOTES_FILE%"
