# Handoff: where psanim stands

Written 2026-11-23, at the end of the first build. For the next person or
agent.

## State

Working and tested on macOS (Apple silicon):

- `crates/psanim`: engine. Pure, stateful (`step`) and persistent
  (`/Persist`, with optional `overlay`) movies; motion blur; params,
  signals, events; budgets; structured errors with PostScript backtraces.
- `crates/psanim-cli`: `info check frame sheet render bench api`.
  MP4, WebM, GIF, PNG sequence exercised.
- `crates/psanim-wasm` + `web/`: browser engine, `<psanim-player>`,
  playground. Exercised in Node and Chromium.
- `gallery/`: nine pieces. `skills/psanim/`: the agent skill.
  `docs/` + `scripts/build_site.py`: Markdown docs and the HTML site.
- PSCat branch `feature/psanim-library-api` in `../pscat-psanim-api`:
  three commits on top of `feature/0.7.1` @ `72a2c4e`. Full PSCat suite
  green (3688 passed). **Local only: not pushed, not merged, no issue or
  PR opened.** That is the owner's call; `notes/psanim-library-api.md` in
  that branch is the write-up to open one from.

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
   `animation` therefore runs twice; keep it to definitions.
6. **`backdrop` respects the CTM and clip** so reels can wipe and push
   clips that start with one.
7. **PSCat changes are general.** Nothing in `pscat::host` knows about
   animation. Keep it that way.

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

## Known rough edges

- `reelduration` cannot tell an undefined clip from an undefined
  transition when called before they are defined; a transition must be
  named `t-...` or defined first. Wrong order silently lengthens the film.
- `check`'s abrupt-change detector is a threshold on mean difference;
  it misses small pops in busy frames.
- `MAX_OUTPUT` truncates printed text at 64 KB per collection.
- The browser player renders on the main thread and has no way to show
  `print` output.
- Dropping a `Movie` from thread-local teardown panics (a PSCat interner
  ordering issue; documented in the PSCat note). `psanim-wasm`'s test
  unloads explicitly.

## Next steps

`docs/roadmap.md`. The first three: get the PSCat branch reviewed and
merged; a Worker-based browser player; an MCP server over the CLI's
JSON.
