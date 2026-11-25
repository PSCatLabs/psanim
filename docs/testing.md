# Testing, benchmarking, contributing

## Run everything

```sh
cargo test --release            # all crates: unit, integration, doc and documentation tests
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
scripts/build_wasm.sh && node scripts/wasm_smoke.mjs     # the browser build
```

`--release` matters: the tests render thousands of frames.

## What is tested, and where

| Area | Test | What it pins |
|---|---|---|
| Easing, path math, raster ops, signals, hashing | unit tests beside the code | endpoints, monotonicity, symmetry; arc length; trim proportions; compositing keeps pixels premultiplied; cropped blur equals full blur |
| The engine | `crates/psanim/tests/engine.rs` | frames are time-addressable (any order, fresh movie, same bytes); every time scope; tweens and keyframes; paths; layers; every transition starts on A and ends on B; stateful replay and frame-rate independence; persistent seek; motion blur; errors, hints and backtraces; budgets; file access; interrupt |
| The vocabulary | `tests/vocabulary.rs` | every documented word exists; every public prelude and library name is documented, under its own library and no other; a library loads once and an unloaded library's word says which library it is in; no PSCat library redefines a psanim word |
| The libraries | `tests/libraries.rs` | every entrance is absent before, moving during, and **pixel-identical to the plain drawing** after; every emphasis rests at both ends; a motion scales about the drawing's measured centre; exits are entrances backward; scripts, storyboards and cascades give the clocks they promise; boxes cut up cover their parent and interpolate; `within` re-bases and clips; themes; text is measured, wrapped and fitted, and kinetic text ends as plain text; equations set term by term match the line set whole; connectors run edge to edge; charts map data and grow in proportion; code types, focuses and diffs; a frame using every library still depends on its time alone and its viewport |
| Motions, code, formatting | unit tests in `src/motion.rs`, `src/code.rs`, `src/vocab.rs` | poses at the ends of every built-in motion; tokenized lines round-trip; the diff; number formats; ticks; smooth keys; straightening keeps length |
| Documentation | `tests/docs.rs` | every `postscript` fence in the skill and docs runs as a movie; every example and gallery piece loads and renders |
| Parallel rendering | `src/render.rs` tests | parallel equals serial, in order; errors and sink failures stop the render |
| Contact sheets | `src/sheet.rs` tests | every thumbnail is on the sheet |
| The browser ABI | `crates/psanim-wasm` test, `scripts/wasm_smoke.mjs` | load, render, signals, errors through the C ABI, natively and in Node |
| PSCat's host API | `tests/host_api.rs` in the PSCat branch | see [pscat-integration](pscat-integration.md) |

Determinism is tested on pixels, not assumed: the same frame must be the
same bytes whether rendered first, last, twice, on another `Movie`, or on
another thread.

## Benchmarks

```sh
psanim bench gallery/epicycles/epicycles.ps          # per-frame timings, serial and parallel
scripts/bench_gallery.sh                             # every gallery piece, as a table
```

Results and their conditions are in [benchmarks.md](benchmarks.md).

## Adding a word to the vocabulary

1. Implement it: a native operator in `ops.rs`, or a procedure in
   `prelude.ps` if it must run a body (never `begin` a dictionary around a
   user procedure).
2. Add its entry to `api.rs`. The vocabulary tests fail until it is there,
   and fail if a PSCat library already uses the name.
3. Test it in `tests/engine.rs` (core) or `tests/libraries.rs` (a library word).
4. `scripts/gen_reference.sh` regenerates the skill's `vocabulary.md`.

## Adding a gallery piece

Put `name.ps` and a `README.md` in `gallery/name/`. It must pass `psanim
check` and is picked up by `tests/docs.rs` and `scripts/build_gallery.sh`
automatically. Follow the skill's `craft.md`; look at every sheet.

## Conventions

- The engine crate stays portable: no processes, clocks, or files.
  Platform code goes in `psanim-cli`.
- An error message says what to do. If a mistake is easy to make, add a
  hint for it (`explain` in `engine.rs`) and a pitfall to the skill.
- Do not claim what was not run. Benchmarks say when and on what;
  platform notes say what was exercised.
