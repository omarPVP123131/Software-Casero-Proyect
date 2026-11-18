#!/usr/bin/env bash
# Captura la ventana Tauri real (no el navegador) bajo Xvfb.
#   ./scripts/capture-app.sh salida.png [WxH]
# Requiere: xvfb-run, xdotool, imagemagick. Necesita target/release/photolab.
set -euo pipefail
OUT="${1:-/home/user/photolab/assets/app-tauri.png}"
SIZE="${2:-1600x1000}"
cd "$(dirname "$0")/.."

Xvfb :98 -screen 0 "${SIZE}x24" -nolisten tcp &
XPID=$!
sleep 2
DISPLAY=:98 ./target/release/photolab >/tmp/app-capture.log 2>&1 &
APID=$!
sleep 7                                  # WebView + primera escena
DISPLAY=:98 xdotool search --name "PhotoLab" windowactivate --sync 2>/dev/null || true
DISPLAY=:98 import -window root "$OUT"
kill "$APID" 2>/dev/null || true; sleep 1
kill "$XPID" 2>/dev/null || true
echo "captura → $OUT"
