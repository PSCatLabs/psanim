# psanim and PSCat

psanim is a consumer of PSCat's Rust library. It never runs the `pscat`
binary. This page records which PSCat it is built against, what it needed
PSCat to grow, and how the two repositories are built together.

## What is tested together

| | |
|---|---|
| PSCat repository | https://github.com/PSCatLabs/pscat |
| Upstream base | `feature/0.7.1` at `72a2c4e` (the active development branch; `main` was 64 commits behind it on 2026-11-23) |
| Feature branch | `feature/psanim-library-api` (local only: not pushed, not merged) |
| Tested revision | `b1c1f8a` |
| PSCat version | 0.7.0-rc.5 |
| Local checkout | a git worktree at `../pscat-psanim-api`, beside this repository |

The existing checkout (`pscat_gpu`, on `main`) was left untouched; the
feature branch lives in its own worktree, with its Cargo `target/` linked
by PSCat's `scripts/worktree_target.sh` as that repository's conventions
require.

## Building both together

```sh
# once: the PSCat feature branch as a sibling worktree
git -C /path/to/pscat worktree add -b feature/psanim-library-api ../pscat-psanim-api origin/feature/0.7.1
#   (or, from a clone that already has the branch: git worktree add ../pscat-psanim-api feature/psanim-library-api)

# then, in psanim
cargo build --release          # builds pscat from ../pscat-psanim-api as a path dependency
cargo test  --release
```

The dependency is declared once, in the workspace `Cargo.toml`:

```toml
[workspace.dependencies]
pscat = { path = "../pscat-psanim-api", default-features = false }
```

`default-features = false` leaves out PSCat's window (winit, softbuffer):
psanim uses only the headless library.


## What psanim uses from PSCat

Only the supported, documented API (`docs.rs` surface), nothing hidden:

| psanim needs to | PSCat API | New on the branch |
|---|---|---|
| build an interpreter of the movie's size | `Interp::with_page_scaled` | |
| run setup, the prelude, each frame | `Interp::run_str` | |
| bound untrusted programs | `Limits`, `Interp::set_limits`, `steps_used`, `set_read_roots`, `TrackingAllocator` | |
| add `now`, `tween`, `trimpath`, layers, ... (about 70 operators) | `Interp::register_operator` | yes |
| read the `animation` dictionary; hold `save` objects | `host::Value`, `pop_value`, `push_value`, `lookup` | yes |
| take typed operands, leave results | `pop_f64`, `pop_int`, `pop_name`, `pop_numbers`, `push_real`, ... | yes (documented; `pop_f64`/`pop_int` existed hidden) |
| measure, walk, trim and morph paths | `Interp::current_path`, `set_current_path`, `host::PathSegment` | yes |
| read a frame without copying | `Interp::canvas_premultiplied_rgba8`, `canvas_size` | yes |
| draw on layers and composite them; keep a canvas across frames | `Interp::swap_canvas`, `Page::transparent`, `premultiplied_rgba8_mut`, `from_premultiplied_rgba8` | yes |
| say where in the program a frame failed | `Interp::error_details`, `host::ErrorDetails::backtrace` | yes |
| cancel a render | `Interp::set_interrupt`, `PsError::Interrupted` | yes |
| encode PNGs, lay out contact sheets | `Page::to_png`, `pscat::sheet` | |
| roll frames back | the PostScript operators `save` / `restore` | |

## The PSCat changes

Three commits on `feature/psanim-library-api`, each reviewable alone:

1. **Host integration API**: `pscat::host` (operators from closures,
   `Value`, the current path as data), canvas access and swap on
   `Interp`/`Page`, `error_details`, `set_interrupt`.
2. **Error backtrace**: the procedures that were running and the element
   in each, including a procedure that failed at its last element.
3. **Docs**: a section in `docs/guides/library.md` and a design note,
   `notes/psanim-library-api.md` (decisions, compatibility, what was left
   out).

They are general: nothing in them mentions animation, time or psanim's
types, and PSCat gains no dependency. `tests/host_api.rs` exercises them
as any embedder would. PSCat's full suite passes with them (3688 tests,
0 failures, `cargo test --profile ci`), clippy and rustdoc are clean, and
`benches/perf` is unchanged (fib 27: 88-90 ms before, 89-90 ms after).

Compatibility: additive only. `PsError` gains the `Interrupted` variant
(the enum is `#[non_exhaustive]`).

## Moving to a released PSCat

When the branch is merged and a PSCat release contains it (0.7.1 or
later), psanim's one dependency line becomes

```toml
pscat = { version = "0.7.1", default-features = false }
```

and nothing else changes: psanim uses no hidden module and no path into
PSCat's source tree. Until then the Rust crate API is, by PSCat's own
`docs/STABILITY.md`, experimental, so psanim pins the revision above.

## What PSCat could still grow

Found while building psanim, not needed to ship it:

- **Group opacity and blend as PostScript operators.** psanim implements
  layers in the host with `swap_canvas`. A PSCat-level transparency group
  would let SVG and PDF output carry them too.
- **Virtual files**: a host serving `(name) run` from memory.
- **A host `image`**: painting host pixels under the current CTM.
- **Source positions for procedure bodies**, which would turn the
  backtrace's excerpts into line numbers.
- **Dropping an `Interp` during thread-local teardown panics** (the name
  interner's thread-local can be destroyed first). Observed in a test of
  `psanim-wasm`; hosts should drop interpreters explicitly.
