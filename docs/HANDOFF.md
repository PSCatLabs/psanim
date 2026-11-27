# Handoff: where psanim stands

Written 2026-11-23 at the end of the first build; updated 2026-11-24 at
the end of the second, which added the expressive libraries, and
2026-11-25 at the end of the third (milestone 0.2: components, the
catalog, and the pieces built from them). For the next person or agent.

## State

Working and tested on macOS (Apple silicon):

- `crates/psanim`: engine. Pure, stateful (`step`) and persistent
  (`/Persist`, with optional `overlay`) movies; motion blur; params,
  signals, events; budgets; structured errors with PostScript backtraces.
- `crates/psanim-cli`: `info check frame sheet render bench api`.
  MP4, WebM, GIF, PNG sequence exercised.
- `crates/psanim-wasm` + `web/`: browser engine, `<psanim-player>`,
  playground. Exercised in Node and Chromium.
- **The pk libraries** (second build): `crates/psanim/lib/pk*.ps`, seven
  libraries in PostScript loaded with `/name use`, and their native
  halves in `src/vocab.rs` (operators), `src/motion.rs` (the motion
  table and poses), `src/code.rs` (tokenizer, diff), plus
  `raster::composite_placed`. About 140 new words; one reference table
  for all of it. `tests/libraries.rs` (31 tests) checks what the words
  promise and that they compose. Read `docs/vocabulary-design.md` before
  changing them and `docs/vocabulary-evaluation.md` for what agents did
  with them.
- `gallery/`: thirty-five pieces. `skills/psanim/`: the agent skill, now
  with a chapter per kind of film (`stage`, `type`, `explain`, `ink`,
  `storytelling`). `docs/` + `scripts/build_site.py`: Markdown docs and
  the HTML site, which renders every library example as a looping film.
