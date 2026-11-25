# The expressive vocabulary: design and rationale

The first phase of psanim built the machinery: a clock, time scopes, pure
frames, layers, reels. This document explains the second: the `pk`
libraries, the vocabulary an author (usually an agent) writes a film in.
It says what the abstractions are, why these and not others, and what was
tried and rejected. The measurements behind the claims are in
[the evaluation](vocabulary-evaluation.md).

## The question

*What would an expressive, composable vocabulary for animation look like
if its primary authors were intelligent agents?*

Our answer has three parts.

1. **An agent's scarce resource is correct spatial and temporal
   bookkeeping, not ideas.** The first gallery's friction reports are
   mostly arithmetic: where a label goes, when an entrance ends, which
   alpha multiplies which. A vocabulary should take the bookkeeping and
   leave the decisions.
2. **PostScript already has the right shape.** Its drawing model is
   *state you re-base for a procedure* (`translate`, `gsave`). psanim's
   core extended that to time. The libraries extend it twice more, to
   the canvas and to the look, and stop there.
3. **Few orthogonal ideas beat many convenient ones.** There are about
   140 new words, but five ideas. A reader who knows the five can guess
   most of the words.

## The five ideas

### 1. A box is four numbers

`[x y w h]`. Layout is cutting boxes (`inset`, `cols`, `rows`, `cells`,
`splitx`, `placed`) in setup and naming the pieces. Everything that
needs a place takes a box: `node`, `showin`, `codeblock`, `plotframe`,
`lookat`, `spotlight`, `within`.

Because a box is a plain array, the core's interpolation already works on
it: `u a b between` is a thing moving and resizing, `[t box t box ...]
keyframes lookat` is a camera move, and no new word was needed for
either. This is the main reason boxes are arrays and not dictionaries or
objects.

What a box is *not*: an object with identity, children or a transform.
There is no scene graph. A "node" is a procedure call that draws into a
box this frame; next frame it is drawn again from nothing. That is what
keeps frames pure, and it turned out not to cost expressiveness, because
of the next idea.

### 2. A motion measures what it moves

The obvious design for entrances needs every drawable to know its
bounds, which means objects. We did not want objects. Instead:

> the engine draws the body on its own layer, finds the box of what was
> actually painted, and composites that layer under the motion's pose.

So `1.5 /pop { anything } enters` scales *anything* about its own centre,
`/mask-up` slides it inside its own box, `/wipe-right` reveals it across
its own width, with no anchor, size or pivot from the author. The
drawing is an object for exactly as long as it is moving. At rest
(before, it is skipped; after, it is drawn directly) there is no layer
and no cost, and the result is pixel-identical to the plain drawing,
which a test asserts for every motion.

A motion is a function from one number to a pose (offset, scale, turn,
opacity, blur, reveal, pivot). Entrances run it 0 to 1, exits run the
same function backward (so the exit of `/pop` anticipates, for free),
emphases are poses that rest at both ends. A program adds a motion by
putting a procedure in `motions` that returns a pose dictionary.

Costs, stated plainly: a moving thing is rasterized, so a large scale-up
is soft while it moves; each concurrent motion is a canvas-sized layer
(a few ms at 720p); and a motion applied to something that covers the
frame measures the frame.

### 3. Time is scripted or storyboarded, never tabulated

The first gallery's films are full of absolute seconds. Changing one
means changing thirty. Two small devices remove them:

- **`script` / `then` / `wait`**: a cursor. Each step starts where the
  last one said the next should. Because a frame is a pure function, a
  script is not a timeline being played: it is re-run every frame and
  each step asks "has my time come?". That is why scripts nest and need
  no engine support beyond a counter.
- **`storyboard`**: named beats with durations, declared before
  `animation` so they also give the film's length. `/build { } inbeat`,
  `/build { } frombeat`.

`cascade` (a row of things, each a gap later) completes the set. These
live in the core, not a library, because they are about time, like
`during`.

