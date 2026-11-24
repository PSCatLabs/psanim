# What goes wrong, and why

## The frame is blank

`psanim check` says so before you look. Usual causes, most likely first:

- **Colour equals the background.** Every frame starts with the default
  colour, black, and the default `/Background` is black. Set a colour.
- **Off the canvas.** Coordinates are in `/Size` units, origin bottom
  left, y up. `psanim frame --json` reports the bounding box of what was
  drawn (`stats.bbox`, in pixels, y down).
- **The window is closed.** `a b { } during` draws nothing outside
  `a..b`. A sequence or reel shorter than `/Duration` leaves the tail
  empty: set `/Duration film reelduration`.
- **A tween is at 0.** `0 0 between`-style mistakes: a scale of 0, an
  alpha of 0, `0 0 trimpath`.

## It errors on some frames only

That is normal for a time-dependent program: a branch only taken after
t = 3. The error names the frame; reproduce it with `psanim frame movie.ps
--frame N`. Run `psanim check --all` (every frame, small) before a long
render.

## `invalidfont`

Nothing selected a font *in this frame*. Graphics state does not carry
from setup or from the previous frame. Put `findfont scalefont setfont`
in the scene (it is cheap: fonts are cached).

## `nocurrentpoint`

`show`, `lineto`, `rlineto`, `curveto` need a `moveto` first; every frame
starts with an empty path. Also: `grestore` and `stroke`/`fill` discard
the path. To use a path twice, `gsave fill grestore stroke`.

## `stackunderflow` in a combinator

Operands in the wrong order. The procedure is always last:

```postscript partial
0 2 { ... } during          % not  { ... } 0 2 during
3 0.1 { ... } staggered     % index, gap, body
0.5 { ... } faded           % alpha, body
6 0.8 { ... } glowing       % radius, strength, body
```

`psanim api NAME` prints the order.

## `scene` left N operands on the stack

Something pushed a value nothing consumed. The error lists the values,
which usually identifies the line: a number computed and not used, a
procedure name written `/name` where `name` was meant, an operator given
one operand too many. (One operand too few is a `stackunderflow`.) Bodies
of loops and combinators must be stack-neutral too.

## The value is right at the top level and wrong inside a scope

`now`, `tween`, `keyframes` and `progress` read the innermost time scope.
A procedure that computes a film-wide value (the sun's height, a camera
position) gives a different answer when called inside `during` or
`sequence`. Wrap its body in `{ ... } globally`, or pass `globaltime` to
`keyat`.

## I cannot see what a value is

Print it. `now ==`, `x y pstack`, `(reached here) =` in setup or in the
scene are shown by `psanim frame` (on stderr) and `psanim check`, and are
in `--json` output as `printed`. Remove them before the final render.

## A name I defined has the wrong value

- **In the scene, `def` is per frame.** `/x x 1 add def` in `scene` does
  not accumulate: the VM is rolled back before every frame. Compute from
  `now`, or make the movie stateful (see modes.md).
- **Nested procedures share names.** `/i exch def` in a loop that calls a
  procedure which also does `/i exch def` clobbers the outer `i`. Open a
  dictionary: `/myproc { 6 dict begin ... end } def`. (The body of a
  psanim combinator may `def` freely; the engine's own procedures never
  open a dictionary around it.)
- **A local dictionary and `exit`/errors.** If a body leaves a `begin`
  open, the next frame still starts clean, but later lookups in *this*
  frame resolve in the stale dictionary.

## A name I defined broke something else

`/width 300 def`, `/frame { ... } def`, `/span ... def`, `/dt ... def`
shadow psanim's own words for the rest of the program; `/index`, `/add`,
`/count`, `/copy` shadow PostScript's. Prefix your names (`/bar-w`,
`/cardframe`). psanim's own procedures are bound and keep working, but
your code that expected the original will not.

## The motion pops

- An object appears at full size: animate its entry (`0 0.4 /back-out
  tween` on scale, or `faded`).
- Two adjacent `during` windows compute a value differently at the
  boundary. Make the end of one equal the start of the next, or use one
  `keyframes` track across both.
- `check` flags an abrupt change with the frame number. A deliberate cut
  is fine; ignore the warning for it.

## It flickers

Something random changes per frame. `rand` is reseeded each frame, so
flicker usually comes from seeding with `frame` or `now`, from `wiggle`
at too high a frequency, or from `filmgrain` (which is meant to move).
For per-item constants use `i hash01`.

## It is slow

`psanim bench movie.ps` gives milliseconds per frame (`check` times
*preview-size* frames, which are several times cheaper). Typical costs
at 1280x720 on one core: a few hundred shapes and text, 1 to 5 ms; each
layer (`faded`, `blurred`, `glowing`), 2 to 4 ms to composite, plus for a
blur or glow 5 to 30 ms depending on how much of the frame is drawn on
it (an empty region costs nothing; a full-frame layer is the expensive
case, and cost grows with the pixel count); `vignette` and `filmgrain`,
about 3 to 6 ms each; paintkit watercolour strokes, tens of ms each.
Cures, in order:

1. Move anything that does not depend on time into setup (loading
   libraries, building arrays of points, L-system strings).
2. Fewer, larger layers: one `glowing` around many shapes, not one per
   shape.
3. Preview at `--scale 0.5`; render final frames on all cores (the
   default for a pure movie).
4. For a picture that only accumulates (a painting), use `/Persist true`
   so each frame paints only what is new (modes.md).

## `check` says cover 0 but the frame is not empty

`cover` is the fraction of pixels that differ from the frame's most
common colour, so a frame filled edge to edge with one colour has cover
0 and no bounding box. It measures "is there a subject", not "was
anything painted".

## The video looks different from the PNG

MP4 and WebM are 4:2:0: fine saturated red or blue detail softens, and
very dark gradients can band. Check stills from the video itself:
`ffmpeg -ss 2.5 -i movie.mp4 -frames:v 1 f.png`. Raise quality with
`--crf 12`. Fine line work compresses poorly: tens of megabytes for ten
seconds is normal at the default quality (`--crf 22` halves it), and a
GIF of it will be large whatever you do; prefer MP4 or WebM. Thin lines under 1.5 px shimmer in motion: thicken them or
render larger.
