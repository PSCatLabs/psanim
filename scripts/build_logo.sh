#!/usr/bin/env bash
# Render every version of the logo from its one source, gallery/logo/logo.ps,
# into docs/img/ (kept in git: the README and GitHub cannot run a build).
#   scripts/build_logo.sh
# Needs ffmpeg. The still versions are frame 0 of the film, the resting mark.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q -p psanim-cli
P=target/release/psanim
src=gallery/logo/logo.ps
out=docs/img
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$out"

$P check "$src" --all -q > /dev/null

# The animated lockup for the README, on a dark and on a light ground. GIF delays are
# whole hundredths of a second, so 25 fps (4/100 s) keeps the 4.4 s loop exact.
for g in dark light; do
  $P render "$src" --param ground=$g --width 912 --fps 25 -o "$tmp/$g/" -q
  name=logo; [ $g = light ] && name=logo-light
  ffmpeg -v error -y -framerate 25 -i "$tmp/$g/frame_%05d.png" \
    -vf "split[a][b];[a]palettegen=max_colors=64:reserve_transparent=0[p];[b][p]paletteuse=dither=none:diff_mode=rectangle" \
    -loop 0 "$out/$name.gif"
  $P frame "$src" --param ground=$g --at 0 -o "$out/$name.png" -q
done

# The mark alone, on nothing.
$P frame "$src" --param form=mark --param ground=transparent --at 0 -o "$out/logo-mark.png" -q
# Favicons: the tile edge to edge at 32 (and as .ico), and an opaque 180 for touch icons.
$P frame "$src" --param form=icon --param ground=transparent --at 0 --width 32 -o "$out/favicon-32.png" -q
ffmpeg -v error -y -i "$out/favicon-32.png" "$out/favicon.ico"
$P frame "$src" --param form=mark --at 0 --width 180 -o "$out/favicon-180.png" -q
# GitHub's social preview (uploaded by hand in the repository's Settings).
$P frame "$src" --param form=social --at 0 -o "$out/social-preview.png" -q

for f in logo.gif logo-light.gif logo.png logo-light.png logo-mark.png favicon-32.png favicon.ico favicon-180.png social-preview.png; do
  printf '  %-20s %7d bytes\n' "$f" "$(wc -c < "$out/$f")"
done
