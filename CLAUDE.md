# psanim: notes for an agent picking this up

Read `docs/HANDOFF.md` first: state, decisions, what is next.

- **What this is**: an animation engine for PostScript programs, built on
  the PSCat interpreter as a Rust library. `README.md` has the map.
- **PSCat is a git dependency**: `feature/0.8.0` of PSCatLabs/pscat,
  pinned by `Cargo.lock` (`docs/pscat-integration.md` says how to move
  the pin or build against a local checkout). A change psanim needs in
  PSCat is an issue there (label `psanim`), not an edit from here. Do
  not push to PSCat, merge there, or touch a PSCat checkout without
  being asked.
- **Build and test**: `cargo build --release`, `cargo test --release`,
  `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all`.
- **To author or debug a movie**, use the skill in `skills/psanim/`
  (also linked at `.claude/skills/psanim`). Always `psanim check`, then
  look at a `psanim sheet`.
- **One vocabulary table**: `crates/psanim/src/api.rs`, for the core and
  for the libraries. After changing it run `scripts/gen_reference.sh`.
  Tests fail if the table, the interpreter, the prelude and the
  libraries disagree, or if a PSCat library already uses a name.
- **The pk libraries** are PostScript in `crates/psanim/lib/`, compiled
  into the binary and loaded by a movie with `/pkmotion use`; their
  native halves are in `src/vocab.rs`, `src/motion.rs`, `src/code.rs`.
  `docs/vocabulary-design.md` says why they are shaped as they are.
  Library words that run a caller's procedure must not leave a
  dictionary open around it.
- **Fresh-agent runs** need a binary that does not change under them:
  copy `target/release/psanim` to `out/bin/psanim` and point them there.
- **Docs are tested**: every `postscript` fence in `skills/` and `docs/`
  runs as a movie (`crates/psanim/tests/docs.rs`). Mark a fragment that
  cannot run alone as `postscript partial`.
- **Generated, not committed**: `site/` (docs site and rendered films),
  `web/psanim.wasm`, `out/`. Rebuild with `scripts/build_gallery.sh`,
  `scripts/build_wasm.sh`, `scripts/build_site.py`.
- Do not claim a format, platform or number that was not run. The
  benchmarks and platform notes say what was measured and where.