- **Components and the catalog** (third build, epic #43): `component`,
  `instance`, `place` make any movie a part of another, each instance
  with its own names, clock, seed, parameters and box (blur, glow,
  grain and vignette stay inside it; operators and library words mean
  what it was written against). `crates/psanim/catalog/*.ps` holds 26
  ready-made entries compiled in (`psanim catalog`): spinner, progress,
  testdots, pipeline, terminal, fire, fireworks, confetti, clocks,
  credits, crawl, chat, twiddle, splitflap, odometer, ledmatrix,
  dotmatrix, stat, gauge, horizon, timeseries, status, counter. New core
  words: particles, onplane (perspective), playlist, typing and session
  plans, time of day, signals with a past. New library: pkfigure
  (character rigs). `<psanim-widget>` runs a movie live in a page
  (`docs/widgets.md`). Each piece of this was built on its own branch and
  merged in `integration-0.2`; the merge commits say what met what.
- PSCat: a git dependency on `feature/0.8.0`, pinned at `d385971` by
  `Cargo.lock`. The host API (PSCatLabs/pscat#1410) is merged there.
  psanim uses none of 0.8.0's additions yet; PSCatLabs/pscat#1488 lists
  them (line numbers in errors, the UTF-8 check, Unicode text, pixel
  fonts).

## Decisions worth knowing before changing things

Full reasoning in `docs/architecture.md`.

1. **Frames are pure by rollback.** Setup once, `save`, and every frame
   starts with `restore save`. Do not add state that survives a frame
   outside the explicit modes.
2. **Time scopes are host state**, a stack in `Ctx`; combinators are
   `native-decide + PostScript-exec` pairs because a native operator
   cannot call a procedure. The prelude never `begin`s a dictionary
   around a user procedure.
3. **`scene` and `step` run on an empty operand stack and must leave it
   empty.** This replaced "time is passed as an operand" after an agent's
   stack underflow silently consumed the engine's value.
4. **Layer combinators take their operands before running the body**
   (`.fadebegin` and friends), so a body sees its caller's stack.
5. **The declaration is found by a probe pass**: the program runs in a
   throwaway interpreter until `animation`, which `quit`s. Code before
   `animation` therefore runs twice; keep it to definitions. The probe's
   printing is dropped (the real pass prints once), and `check` warns when
   that code prints or takes over 50 ms (`Meta::preamble_warning`; the
   host passes `Options::clock` because the engine keeps no clock). Not
   removed: letting `animation` size the canvas in place would also have
   to rebuild the interpreter's scale, seed and the `width` / `height`
   the preamble sees, which is a larger change than the warning.
6. **`backdrop` respects the CTM and clip** so reels can wipe and push
   clips that start with one.
7. **PSCat changes are general.** Nothing in `pscat::host` knows about
   animation. Keep it that way. (The second build needed no PSCat
   changes at all.)
8. **No scene graph, still.** Entrances and exits work on any drawing
   because the engine draws the body on a layer, measures what was
   painted and places that under a pose (`.motionbegin` / `.motionend`).
   A thing at rest is drawn directly: a test asserts pixel equality for
   every motion. Do not add objects with bounds to get a feature this
   mechanism can give.
9. **A box is `[x y w h]`, a plain array**, so `between` and `keyframes`
   already animate boxes and cameras. Keep it an array.
10. **No library word owns a clock.** Animated quantities (`/Draw`,
    `/Grow`, `/Reveal`, `u marker`) are numbers the caller computes, so
    every word works under any time scope. `width`, `height` and `vmin`
    come from a viewport stack (`within`), so every word works in any
    box.
11. **Libraries are opt-in** (`use`), to keep a movie's namespace small;
    an unloaded library's word fails with a hint naming the library.
12. **A time scope has a window and a cycle.** `progress` and `span` use
    the window; `turns` and `loopnoise` use the cycle, which equals the
    window except under `delayed` (and `staggered`, `cascade`, `echoed`),
    a pure shift that shortens the window but leaves the cycle, so a loop
    closes in a late copy too. `after` and `enters` restart the clock and
    keep the shortened window as the cycle.
13. **Blur and glow stay inside the clip they began under** (#66).
    `.clipmask` in the prelude paints through the interpreter's own clip,
    in device space, on a scratch layer, over the newest clip's box
    (`clippath`), and the engine keeps the coverage; `.layerend`,
    `blurlayer` and `glowlayer` confine what they spread to it. That is
    exact for any clip (rotated, curved, even-odd, nested, anti-aliased)
    with no PSCat change. `faded` and `blended` take none: their body was
    drawn under the clip already. It is the clip in force when the effect
    begins; one the body sets and leaves does not confine it. Cost at
    1280x720 on an Apple M4, median frame, one `blurred`: +0.3 ms
    unclipped, +0.6 ms inside a `within`. A read-only PSCat accessor for
    the current clip's coverage (`None` when nothing is clipped) would
    save the fill and the scan. pkmotion's moving layers still ignore the
    clip (a slide or /focus inside a `within` shows past its edge). The
    same mask fixes them, but a 12-item cascade then measured +3.9 ms a
    frame inside a `within` (+1.0 ms unclipped), so that is a follow-up.

## How the gallery was made, and what it taught

Two pieces (`epicycles`, `one-frame`) were written by hand while building
the engine. Five were written by agents given only the skill and a brief,
with a required friction report. Those reports drove a round of engine
and skill changes (commit "Engine changes from the gallery authors'
friction reports"): the empty-stack contract, printing, `globally`,
`turns`, `loopnoise`, `polypath`, `eachspaced`, `eachonpath`, `fmod`,
`overlay`, loop-seam measurement, cropped blur, honest timing for
sequential movies, and most of `pitfalls.md`. The reports are summarised
in the commit message; the pieces' READMEs credit the techniques.

Requests from those reports that were **not** built, in rough order of
how often they came up:

- a tapered stroke along a path (comet trails, brush marks);
- progressive reveal of a textured stroke as a first-class word (the
  recipe in `modes.md` does it by hand);
- a gradient shorthand without loading artkit;
- radial repetition (`n { } around`), a disc primitive;
- a parallax camera and "visible world rectangle" for culling;
- velocity-continuous keyframes;
- per-contour morphing (a paper fold reads as a blob);
- canvas checkpoints cached on disk, so iterating on the end of a
  persistent painting does not repaint it from the start;
- `check` naming the procedure that left operands on the stack.

### The second gallery

`canon` was written by hand as the smallest statement of the design (a
scene re-based in time, space and theme, inside itself). The other five
(`how-it-works`, `circle-area`, `birthday`, `fresh`, `heron`) were written
by fresh agents given the skill, a brief and a required friction report,
and nothing else. Their reports produced, the same day: `sym` and run
lists for mathematics, `straightenpath`, `plotmark` / `plotrule` /
`inplot` / `valueat`, side and waypoint routes for edges, `placetext`,
`tempo` / `counts` / `kick`, `revealband`, `insetxy` / `subbox` /
`aspect`, multi-box `spotlight`, honest bar labels, printed output shown
when a frame fails, and about a third of the chapters' text. The films
themselves were left as their authors wrote them, workarounds included:
they are the record of what the vocabulary lacked that day.
`docs/vocabulary-evaluation.md` has the numbers and what was *not*
built; `docs/roadmap.md` ranks it.

## Known rough edges

- Motions rasterize what moves (soft at large scale-ups) and cost a
  layer each, about 1 to 1.5 ms at 720p.
- Cuts between storyboard beats leave single blank frames unless
  something persists across them; only `check --all` sees them.
- The round-two films' hand-built helpers duplicate words added after
  them. Porting them is optional tidying, not a fix.

- `reelduration` cannot tell an undefined clip from an undefined
  transition when called before they are defined, so it takes built-in
  names and `t-...` for transitions. A custom transition with another name,
  defined later, used to lengthen the film silently (#19); load now
  re-plans every measured reel once the program has run and fails with
  the entry's name if the length differs. The fix for the author is still
  to define the transition first or name it `t-...`; a declared
  `/name deftransition` was not needed.
- `check`'s pop detector (per tile, against the same tile in the pairs
  either side; docs/cli.md) is tuned on the gallery and test movies, not
  proven. Local pops are notes by default and warnings under `--all`
  (a change over half the frame is always a warning); a movie declares
  intended cuts with `/Cuts [t ...]`. The default scan takes about 3 s
  however slow the frames are, so it covers fewer runs of 24 frames
  (none over about 100 ms a frame): a pop elsewhere in a long movie is
  found only by `check --all`. How many runs it covers depends on the
  machine; `--sweep N` fixes the number. A movie with state is scanned only around
  its sampled frames. Rendering a pure movie's frames is cheap enough
  that `--all` costs seconds; for the slowest pieces it is minutes.
- Code before `animation` still runs twice (see decision 5); `check`
  warns when it prints or takes over 50 ms, but nothing prevents it.
- `MAX_OUTPUT` truncates printed text at 64 KB per collection. The text
  then ends with `[output truncated at 64 KB]` and `--json` says
  `"output_truncated": true`.
- The browser player renders on the main thread and has no way to show
  `print` output.
- Dropping a `Movie` from thread-local teardown panics (a PSCat interner
  ordering issue; documented in the PSCat note). `psanim-wasm`'s test
  unloads explicitly.

## Next steps

`docs/roadmap.md`, in two lists. For the vocabulary: a stroke scheduler
for painting over time, equations that transform, soft reveal edges. For
the engine: get the PSCat branch reviewed and merged; a Worker-based
browser player; an MCP server over the CLI's JSON.
