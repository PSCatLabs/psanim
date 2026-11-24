#!/usr/bin/env bash
# Time every gallery piece at full size: a Markdown table on stdout.
#   scripts/bench_gallery.sh > /tmp/bench.md
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q -p psanim-cli
P=target/release/psanim
echo "| Piece | Mode | Size | Frames | Load ms | Median ms/frame | p95 | 1 thread fps | Threads | All-core fps |"
echo "|---|---|---|---|---|---|---|---|---|---|"
for d in gallery/*/; do
  name=$(basename "$d")
  $P bench "gallery/$name/$name.ps" --json --count "${COUNT:-48}" > /tmp/psanim-bench.json
  $P info "gallery/$name/$name.ps" --json > /tmp/psanim-info.json
  python3 - "$name" <<'PY'
import json, sys
b = json.load(open('/tmp/psanim-bench.json')); i = json.load(open('/tmp/psanim-info.json'))['movie']
print("| %s | %s | %dx%d | %d | %.1f | %.1f | %.1f | %.1f | %d | %.1f |" % (
    sys.argv[1], b['mode'], b['width'], b['height'], i['frames'], b['load_ms'],
    b['frame_ms']['median'], b['frame_ms']['p95'], b['serial_fps'], b['threads'], b['parallel_fps']))
PY
done
