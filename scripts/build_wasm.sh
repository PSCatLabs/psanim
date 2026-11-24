#!/usr/bin/env bash
# Build the browser engine and put it beside the player: web/psanim.wasm.
#   scripts/build_wasm.sh
# Needs the wasm32-unknown-unknown target (rustup target add wasm32-unknown-unknown).
# No wasm-bindgen or wasm-pack: the module exports a plain C ABI (crates/psanim-wasm).
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --profile wasm -p psanim-wasm --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/wasm/psanim_wasm.wasm web/psanim.wasm
ls -l web/psanim.wasm | awk '{printf "web/psanim.wasm: %.1f MB", $5/1e6}'
printf ' (%.1f MB gzipped)\n' "$(gzip -9 -c web/psanim.wasm | wc -c | awk '{print $1/1e6}')"
