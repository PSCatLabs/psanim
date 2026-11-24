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

`psanim bench movie.ps` gives milliseconds per frame. Typical costs at
1280x720: a few hundred shapes and text, 1 to 5 ms; a `glowing` or
`blurred` layer, 5 to 15 ms each; paintkit watercolour strokes, tens of
ms each. Cures, in order:

1. Move anything that does not depend on time into setup (loading
   libraries, building arrays of points, L-system strings).
2. Fewer, larger layers: one `glowing` around many shapes, not one per
   shape.
3. Preview at `--scale 0.5`; render final frames on all cores (the
   default for a pure movie).
4. For a picture that only accumulates (a painting), use `/Persist true`
   so each frame paints only what is new (modes.md).

## The video looks different from the PNG

MP4 and WebM are 4:2:0: fine saturated red or blue detail softens, and
very dark gradients can band. Check stills from the video itself:
`ffmpeg -ss 2.5 -i movie.mp4 -frames:v 1 f.png`. Raise quality with
`--crf 12`. Thin lines under 1.5 px shimmer in motion: thicken them or
render larger.
