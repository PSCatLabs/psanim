---
name: psanim
description: Author, preview, debug and render animations written as PostScript programs with the psanim engine (PSCat-powered). Use when asked to make an animation, motion graphic, animated explainer (architecture diagrams, code walkthroughs, data stories, maths), kinetic typography, title sequence, looping visual, simulation or short film as code, or to edit or diagnose a psanim `.ps` movie. Covers the core vocabulary (time scopes, tweens, path drawing, layers, reels), the pk libraries (layout and themes, named motions, type, diagrams, code, charts, ink), the check / sheet / render loop, and how to judge motion from stills.
---

# psanim: animation as a PostScript program

A movie is one `.ps` file: a declaration, then a `scene` procedure that
psanim calls once per frame. The scene reads the clock with `now` and
draws with ordinary PostScript plus everything in PSCat's libraries
(`(lib/artkit.ps) run`, paintkit, the fonts). There is no timeline file
and no scene graph: **a frame is a function of its time**, so any frame
renders on its own, in any order.

```postscript
%!PS
<< /Size [1280 720] /Duration 6 /FPS 30 /Background [0.05 0.06 0.09] >> animation

/scene {
  640 360 translate
  0.5 2.0 /cubic-out tween 0 300 between 0 translate    % slide 300 right over t = 0.5..2
  1 0.8 0.3 setrgbcolor  0 0 40 0 360 arc fill
} def
```

## The loop (run these; do not skip `check`)

```sh
psanim check  movie.ps                 # 1. loads, samples 16 small frames, reports errors and warnings
psanim sheet  movie.ps -o sheet.png    # 2. 12 frames on one labelled image: LOOK at it
psanim frame  movie.ps --at 3.2 -o f.png     # 3. one frame at full size, to judge detail
psanim check  movie.ps --all           # 4. every frame, small: before any long render
psanim render movie.ps -o movie.mp4    # 5. the film (also .webm, .gif, or a directory of PNGs)
```

- `check` is cheap (milliseconds per frame) and catches most mistakes
  before you look at anything: errors with the frame number and a
  backtrace through your own procedures, blank frames, a movie that never
  moves, abrupt pops, and for a `/Loop true` movie whether the join is
  as smooth as the steps beside it. Add `--json` for data.
- `check` samples 16 frames; a single blank frame where one section ends
  and the next has not begun, or an error in a branch taken only at
  t = 31, needs `check --all`.
- A contact sheet shows *composition over time*. Useful forms: `--at
  2,7.5,12` (named moments), `--count 16 --cols 4 --thumb 480` (legible
  thumbnails). A 24-up sheet hides small type: look at one full-size
  `frame` per section before calling it done.
- To judge *motion* (an
  easing, an overlap, a pop) sample densely around the moment:
  `psanim sheet movie.ps --from 2.0 --to 3.0 --count 12`.
- `psanim api` lists the whole vocabulary; `psanim api tween` explains one
  word. Trust it over memory.
- Preview small while iterating: `--scale 0.5` on any command. `psanim
  bench movie.ps` times full-size frames; `psanim info` prints what the
  movie declares; `psanim help` lists every option.
- A persistent movie (`/Persist true`) repaints from frame 0 for every
  `check`, `sheet` and `frame`: expect each to cost the film up to that
  moment.

## The five ideas

1. **`now` is the clock, in seconds.** At the top of `scene` it is the
   movie time. `progress` is the same thing as 0..1.
2. **Time scopes re-base the clock for a procedure**, the way `translate`
   re-bases space. `a b { ... } during` runs the body only for
   `a <= now < b`, and inside it `now` starts from 0 and `progress` runs
   0..1. So a procedure written against `now` can be placed, delayed
   (`d { } delayed`), repeated (`p { } looped`), slowed (`k { }
   timescaled`) or staggered across a row (`i gap { } staggered`) without
   changing it.
3. **`tween` then `between`.** `a b /ease tween` is the eased progress of
   `now` from time a to time b, clamped to 0..1 outside. Pipe it into
   `v0 v1 between` to get a value: `1 2.5 /cubic-out tween 100 400
   between`. `between` also interpolates arrays (points, colours).
4. **Draw a path on with `trimpath`.** Build any path (including text via
   `true charpath`), then `0 u trimpath stroke` shows the first `u` of it.
5. **Layers group.** `alpha { ... } faded` fades a whole drawing as one
   thing; `radius strength { ... } glowing` adds light; `[...] reel`
   joins clips with dissolves, wipes and pushes.

## Rules that save a debugging round

- **Every frame starts clean**: no font, no path, default colour, the
  whole VM as setup left it. Select fonts *inside* `scene`. Anything a
  frame `def`s is gone on the next frame; that is what makes frames
  independent. Compute constants in setup (the top level), per-frame
  values in `scene`.
- **`scene` runs on an empty operand stack and must leave it empty.**
  The time is not an operand: read `now`. A value left behind is reported
  as an error with the values shown, because it is nearly always a
  mistake (an operand some operator was meant to take).
- **Combinators take their procedure last**: `0 2 { ... } during`.
- **Stack discipline**: a body must leave the stack as it found it. Use
  a local dictionary for names (`8 dict begin ... end`) so recursion and
  nesting do not clobber them.
- **Randomness**: `rand` is reseeded identically every frame, so a random
  layout holds still. For a per-item constant use `i hash01` (0..1,
  stable). For smooth drift use `key freq amp wiggle`.
