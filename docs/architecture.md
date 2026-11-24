# Architecture and design rationale

psanim renders animations that are written as PostScript programs. This
document says how it is built, why, and what was considered and rejected.
For how to *write* an animation see the [authoring guide](authoring.md).

## The shape of the system

```
                    ┌──────────────────────────── psanim (this repository) ───────────────────────────┐
  movie.ps ───────▶ │  psanim (library)                                                               │
                    │    Movie ── load: probe declaration ▸ build interpreter ▸ run setup ▸ save      │
  options ────────▶ │          ── frame(t): restore ▸ set clock ▸ call scene ▸ read canvas            │
  (scale, fps,      │    ops.rs      native operators: clock, time scopes, tweens, paths, layers      │
   params,          │    prelude.ps  PostScript half: combinators, reels, text helpers                │
   signals,         │    raster / pathmath / ease / signals   pure building blocks                    │
   budget)          │    render      frame ranges on worker threads      sheet   contact sheets       │
                    │                                                                                 │
                    │  psanim-cli  info check frame sheet render bench api ── ffmpeg (a subprocess)   │
                    │  psanim-wasm C ABI for the browser ── web/psanim.js ── <psanim-player>          │
                    └───────────────────────────────────────┬─────────────────────────────────────────┘
                                                            │ Rust API, in process
                    ┌───────────────────────────────────────▼──────────────────────── pscat ──────────┐
                    │  Interp (PostScript machine, VM save/restore)   Gfx (paths, paint, tiny-skia)   │
                    │  lib/*.ps (artkit, paintkit, fonts, ...) compiled in      Limits, read roots    │
                    │  pscat::host   register_operator · Value · current_path · swap_canvas ·         │
                    │                error_details + backtrace · set_interrupt        (new, for this) │
                    └─────────────────────────────────────────────────────────────────────────────────┘
```

Three crates:

| Crate | What it is | Depends on |
|---|---|---|
| `psanim` | The engine. Portable: no processes, no clock, no files of its own. | `pscat`, `serde_json` |
| `psanim-cli` | The `psanim` command. The only place video encoding lives. | `psanim`, an `ffmpeg` on PATH for video |
| `psanim-wasm` | A C ABI over `Movie` for `wasm32-unknown-unknown`. | `psanim` |

## The animation model

### A frame is a function of time

A movie is a PostScript program. Its top level runs once (**setup**); it
must define a procedure, `scene`, which the engine calls once per frame.
The scene reads the time with `now` and draws.

That is the INIT brief's starting sketch, kept, because it is the right
primitive: it needs no retained scene graph, it leaves the whole of
PostScript available, and it makes every frame addressable. What psanim
adds is the guarantee that the function is *pure*, and a vocabulary for
writing it.

### Purity by rollback, using PostScript's own checkpoint

A scene is arbitrary code. It can `def`, fill arrays, leave dictionaries
open. If frame 300 could see what frame 299 did, frames would only exist
in order.

PostScript already has the mechanism: `save` checkpoints the virtual
memory and `restore` rolls every array and dictionary back to it. So:

1. After setup the engine runs `save` and keeps the save object (in Rust,
   through `pscat::host::Value`, not in a PostScript name that a rollback
   would itself undo).
2. Before every frame it runs `restore save`: back to the checkpoint, and
   a fresh checkpoint for next time.
3. Then it resets the graphics state, erases, reseeds `rand`, sets the
   clock, and calls `scene`.

The cost was measured before the design was committed to: PSCat's
`save`/`restore` is a copy-on-write journal, and a frame with the
rollback costs the same as one without (0.9 ms against 1.2 ms for a small
scene at 1280x720; the difference is noise). Interpreter start-up is
1.4 ms and loading artkit 4 ms, so workers that each load their own copy
are cheap too.

*Rejected:* re-running the whole program per frame (setup cost on every
frame, and no way to precompute); a fresh interpreter per frame (same);
trusting scenes to be pure (one stray `def` makes frames order-dependent,
and nothing would say so).

### Time is graphics state

PostScript has a current transformation matrix: `translate` changes what
every later coordinate means, and `gsave`/`grestore` scope it. psanim
gives time the same treatment. There is a **current time scope**: a local
clock and a window. `now` and `progress` read it. Combinators run a
procedure under a changed scope and put it back:

| Space | Time |
|---|---|
| `x y translate` | `d { } delayed` |
| `k k scale` | `k { } timescaled` |
| a clip path | `a b { } during` |
| a tiling | `p { } looped` |

This is the central idea of the model. A procedure written against `now`
is a reusable piece of motion: it can be placed, delayed, staggered,
looped, reversed or slowed by its caller without being edited, exactly as
a procedure that draws at the origin can be placed by a `translate`.
Sequencing (`sequence`, `reel`) is built from the same scopes, so there
is no separate timeline structure to keep consistent with the code.

*Rejected:* a retained scene graph with animated properties (Manim,
Motion Canvas). It makes simple things declarative, but it is a second
language beside PostScript, it needs object identity across frames, and
it cannot express a procedural scene whose structure depends on time
without escape hatches. Generators with `yield` (Motion Canvas) order
time imperatively, which makes random access to a frame require replay.
Shader-style `f(uv, t)` has the right purity but no drawing model.

### What is native and what is PostScript

The rule: **Rust owns state and arithmetic; PostScript owns anything that
runs a procedure.** A native operator cannot call back into the
interpreter, and should not need to.