### 4. A scene is a function of its viewport and its theme, too

`box { } within` re-bases the canvas the way `during` re-bases the
clock: `width`, `height`, `canvasbox` and `vmin` become the box's. A
scene written against those can be placed in a panel, twice side by side
for a comparison, or inside itself ([Canon](../gallery/canon/README.md)).
The same property gives aspect-ratio independence: a film composed in
boxes and `vmin` is a function of `/Size`.

A **theme** maps roles to colours and fonts (`/accent ink`, `/title 40
face`). Library words take roles wherever they take colours, so a
diagram, a chart and a code panel in one film agree without the author
choosing a single RGB value, and changing the look is one line.

Together these give the four separations the skill teaches: content (a
procedure that draws a thing at rest), place (a box), look (roles), time
(a motion at a moment). Each can be edited without touching the others.

### 5. Structure is the current path

Connectors, plotted lines, hand-drawn marks and braces are all delivered
as *the current path*, not as finished drawings: `a b /elbow connect`,
`pts plotline`, `box circled`. The core already knows how to draw a path
on (`trimpath`), ride it (`onpath`, `pathpoint`), taper it (`taperpath`)
and reveal a texture along it (`revealalong`), so every one of those
structures animates in every one of those ways without each library
re-implementing them. One-call conveniences (`edge`, `arrow`) sit on top
for the common case.

## The libraries

| | what it adds | built on |
|---|---|---|
| core additions | `within`, `fitted`, `script`/`then`, `storyboard`, `tempo`/`counts`, `cascade`, `onpath`, `taperpath`, `straightenpath`, `smoothkeys`, `valueat`, `fmtnum`, `codetokens` ... | Rust + prelude |
| `pkstage` | boxes, themes, `lookat`, `spotlight` | PostScript |
| `pkmotion` | `enters`, `leaves`, `onstage`, `emphasized`, `motion`, `echoed`, `rank` | native layer placement |
| `pktype` | `showin`, `placetext`, `fitface`, `paragraph`, `kinetic`, `marker`, `typewrite`, `captions`, `sym`, run lists (`showruns`) | pkstage, pkmotion |
| `pkdiagram` | `node`, `edge`, `connect`, `arrow`, `pulse`, `group`, `note`, `circled` | pktype |
| `pkcode` | `codeblock`, `codelinebox`, `codediff` | native tokenizer and diff |
| `pkchart` | `plotframe`, `plotaxes`, `bars`, `hbars`, `plotline`, `plotmark`, `plotrule`, `dots`, `donut`; for dashboards `stattile`, `dial`, `horizon`, `sparkline`, `heatmap`, `statuslight`, `panel`, `thresholdrole` | pktype |
| `pkink` | `revealalong`, `revealband`, `around`, `brushstroke`, `boxgradient` | pkstage |

Libraries are PostScript source compiled into the binary and loaded with
`/pkmotion use`. They are opt-in so that a movie's namespace holds only
what it asked for; a word used without its library produces an error
that names the library.

### What is in Rust and what is in PostScript

In Rust: anything that touches pixels (placing a layer under a pose),
anything that is data processing rather than drawing (tokenizing,
diffing, number formatting, tick selection, ranks), and anything that
needs the flattened path (`taperpath`). In PostScript: everything that
draws, so an author can read a library word, copy it and change it. The
libraries are about 1,750 lines of PostScript.

### Conventions

- Operands first, procedure last; positional for what is always given,
  a dictionary (`<< /Grow u /Role /accent >>`) for what is optional. A
  bare name is accepted where the dictionary would hold one common
  option (`a b /elbow edge`).
- Animated quantities are plain numbers the caller computes (`/Draw`,
  `/Grow`, `/Reveal`, `u marker`), almost always with `tween` or `ramp`.
  No library word owns a clock. This keeps every word usable under any
  time scope, script or signal.
- Library words that run a caller's procedure never leave a dictionary
  open around it; private names carry a dot or a prefix.
