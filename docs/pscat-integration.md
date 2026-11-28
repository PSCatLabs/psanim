# psanim and PSCat

psanim is a consumer of PSCat's Rust library. It never runs the `pscat`
binary. This page records which PSCat it is built against, what it needed
PSCat to grow, and how the two repositories are built together.

## What is tested together

| | |
|---|---|
| PSCat repository | https://github.com/PSCatLabs/pscat |
| Branch | `feature/0.8.0`, PSCat's development branch for 0.8.0 (not released) |
| Pinned revision | `d385971`, recorded in `Cargo.lock` |
| PSCat version | 0.7.1 (the branch has not bumped it yet) |

The host API psanim needs was developed on `feature/psanim-library-api`
(PR [#1410](https://github.com/PSCatLabs/pscat/pull/1410), issue
[#1403](https://github.com/PSCatLabs/pscat/issues/1403)) and merged into
0.7.1; `feature/0.8.0` carries it forward.

## Building both together

Nothing to check out: Cargo fetches PSCat.

```sh
cargo build --release
cargo test  --release
```

The dependency is declared once, in the workspace `Cargo.toml`:

```toml
[workspace.dependencies]
pscat = { git = "https://github.com/PSCatLabs/pscat", branch = "feature/0.8.0", default-features = false }
```

`Cargo.lock` pins the revision, so a build is reproducible. To move to
the branch's current head, run `cargo update pscat`, then the tests, and
update the table above.

To build against a local PSCat checkout (to try a change in both at
once) without editing the manifest:

```sh
cargo test --release --config 'paths=["/path/to/pscat"]'
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
| know whether a transparency group is open | the PostScript operators `begingroup` / `endgroup`, wrapped by the prelude | |

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

- **Group opacity and blend as PostScript operators.** Done in 0.8.0:
  `begingroup` / `endgroup`. See below for how psanim lives with them.
- **A host query for open transparency groups.** While a group is open
  `swap_canvas` exchanges the page below the group's layer, so a host
  that draws layers by swapping must not do so inside one; but nothing
  in the host API says whether one is open. psanim counts them itself
  (below).

## Transparency groups and psanim's layers

What PSCat does with a group open, as a host sees it (PSCat's
`docs/guides/library.md`, and checked by `crates/psanim/tests/groups.rs`):
the program paints on the group's layer; `canvas_premultiplied_rgba8`,
`pages`, `page`, `into_last_page` and `swap_canvas` all act on the page
below that layer, and the group composites onto whatever page is there
at `endgroup`; the end of a run (and an uncaught error) closes open
groups with their marks kept; an interrupt drops them, leaving the page
whole; `swap_canvas` inside a pattern's `PaintProc` hands the page back.

So a psanim layer opened inside a group would be swapped in *below* the
group and never painted on: its effect is lost, and the group composites
onto the layer page instead of the frame. Before #109 that happened
silently (`faded` and `blurred` inside a group drew at full strength and
sharp; `vignette` and `filmgrain` turned the frame white, the clip mask
they paint landing in the group).

psanim's answer, all in psanim:

- the prelude's `begingroup` / `endgroup` run PSCat's and tell the engine,
  which keeps one stack of open layers and groups (`Ctx::nest`);
- a layer or pixel effect refuses to open while a group is innermost
  (`groupnesting`, naming the word); `faded` and `blended` (`/normal`,
  `/multiply`) run as a group of their own there instead, keeping a
  layer's rules: the caller's alpha and blend mode go to the marks
  inside (a group would start them at 1 and `/Normal` and composite at
  the caller's alpha), and the graphics state the body leaves outlasts
  it (`gstate currentgstate` before `endgroup`, `setgstate` after);
- a layer's body must close the groups it opens and no others
  (`groupnesting`); a frame must close its groups (`unclosedgroup`,
  beside `unclosedlayer`);
- the stack is cleared at the start of every frame, so a frame that
  failed or was interrupted with a group open leaves nothing behind (the
  next frame is byte-identical to a fresh movie's, tested in all three
  modes).

The limit: psanim sees only groups opened and closed through those two
names, so one that `erasepage`, `showpage` or a `restore` ends is still
counted open (the frame then fails with `unclosedgroup`), and one opened
by `systemdict /begingroup get exec` is not counted at all. A host query
in PSCat would remove both.
- **Virtual files**: a host serving `(name) run` from memory.
- **A host `image`**: painting host pixels under the current CTM.
- **Source positions for procedure bodies**, which would turn the
  backtrace's excerpts into line numbers.
- **Dropping an `Interp` during thread-local teardown panics** (the name
  interner's thread-local can be destroyed first). Observed in a test of
  `psanim-wasm`; hosts should drop interpreters explicitly.