- Native (`ops.rs`): the clock and scope stack, easing curves, keyframe
  evaluation, hashing and noise, arc-length path operations (`trimpath`,
  `pathpoint`, `resamplepath`, `morphpath`), layers and pixel effects.
- PostScript (`prelude.ps`): the combinators themselves. `during` is
  `.during { exec .timepop } if`: the native half decides and pushes the
  scope, the PostScript half runs the body.
- The prelude never opens a dictionary around a user procedure, so a
  body's `def` lands where its author expects, and every prelude
  procedure is `bind`-ed.

The vocabulary is one table (`api.rs`). `psanim api`, the reference in
the docs and the agent skill are generated from it, and tests check it
against the interpreter in both directions, and against every PSCat
library for name clashes (which is how `grain` became `filmgrain`).

### Movies that are not pure

Some pictures are history: a simulation, a painting that accumulates.
They are explicit modes, not accidents.

**Stateful.** If the program defines `step`, the engine calls it at a
fixed tick rate, and what it changes persists. Frame *t* shows the state
after `floor(t × rate)` ticks; the scene still draws between a `save` and
a `restore` of its own, so drawing cannot disturb the simulation. Ticks
are fixed, so the result does not depend on the output frame rate.

Seeking uses the same mechanism as purity, nested: every two seconds of
movie the engine takes another `save` *inside* the previous ones. To go
back, it restores the newest checkpoint at or before the target (which
invalidates the later ones, as PostScript defines) and replays ticks from
there. `rand` is seeded per tick, so a replay is exact.

**Persistent.** `/Persist true` keeps the canvas between frames. Each
tick paints one frame of the declared rate over the last; canvas
checkpoints are kept beside the VM checkpoints for seeking.

Both render on one thread, in order, and say so (`psanim info` reports
the mode).

### Time, frames and seeds, exactly

- Frame `k` shows time `k / fps`. A movie of duration `D` has
  `ceil(D × fps)` frames; the last frame is strictly before `D`, so a
  loop's last frame leads into its first.
- `render_at(t)` renders any `t`, not only frame times: the browser
  player drives a pure movie from the display's clock.
- `rand` is reseeded with the movie seed before every frame (so a random
  layout holds still) and with a per-tick seed inside `step`.
  `hash01` and `wiggle` are stateless functions of their key, the seed,
  and for `wiggle` the time.
- `realtime` and `usertime` read the movie clock, not the wall clock.
- Motion blur renders `n` moments across the shutter interval and
  averages them.

### External inputs

A **signal** is a number that varies with time and is not computed by the
movie; an **event track** is a list of moments. Programs read them with
`signal`, `sinceevent`, `eventcount`. Offline they come from a CSV file,
so a render is reproducible; a live host overwrites them before each
frame (`Movie::set_signal`), which is what the browser player does with
the pointer. The program cannot tell which host it has. Nothing in the
engine assumes a fixed offline frame loop: a host asks for a time and
gets a frame.

## Rendering and export

- **Frames** are premultiplied RGBA borrowed from the interpreter's
  canvas (`Movie::render_frame` returns a `Frame<'_>`; no copy until the
  caller makes one).
- **Layers** swap the interpreter's canvas for a transparent page
  (`pscat`'s `swap_canvas`) and composite in `raster.rs`: opacity, blend
  modes, a three-pass box blur.
- **Parallelism** (`render.rs`): an interpreter is not `Send`, so each
  worker thread loads its own `Movie` and renders the frames it pulls
  from a shared counter; the calling thread reorders them for the sink.
  This is correct *because* frames are pure. A window bounds how far
  workers run ahead of the sink.
- **Encoding** is outside the engine. The CLI pipes opaque RGBA frames to
  `ffmpeg` (H.264 or VP9, yuv420p; GIF with a generated palette) or
  writes a PNG sequence with PSCat's own encoder.
- **GPU**: not used. PSCat's rasterizer is CPU (tiny-skia); its optional
  GPU path covers compiled per-pixel kernels only, which animation scenes
  rarely are. Frame parallelism already scales with cores. See
  [benchmarks](benchmarks.md).

## Errors and diagnostics

An error says where in the *movie* it happened (setup, a frame, a tick),
what the offending command was, where in the program's own procedures
(the backtrace from PSCat's `error_details`), what was on the operand
stack, and when psanim can tell, what to do. `psanim check` then looks at
pixels so an agent does not have to: blank frames, no motion, abrupt
changes, a loop that does not close, the heaviest frame.

## Safety

Generated PostScript is treated as untrusted when the host says so. See
[security](security.md) for what is and is not guaranteed.

## Decisions, in one table

| Decision | Instead of | Because |
|---|---|---|
| PSCat as a library, in process | invoking `pscat` per frame | 1 ms frames; shared setup; host operators |
| `save`/`restore` per frame | trusting scenes, or re-running setup | purity that cannot be broken by accident, at no measured cost |
| Time scopes as state | a timeline data structure | composes like the CTM; motion is reusable procedure |
| Native state, PostScript control | all-native or all-PostScript | natives cannot call procedures; PostScript arithmetic is slow where it is hot |
| Declaration probed by a first pass | a header comment, or fixed size | the canvas must exist before setup; the program stays plain PostScript |
| One vocabulary table | hand-written docs | reference, CLI and skill cannot drift |
| Workers each load the program | sharing an interpreter | `Interp` is `Rc`-based by design; loading is milliseconds |
| ffmpeg as a subprocess of the CLI | a Rust encoder in the engine | keeps the engine portable and its dependency tree small |
| C ABI for wasm | wasm-bindgen | no toolchain beyond `rustup target add`; matches PSCat's own browser build |
