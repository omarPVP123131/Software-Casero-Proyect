#!/usr/bin/env bash
# Compila la física a WASM y deja los artefactos en frontend/src/lib/wasm/.
# Requisitos: rustup target add wasm32-unknown-unknown && cargo install wasm-pack
set -euo pipefail
cd "$(dirname "$0")/.."
wasm-pack build crates/wasm \
  --target web \
  --release \
  --out-dir "$(pwd)/frontend/src/lib/wasm" \
  --out-name photolab_wasm
rm -f frontend/src/lib/wasm/.gitignore
echo "WASM listo: frontend/src/lib/wasm/photolab_wasm_bg.wasm"
