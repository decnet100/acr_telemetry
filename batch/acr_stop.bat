@echo off
:: Signals acr_recorder to stop by creating the stop file. Does not touch acr_notes. Bind to a game controller button for in-game stop.
mkdir "%APPDATA%\acr_recorder" 2>nul
echo. > "%APPDATA%\acr_recorder\acr_stop"
