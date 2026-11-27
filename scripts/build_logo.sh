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

# The animated lockup. GIF delays are whole hundredths of a second, so 25 fps
# (4/100 s) keeps the 4.4 s loop exact. GIF transparency is one bit, which
# leaves the tile's corners and the letters jagged, so each GIF is opaque on
# exactly the ground it is shown on:
#   logo.gif        the README in GitHub's dark theme (#0d1117)
#   logo-light.gif  the README in GitHub's light theme (white)
#   logo-site.gif   the site's home page (its ground, #0b0d12)
gif() { # ground name
  $P render "$src" --param ground=$1 --width 912 --fps 25 -o "$tmp/$1/" -q
  ffmpeg -v error -y -framerate 25 -i "$tmp/$1/frame_%05d.png" \
    -vf "split[a][b];[a]palettegen=max_colors=64:reserve_transparent=0[p];[b][p]paletteuse=dither=none:diff_mode=rectangle" \
    -loop 0 "$out/$2.gif"
}
gif github logo
gif light logo-light
gif dark logo-site

# Stills of the lockup, for anyone who needs the logo as a picture (not used by
# the README or the site): on the site's dark ground and on white.
$P frame "$src" --at 0 -o "$out/logo.png" -q
$P frame "$src" --param ground=light --at 0 -o "$out/logo-light.png" -q

# The mark alone, on nothing.
$P frame "$src" --param form=mark --param ground=transparent --at 0 -o "$out/logo-mark.png" -q
# Favicons: the tile edge to edge at 32 (and as .ico), and an opaque 180 for touch icons.
$P frame "$src" --param form=icon --param ground=transparent --at 0 --width 32 -o "$out/favicon-32.png" -q
ffmpeg -v error -y -i "$out/favicon-32.png" "$out/favicon.ico"
$P frame "$src" --param form=mark --at 0 --width 180 -o "$out/favicon-180.png" -q
# GitHub's social preview (uploaded by hand in the repository's Settings).
$P frame "$src" --param form=social --at 0 -o "$out/social-preview.png" -q

for f in logo.gif logo-light.gif logo-site.gif logo.png logo-light.png logo-mark.png favicon-32.png favicon.ico favicon-180.png social-preview.png; do
  printf '  %-20s %7d bytes\n' "$f" "$(wc -c < "$out/$f")"
done
