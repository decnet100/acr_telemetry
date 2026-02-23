@echo off
:: Signals acr_recorder to stop by creating an empty file in a directory it watches. Binding this to a game controller button for in-game stop is a lot more efficient than listening for a hotkey.
mkdir "%APPDATA%\acr_recorder" 2>nul
echo. > "%APPDATA%\acr_recorder\acr_stop"
