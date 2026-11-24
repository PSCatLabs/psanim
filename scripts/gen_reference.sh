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
