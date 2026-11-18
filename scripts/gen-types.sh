#!/usr/bin/env bash
# Regenera los tipos TypeScript desde Rust (ts-rs).
set -euo pipefail
cd "$(dirname "$0")/.."
PHOTOLAB_TS_OUT="$(pwd)/frontend/src/lib/generated" cargo test -p photolab-model --lib -- export_bindings --quiet
echo "Tipos TS regenerados en frontend/src/lib/generated/"