- **Units**: the canvas is `/Size` units with the origin at the bottom
  left and y up. Output resolution is a render option (`--scale`,
  `--width`), never something the program knows.
- Do not use `save`/`restore`, `showpage`, or files for output; the engine
  owns those. `gsave`/`grestore` are yours.
- Don't name a variable after an operator or a psanim word (`now`, `frame`,
  `width`, `dt`, `span`, `add`, `index`, ...).

## When something fails

Errors name the frame, the command, the procedure and the element:

```
error in frame 45 (t=1.500s): typecheck in `lineto`
  in petal: 0 0 moveto 10 (x) >>> lineto <<< stroke
  in flower: >>> petal <<< 90 rotate
  operand stack (top last): 0.5 10 (x)
  hint: `lineto` got an operand of the wrong type. ...
```

Reproduce with `psanim frame movie.ps --frame 45`. To see a value,
print it: `now ==` or `(here) =` in the program shows up in the output of
`psanim frame` and `psanim check`. Read
[references/pitfalls.md](references/pitfalls.md) for the common causes.

## The libraries: say what you mean

The core vocabulary above is enough for anything, and for most films it
is too low: an entrance is three tweens, a diagram is arithmetic. The
`pk` libraries are the level to write at. Load them in setup, after
`animation`:

```postscript partial
/pkstage use /pkmotion use       % almost always; the others load these
```

| Library | Gives you | Read |
|---|---|---|
| `pkstage` | boxes (`[x y w h]`) and layout, themes (`/accent ink`, `/title 40 face`), `lookat` (camera by box), `spotlight` | [stage.md](references/stage.md) |
| `pkmotion` | `2 /pop { ... } enters`: entrances, exits and emphasis for any drawing, by name | [stage.md](references/stage.md) |
| (core) | `script` / `then` (one thing after another), `storyboard` (named beats), `cascade`, `within` | [stage.md](references/stage.md) |
| `pktype` | text in boxes, wrapping, `kinetic` type, counters, captions | [type.md](references/type.md) |
| `pkdiagram` | `node`, `edge` (with flowing pulses), `group`, `note`, `circled` | [explain.md](references/explain.md) |
| `pkcode` | `codeblock` (typing, focus), `codediff` | [explain.md](references/explain.md) |
| `pkchart` | `plotframe`, `plotaxes`, `bars`, `hbars`, `plotline`, `donut` | [explain.md](references/explain.md) |
| `pkink` | tapered strokes, `revealalong`, `around`, gradients | [ink.md](references/ink.md) |

The way of working they assume: **draw each thing at rest, in a box, in
theme colours; then say when it arrives and by what motion.**

```postscript
%!PS
<< /Size [1280 720] /Duration 6 /FPS 30 >> animation
/pkdiagram use                                   % loads pkstage, pkmotion, pktype
/page canvasbox 8 vmin inset def
page 0.3 splity /body exch def /head exch def
/slots body 3 12 vmin cols def
/scene {
  ground
  /display 9 vmin face /fg ink
  head /w boxpoint moveto (How it works) /glyphs 0.04 /mask-up kinetic
  {
    0.8 wait
    0.5 { /pop { (write) slots 0 get /surface node } entering } then
    0.5 { /pop { (check) slots 1 get /surface node } entering
          slots 0 get slots 1 get << /Draw 0.1 0.6 /cubic-inout tween >> edge } then
    0.5 { /pop { (render) slots 2 get << /Fill /accent /Ink /bg >> node } entering
          slots 1 get slots 2 get << /Draw 0.1 0.6 /cubic-inout tween /Pulse 1.2 2 pulsetrain >> edge } then
  } script
} def
```

`psanim api --libs` lists the libraries, `psanim api pkmotion` one
library's words, `psanim api enters` one word. What to do with them (how
to introduce an idea, compare two things, follow a process, single out a
detail) is in [storytelling.md](references/storytelling.md).

## Reference, by need

| You want | Read |
|---|---|
| Boxes, themes, motions, scripts, storyboards, camera by box | [references/stage.md](references/stage.md) |
| Text: fitting, wrapping, kinetic type, counters, captions | [references/type.md](references/type.md) |
| Diagrams, code, charts | [references/explain.md](references/explain.md) |
| Brush marks, self-drawing textured strokes, PSCat media in motion | [references/ink.md](references/ink.md) |
| How to structure an explanation or a story | [references/storytelling.md](references/storytelling.md) |
| Every core word, stack effect and meaning | [references/vocabulary.md](references/vocabulary.md) (generated; same as `psanim api --markdown`) |
| Every library word | [references/vocabulary-libraries.md](references/vocabulary-libraries.md) (generated) |
| Low-level recipes: write-on, cameras, particles, loops, reels, keyframes | [references/patterns.md](references/patterns.md) |
| What goes wrong and why | [references/pitfalls.md](references/pitfalls.md) |
| Making it good: timing, easing, composition, colour, how to review motion | [references/craft.md](references/craft.md) |
| Stateful simulations, persistent canvases, signals, parameters, motion blur | [references/modes.md](references/modes.md) |
| Complete small movies to start from | [examples/](examples/) |

For drawing itself (brushes, palettes, L-systems, text layout, charts),
use the `psart` and `pscat` skills: everything they describe works inside
a scene. Worth knowing without opening them, because they need no library:
`a setalpha` (opacity 0..1 for what is painted next), the native noise
operators (`x y z gnoise3`, `x y z oct gain lac fbm3`: drive z with time),
and gradients through `shfill` (artkit's `axialsh`/`radialsh` wrap it).
