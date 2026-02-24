@echo off
:: Appends real time + recording elapsed + "#marker aborted#" to acr_notes, then signals stop. Use when the run was aborted (crash, offtrack, etc.).
set "NOTES_DIR=%~dp0..\telemetry_raw"
set "NOTES_FILE=%NOTES_DIR%\acr_notes"
mkdir "%NOTES_DIR%" 2>nul
for /f "usebackq delims=" %%T in (`powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-dd HH:mm:ss'"`) do set RT=%%T
set "ELAPSED=0"
if exist "%NOTES_DIR%\acr_elapsed_secs" for /f "usebackq delims=" %%E in ("%NOTES_DIR%\acr_elapsed_secs") do set ELAPSED=%%E
echo [%RT%] [elapsed %ELAPSED%s] #marker aborted#>> "%NOTES_FILE%"
mkdir "%APPDATA%\acr_recorder" 2>nul
echo. > "%APPDATA%\acr_recorder\acr_stop"
