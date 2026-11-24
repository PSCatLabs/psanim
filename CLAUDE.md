# psanim: notes for an agent picking this up

Read `docs/HANDOFF.md` first: state, decisions, what is next.

- **What this is**: an animation engine for PostScript programs, built on
  the PSCat interpreter as a Rust library. `README.md` has the map.
- **PSCat lives beside this repo** at `../pscat-psanim-api`, a
  checkout of the branch `feature/psanim-library-api` (draft PR
  PSCatLabs/pscat#1410). Changes to PSCat go there, as separate commits,
  following that repository's `AGENTS.md` and `SDLC.md`. Do not push
  to it, merge it, or touch any other PSCat checkout without being
  asked.
- **Build and test**: `cargo build --release`, `cargo test --release`,
  `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all`.
- **To author or debug a movie**, use the skill in `skills/psanim/`
  (also linked at `.claude/skills/psanim`). Always `psanim check`, then
  look at a `psanim sheet`.
- **One vocabulary table**: `crates/psanim/src/api.rs`. After changing
  it run `scripts/gen_reference.sh`. Tests fail if the table, the
  interpreter and the prelude disagree, or if a PSCat library already
  uses a name.
- **Docs are tested**: every `postscript` fence in `skills/` and `docs/`
  runs as a movie (`crates/psanim/tests/docs.rs`). Mark a fragment that
  cannot run alone as `postscript partial`.
- **Generated, not committed**: `site/` (docs site and rendered films),
  `web/psanim.wasm`, `out/`. Rebuild with `scripts/build_gallery.sh`,
  `scripts/build_wasm.sh`, `scripts/build_site.py`.
- Do not claim a format, platform or number that was not run. The
  benchmarks and platform notes say what was measured and where.
