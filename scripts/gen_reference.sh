#!/usr/bin/env bash
# Regenerate the vocabulary reference from the table in crates/psanim/src/api.rs.
#   scripts/gen_reference.sh
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q -p psanim-cli
P=target/release/psanim
out=skills/psanim/references/vocabulary.md
{
  printf '# psanim vocabulary\n\n'
  printf 'Generated from `crates/psanim/src/api.rs` by `scripts/gen_reference.sh`. Do not edit by hand.\n\n'
  printf 'Stack effects read left to right, topmost operand last, as in the PostScript Language Reference: `a b /name  tween  u` takes three operands and leaves one.\n\n'
  printf 'Easings (for `ease`, `tween`, `keyframes`): %s\n\n' "$($P api | sed -n 's/^Easings: //p')"
  printf 'Transitions (for `reel`): %s\n\n' "$($P api | sed -n 's/^Transitions: //p')"
  $P api --markdown
} > "$out"
echo "wrote $out ($(grep -c '^### ' "$out") words)"

libs=skills/psanim/references/vocabulary-libraries.md
{
  printf '# psanim library vocabulary\n\n'
  printf 'Generated from `crates/psanim/src/api.rs` by `scripts/gen_reference.sh`. Do not edit by hand.\n\n'
  printf 'The words of the libraries loaded with `use`. The chapters (stage.md, type.md, explain.md, ink.md) show them at work; this is the list. `psanim api NAME` prints one entry.\n\n'
  printf 'Motions (for `enters`, `leaves`, `onstage`, `kinetic` ...): %s\n\n' "$($P api pkmotion | sed -n 's/^Entrances: //p')"
  printf 'Emphases (for `emphasized`): %s\n\n' "$($P api pkmotion | sed -n 's/^Emphases: //p')"
  for lib in $($P api --libs | sed -n 's|^  /\([a-z]*\) use.*|\1|p'); do
    $P api "$lib" --markdown
  done
} > "$libs"
echo "wrote $libs ($(grep -c '^### ' "$libs") words)"
