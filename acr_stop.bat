@echo off
:: Signals acr_recorder to stop. Bind this to a game controller button for in-game stop.
mkdir "%APPDATA%\acr_recorder" 2>nul
echo. > "%APPDATA%\acr_recorder\acr_stop"
