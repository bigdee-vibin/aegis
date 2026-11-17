@echo off
rem Aegis demo launcher: double-click to open the demo in your browser.
rem Serves the static site/ folder from this machine only (127.0.0.1). Simulation, no real firewall.
setlocal
cd /d "%~dp0..\site" || (echo Could not find the site folder. & pause & exit /b 1)
set PORT=8080
where python >nul 2>nul
if errorlevel 1 (
  echo Python was not found. Opening the hosted demo instead.
  start "" "https://bigdee-vibin.github.io/aegis/"
  exit /b 0
)
echo Aegis demo running at http://127.0.0.1:%PORT%/  (close this window to stop)
start "" "http://127.0.0.1:%PORT%/"
python -m http.server %PORT% --bind 127.0.0.1
