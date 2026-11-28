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
  moves, cuts, and for a `/Loop true` movie whether the join is
  as smooth as the steps beside it. Local abrupt changes (something
  appears or vanishes in one frame) are *notes* by default: look at each
  with `sheet --frame F,F+1`, fix the ones with no entrance, and declare
  the intended ones with `/Cuts [t ...]`; `check --all` makes them
  warnings. Add `--json` for data.
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
   joins clips with dissolves, wipes and pushes. PSCat's own
   `<< /Alpha a >> begingroup ... endgroup` fades as one thing too; a
   psanim layer (`faded`, `blurred`, `glowing`, a moving motion) must not
   open inside one, where it loses its effect: put the group inside the
   layer ([pitfalls](references/pitfalls.md)).

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
  `width`, `dt`, `span`, `add`, `index`, ...). Three names belong to the
  engine: `scene`, and `step` and `overlay`, which change the movie's
  mode if you define them. Call your own procedure `card` or `stage1`,
  never `step`.

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
| (core) | `component`, `instance`, `place`: a movie written on its own, placed in another in a box at a time | [components.md](references/components.md) |
| (core) | `spec { ... } particles`: sparks, fireworks, confetti, snow; bursts and streams under gravity, drag and wind, every particle a function of time | [patterns.md](references/patterns.md#particles-without-state) |
| (core) | `cellgrid`, `cellsteps`, `cellrows`, `cellcolors`: cellular automata (Life, Brian's Brain, Rule 30) run natively, in a pure movie; the `/automaton` catalog entry | [modes.md](references/modes.md#cellular-automata-a-simulation-that-stays-pure) |
| (core) | `playlist`, `playlistduration`, `weektime`: a screen that cycles through scenes for ever, with transitions, a schedule and a safe area | [signage.md](references/signage.md) |
| `pktype` | text in boxes, wrapping, `kinetic` type, counters, captions, end credits, lines of maths and one equation becoming the next (`morphruns`) | [type.md](references/type.md) |
| `pkdiagram` | `node`, `edge` (with flowing pulses), `group`, `note`, `circled` | [explain.md](references/explain.md) |
| `pkcode` | `codeblock` (typing, focus), `codediff` | [explain.md](references/explain.md) |
| `pkchart` | `plotframe`, `plotaxes`, `bars`, `hbars`, `plotline`, `serieslabels`, `tracer`, `donut`; dashboards: `stattile`, `dial`, `horizon`, `sparkline`, `heatmap`, `statuslight` | [explain.md](references/explain.md) |
| `pkink` | tapered strokes, `revealalong`, `around`, gradients | [ink.md](references/ink.md) |
| `pkfigure` | a rig for characters that dance: `dancepose` from a beat, `squashed`, `limb`, `eye`, `mouth`, `spring` | [figure.md](references/figure.md) |
| `pkpixel` | a pixel font for dot-matrix work: `7 pixelfont`, then `pixelcols` and `pixeltext` give a character's or a line's dot columns as bit masks; the LED matrix, LED sign and dot-matrix printer entries share it. `/Phosphor pixelface` loads one of PSCat's pixel faces (terminal, printer, LED) | [type.md](references/type.md#pixel-faces-terminals-printers-led-signs), [vocabulary-libraries.md](references/vocabulary-libraries.md) |

The way of working they assume: **draw each thing at rest, in a box, in
theme colours; then say when it arrives and by what motion.** Start from
this film: it is the shape most films should have.

```postscript
%!PS
<< /Size [1280 720] /Duration 9 /FPS 30 >> animation
/pkdiagram use                                   % loads pkstage, pkmotion, pktype

% ---- places: cut from the page, in vmin, named ------------------------------
/page  canvasbox 7 vmin inset def
/head  page 0 0.84 1 0.16 subbox def             % top 16%
/stage page 0 0.30 1 0.38 subbox def             % a band for the diagram
/slots stage 3 9 vmin cols def                   % three nodes, 9 vmin apart
/foot  page 0 0 1 0.14 subbox def
/gap 0.5 def                                     % seconds between nodes

% ---- things at rest ---------------------------------------------------------
/names [ (write) (check) (render) ] def
/card  { names exch get exch << /Size 4.6 vmin >> node } def      % box i

% ---- the film: one step after another, no second written twice --------------
/scene {
  ground
  /display 9 vmin face /fg ink
  (How it works) head /w placetext (How it works) /glyphs 0.04 /mask-up kinetic
  {
    1.0 wait
    0 { slots gap { /i exch def /b exch def /pop { b i card } entering } cascade } then
    slots gap cascadespan 0.4 add wait
    0.9 { slots 0 get slots 1 get << /Draw 0 0.6 /cubic-inout tween >> edge } then
    1.2 { slots 1 get slots 2 get << /Draw 0 0.6 /cubic-inout tween /Pulse 1.2 2 pulsetrain >> edge } then
    0 { [/rise 0.6] { /body 4.4 vmin face /muted ink
        (each step starts where the last one said the next should) foot /w showin } entering } then
  } script
  slots 2 get 6.5 7.5 /quad-out tween 0.5 mul spotlight
} def
```

### Write it to be changed

A film is edited more often than it is written: a step is inserted, the
pace changes, the frame becomes vertical, the look changes. Measured on
the same 20 second film: inserting one two-second step took **3 changed
lines** when the film was a script and **about 50, with a dozen numbers
retyped by hand,** when it was written in absolute seconds (and a missed
number still passes `check`). So:

1. **Time: a `script`, not a list of seconds.** `seconds { ... } then`
   starts a step where the last one said the next should; inside it the
   clock starts from 0, so `entering`, `0 0.6 tween` and `edge /Draw` are
   written from zero. For a long film, a `storyboard` of named beats,
   each beat a script. The only absolute times should be things that
   belong to the whole film (the title, a closing `spotlight`).
2. **Never copy a time that depends on another.** Name the gap (`/gap 0.5
   def`) and derive: `slots gap cascadespan` is how long a cascade takes
   to start its last item, `3 gap mul` when item 3 starts.
3. **Place: boxes cut from the page, not coordinates.** `page 0 0.3 1
   0.38 subbox`, `cols`, `rows`, `cells`, `placed`; sizes in `vmin`.
   Things placed relative to a named box follow when the layout changes.
   A literal like `[ 70 460 190 80 ]` works at one size only.
4. **Look: roles, not colours.** `/accent ink`, `/title 6 vmin face`. A
   restyle is then one `usetheme` line.
5. **Say what can be edited, at the top.** A block of named durations
   (`/lookdur 3.4 def`), the data, the theme line. The film's length
   should follow from them (`storylength`, or the sum of the names).
6. **Then tune the sizes by looking.** Derived layout is not automatically
   good layout: five equal columns across a page give five cramped
   boxes. Give a node at least 16 vmin by 9 vmin, body text 4 to 5 vmin,
   labels no less than 3 vmin, and check one full-size `frame` per
   section.

### The words you will use most

| to | write |
|---|---|
| cut up the page | `canvasbox 7 vmin inset`, `box fx fy fw fh subbox`, `box n gap cols`, `box n gap rows`, `box nc nr gap cells`, `box /c boxpoint` |
| set colour and font | `/fg ink` (`/muted /accent /accent2 /accent3 /good /bad /bg /surface /line`), `/title 6 vmin face` (`/body /mono /display /serif`), `ground` |
| put text in a box | `(text) box /c showin` (`/w` left, `/e` right), `(text) box /title fitface`, `(text) box << /Align /center >> paragraph` |
| bring a thing on | `/pop { ... } entering` in a script step; `t /pop { ... } enters` at a time |
| take it off | `/rise /fade { ... } staged` inside `thenfor` or `inbeat`; `a b /rise /fade { ... } onstage` |
| call attention | `t /shake { ... } emphasized`, `box amount spotlight`, `(text) box /ne u note`, `box circled 0 u trimpath stroke` |
| one after another | `{ 0.6 { ... } then  0.3 wait  2 { ... } thenfor } script` |
| a row, staggered | `boxes gap { /i exch def /b exch def /pop { ... } entering } cascade` |
| animate a number | `a b /cubic-out tween v0 v1 between` (inside a step, a and b count from the step's start) |
| type that moves | `(TEXT) box /c placetext (TEXT) /glyphs 0.05 /mask-up kinetic` |
| a diagram | `(label) box /surface node`, `boxA boxB << /Draw u /Label (x) /Pulse u >> edge` |
| a chart | `box [x0 x1] [y0 y1] plotframe`, `<< >> plotaxes`, `[v ...] << /Grow u >> bars`, `pts plotline 0 u trimpath stroke`, `lines names << /Draw u >> serieslabels`, `pts x << >> tracer`, `x y << /Label (x) >> plotmark` |
| a dashboard | catalog panels `/stat` `/gauge` `/horizon` `/timeseries` `/status` `/counter` reading `/Signal /cpu` from `--signals day.csv`; by hand `v box << /Thresholds [/good 70 /warn 90 /bad] >> stattile`, `/cpu 3 60 signalseries` |
| code | `/src (...) /rust codetokens def` in setup, `src box << /Reveal u /Focus [2] >> codeblock` |
| a sign, a word in another language, emoji | type it: a `face` font reads UTF-8 (`(2πr → 👋🏽 Grüße)`); `/pi sym` for Symbol's design. Korean, Japanese, Chinese, Thai, Arabic and Indic scripts draw empty boxes, and `check` names them |
| particles | `<< /At [x y] /Bursts [1.2] /Count 120 /Gravity 80 /Drag 1.5 >> { 1 particleu sub setalpha 0 0 3 0 360 arc fill } particles` |

### What reviewers mark down

From blind reviews of finished films, the defects that recur:

- **A hold that is not a hold.** "Holds for the last 3 seconds" means
  fully arrived and still for 3 seconds: start the entrance earlier, so
  it has finished when the hold begins.
- **Dead time.** A second or more with nothing arriving and nothing to
  read. Tighten the gap, or let the next thing start.
- **Secondary text too faint or too small.** `/muted` on a dimmed
  element, or a label under 3 vmin, is unreadable in the video.
- **A heading that shouts.** `fitface` fills its box: give a heading a
  box of modest height (10 to 12% of the page), not the page's width.
- **A state that contradicts the story.** If a thing is ruled out, it
  must lose its highlight as well as dim.
- **Things that collide while moving.** A large `/drop` or `/zoom`
  passes through whatever is above or around it: look at three frames
  inside every big entrance.
- **Late for the beat.** "On the beat" or "at 4 s" means *arrived* then,
  not starting then. `enters` takes the start time, so start a motion
  its own duration early: `4 0.6 sub [/rise 0.6] { ... } enters`.
- **An ending that is still moving, or already gone.** End on the
  finished picture, everything arrived and nothing leaving, for at least
  a second unless the brief says otherwise. Look at the last frame.
- **A line with no name.** Name each series at the end of its line with
  `serieslabels` (it keeps names that end close together apart), not in
  a legend the eye has to travel to, and stroke line i in `chartroles i
  get` so each name is in its line's colour.

Before using a word you have not seen in an example, run `psanim api
NAME`: a plausible name that does not exist is the commonest first error.

`psanim api --libs` lists the libraries, `psanim api pkmotion` one
library's words, `psanim api enters` one word. What to do with them (how
to introduce an idea, compare two things, follow a process, single out a
detail) is in [storytelling.md](references/storytelling.md).

## Reference, by need

| You want | Read |
|---|---|
| Boxes, themes, motions, scripts, storyboards, camera by box | [references/stage.md](references/stage.md) |
| Text: fitting, wrapping, kinetic type, counters, captions, end credits | [references/type.md](references/type.md) |
| Diagrams, code, charts | [references/explain.md](references/explain.md) |
| Brush marks, self-drawing textured strokes, PSCat media in motion | [references/ink.md](references/ink.md) |
| A character that dances to a beat (Twiddle, and your own) | [references/figure.md](references/figure.md) |
| How to structure an explanation or a story | [references/storytelling.md](references/storytelling.md) |
| Every core word, stack effect and meaning | [references/vocabulary.md](references/vocabulary.md) (generated; same as `psanim api --markdown`) |
| Every library word | [references/vocabulary-libraries.md](references/vocabulary-libraries.md) (generated) |
| Building a movie from parts: components, instances, placing them | [references/components.md](references/components.md) |
| Signage: a looping playlist of scenes, loaded from a file, with a schedule, live tiles, safe areas | [references/signage.md](references/signage.md) |
| Ready-made elements to place (spinners, clocks, an opening crawl, and more) | [references/catalog.md](references/catalog.md) (generated; `psanim catalog`) |
| Low-level recipes: write-on, cameras, perspective (`onplane`), particles, loops, reels, keyframes | [references/patterns.md](references/patterns.md) |
| What goes wrong and why | [references/pitfalls.md](references/pitfalls.md) |
| Making it good: timing, easing, composition, colour, how to review motion | [references/craft.md](references/craft.md) |
| Stateful simulations, persistent canvases, signals, parameters, the time of day for clocks, motion blur | [references/modes.md](references/modes.md) |
| Complete small movies to start from | [examples/](examples/) |

For drawing itself (brushes, palettes, L-systems, text layout, charts),
use the `psart` and `pscat` skills: everything they describe works inside
a scene. Worth knowing without opening them, because they need no library:
`a setalpha` (opacity 0..1 for what is painted next), the native noise
operators (`x y z gnoise3`, `x y z oct gain lac fbm3`: drive z with time),
and gradients through `shfill` (artkit's `axialsh`/`radialsh` wrap it).
