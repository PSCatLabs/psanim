# psanim and PSCat

psanim is a consumer of PSCat's Rust library. It never runs the `pscat`
binary. This page records which PSCat it is built against, what it needed
PSCat to grow, and how the two repositories are built together.

## What is tested together

| | |
|---|---|
| PSCat repository | https://github.com/PSCatLabs/pscat |
| Branch | `feature/0.8.0`, PSCat's development branch for 0.8.0 (not released) |
| Pinned revision | `c0a5636`, recorded in `Cargo.lock` |
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
| know whether a transparency group is open, and which | `Interp::open_group_count`, `innermost_group_serial` (0.8.0, experimental) | yes (PSCatLabs/pscat#1494) |

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
- **A host query for open transparency groups.** Done in 0.8.0
  (PSCatLabs/pscat#1494): `open_group_count` and
  `innermost_group_serial`. While a group is open `swap_canvas`
  exchanges the page below the group's layer, so a host that draws layers
  by swapping must not do so inside one.
- **The gsave depth, and whether a capture is open** (PSCatLabs/pscat#1497):
  psanim marks gsave levels with a flatness sentinel to put a body's
  levels back across `endgroup`.
- **Virtual files**: a host serving `(name) run` from memory.
- **A host `image`**: painting host pixels under the current CTM.
- **Source positions for procedure bodies**, which would turn the
  backtrace's excerpts into line numbers.
- **Dropping an `Interp` during thread-local teardown panics** (the name
  interner's thread-local can be destroyed first). Observed in a test of
  `psanim-wasm`; hosts should drop interpreters explicitly.

## Transparency groups and psanim's layers

What PSCat does with a group open, as a host sees it (PSCat's
`docs/guides/library.md`; the parts psanim relies on are checked by
`crates/psanim/tests/groups.rs`): the program paints on the group's
layer; `canvas_premultiplied_rgba8`, `pages`, `page`, `into_last_page`
and `swap_canvas` all act on the page below that layer, and the group
composites onto whatever page is there at `endgroup`; the end of a run
(and an uncaught error) closes open groups with their marks kept; an
interrupt drops them, leaving the page whole; `swap_canvas` inside a
pattern's `PaintProc` hands the page back.

So a psanim layer opened inside a group is swapped in *below* the group
and never painted on: its effect is lost, and the group composites onto
the layer page instead of the frame. `faded` and `blurred` inside a group
draw at full strength and sharp; `vignette` and `filmgrain` turn the frame
white (the clip mask they paint lands in the group). A group *inside* a
psanim layer or motion is fine.

What psanim does about it:

- `begingroup` and `endgroup` are PSCat's operators, untouched. (A first
  attempt counted groups by wrapping them in the prelude. It could not be
  made safe: an `endgroup` in a pattern's `PaintProc` closes nothing at
  the page's level, and `//begingroup` must stay an operator.)
- Every question about groups goes through `crates/psanim/src/groups.rs`:
  `groups_open` (how many are open at the page's level) and
  `innermost_group` (which one is innermost). They are asked before a
  layer or pixel effect opens, at the end of a layer's, motion's or
  grouped fade's body, and at the end of a frame's run (`.framedone`,
  before PSCat would close a group left open).
- With an answer, a layer or pixel effect refuses to open inside a group
  (`groupnesting`, naming the word); `faded` and `blended` (`/normal`,
  `/multiply`) run as a group of their own (`.groupfaded` in the
  prelude) by a layer's rules: the caller's alpha and blend mode go to
  the marks inside, the graphics state the body leaves outlasts it, gsave
  levels included (taken off before `endgroup` and put back after), and a
  body that `grestore`s past its start is refused; a body must leave its
  fade's own group innermost, checked by identity; a frame must close
  its groups (`unclosedgroup`).
- What PSCat counts: groups at the page's level, opened by any route
  (`systemdict`, `//begingroup`, bound). A group is closed by `endgroup`,
  `erasepage`, `showpage`, `copypage`, a `restore` past it, the end of a
  run, an uncaught error, a limit or the interrupt flag. An error caught
  with `stopped` does not close one. Inside a pattern's `PaintProc` both
  queries report the page's groups. A group opened under `nulldevice` is
  counted though it has no layer.
- Tested (`crates/psanim/tests/groups.rs`): each refusal and conversion
  above, the pattern and `systemdict` cases, the next frame after an
  error or interrupt with a group open (byte-identical to a fresh
  movie's, all three modes), groups inside layers and motions, and
  parallel renders.

What psanim needs from PSCat, as host API on `Interp`:

- `fn open_group_count(&self) -> usize`: the transparency groups open at
  the page's level (those whose layer stands between the program's
  painting and the page `swap_canvas` exchanges), counted as the
  interpreter does: any `begingroup` counts, however it was called, and
  a group ended by `endgroup`, `erasepage`, `showpage`, `copypage`, a
  `restore` or an error does not. An `endgroup` inside a pattern's
  `PaintProc` does not change it unless it closes a page-level group
  (which PSCat says it cannot). Callable from inside a host operator.
- `fn innermost_group_id(&self) -> Option<u64>`: an identifier of the
  innermost page-level open group, unique among the groups the
  interpreter has opened (an opened-ever counter at its `begingroup`
  would do), `None` when none is open. The same group gives the same
  value while it stays open, and a group opened later never reuses it.
