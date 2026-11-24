#!/usr/bin/env bash
# Render every gallery piece: the film (MP4 and GIF), a poster frame, a contact sheet, and
# its metadata. Output goes to site/gallery/<name>/ (ignored by git: every
# file there is reproducible from the source with the commands below).
#   scripts/build_gallery.sh [name ...]        # all pieces, or just the named ones
#   PREVIEW=1 scripts/build_gallery.sh         # half size, for a quick look
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q -p psanim-cli
P=target/release/psanim
scale=${PREVIEW:+--scale 0.5}
names=("$@")
if [ ${#names[@]} -eq 0 ]; then
  for d in gallery/*/; do names+=("$(basename "$d")"); done
fi
for name in "${names[@]}"; do
  src=gallery/$name/$name.ps
  [ -f "$src" ] || { echo "no $src" >&2; exit 1; }
  out=site/gallery/$name
  mkdir -p "$out"
  echo "== $name"
  $P info "$src" --json > "$out/info.json"
  $P check "$src" --json > "$out/check.json"
  # The poster: the frame named by a "% Poster: SECONDS" comment, else 60% through.
  at=$(sed -n 's/^% *Poster: *\([0-9.]*\).*/\1/p' "$src" | head -1)
  [ -n "$at" ] || at=$(python3 -c "import json;print(json.load(open('$out/info.json'))['movie']['duration']*0.6)")
  $P frame "$src" --at "$at" -o "$out/poster.png" -q
  # A small still for docs/gallery.md, which is kept in git.
  mkdir -p docs/img/gallery
  ffmpeg -v error -y -i "$out/poster.png" -vf scale=480:-2 -q:v 4 "docs/img/gallery/$name.jpg"
  $P sheet "$src" -o "$out/sheet.png" --count 12 --cols 4 --thumb 400 -q
  $P render "$src" -o "$out/$name.mp4" $scale --json > "$out/render.json"
  # An animated GIF too: 480 wide at 20 fps keeps it a few megabytes.
  $P render "$src" -o "$out/$name.gif" --width 480 --fps 20 -q
  cp "$src" "$out/$name.ps"
  python3 -c "import json;r=json.load(open('$out/render.json'));print('   %d frames %dx%d in %.1fs on %d threads (%.1f fps), %.1f MB' % (r['frames'],r['width'],r['height'],r['total_seconds'],r['threads'],r['frames_per_second'],r['bytes']/1e6))"
done