- Names are checked by a test against every PSCat library and against
  each other.

## What was deliberately not built

- **A scene graph or retained objects.** See idea 2: measuring the
  drawing gave the benefits without the model.
- **`introduce`, `compare`, `explain` operators.** The brief suggested a
  storytelling library. The intents are real, but as operators they
  would fix a layout and a pacing that every film wants to choose. They
  are instead a chapter of the skill (`storytelling.md`): each intent is
  five to fifteen lines of the existing words, shown and tested.
- **A separate camera library.** `lookat` plus interpolated boxes covered
  every camera move the productions needed.
- **Auto-layout of graphs.** Explainers want deliberate layouts, and
  grids of boxes give them. A layered-graph layout is on the roadmap for
  data-driven diagrams.
- **An equation layout engine.** PSCat's mathkit already sets a TeX
  subset as paths, which `trimpath` and motions animate. For the lines
  an explainer needs, run lists (`showruns`, `showrun`, `runbox`) set
  mixed type and bring it in term by term; they were added after the
  maths film's author wrote a fifty-line typesetter by hand. One
  equation *becoming* another is the largest remaining gap (see the
  roadmap).

## How it was arrived at

By use. The libraries were drafted from the first gallery's friction
reports, exercised on small test films, documented, and then given to
fresh agents with only the skill and a brief for an ambitious film each,
with a required friction report. Their reports changed the libraries and
the skill; the evaluation lists what changed and what the experiment
measured.

## Components (added for 0.2)

A movie can be built from parts written on their own (`component`,
`instance`, `place`). The constraints above still hold: there is no scene
graph, nothing owns a clock, and a box is a plain array. A component is
not an object with bounds: it is a movie whose scene is called, at a time
the host computes, inside `fitted`.

What makes a component more than a procedure is isolation:

- **Names.** Each instance gets a dictionary, open while its setup and
  scene run. This is the one place the prelude opens a dictionary around
  code it did not write, and it is the point: that dictionary is the
  component's namespace, not one left around a caller's procedure.
  Libraries always load into userdict, whoever calls `use`.
- **Clock, seed, parameters.** A native stack of component frames
  (`Ctx::comps`) makes `globaltime`, `duration`, `globally`, `movieseed`,
  the noise words and `param` answer for the innermost component. Signals
  and events stay on the movie's clock: they come from outside.
- **Library state.** Libraries that change a shared dictionary (pkstage's
  `theme`, pkchart's current plot) do it through `.mine`, which inside a
  component gives a private copy made on first use.

Rejected: running each instance in its own interpreter (cost, and the
drawing must land in the host's page); a `/Params` key in the declaration
(the movie's own `param` defaults already say it, and keep a component
file runnable on its own).

## Dashboards (added for 0.2)

Telemetry panels follow the same rules. The pkchart words (`stattile`,
`dial`, `horizon`, `sparkline`, `heatmap`) take numbers and a box; none
reads a signal or a clock. Data reaches a movie only from the host:
signals, which now have a past (`signalago`, `signalseries`,
`signalbetween`) so a pure frame can draw a window of history, and which
a live host feeds with `Movie::record_signal` so the same movie works on
a wall display. The ready-made panels are catalog entries that read
their signal by name. A recorded day and a live feed are the same thing
to the movie; a gallery piece names its recording with a
`% Signals: day.csv` line that the gallery scripts and the docs test pass
as `--signals`.

Two choices keep a dashboard steady. A window of history is read on a
grid fixed in time, not at instants that slide with `now`, so peaks do
not shimmer. And a moving axis cross-fades between the round tick sets
either side of its ideal step (`fadeticks`, `plotaxes /Fade`), instead of
re-picking ticks and jumping.

Rejected: a declaration key naming the movie's data file (the host
should decide where data comes from, and a browser host has no files);
an accumulating history inside the movie (state between frames; a host
recording is the explicit place for it).
