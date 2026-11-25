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

## A shape I did not draw gets filled

`clip` does not clear the path. After `x y r 0 360 arc clip`, the circle
is still the current path and the next `fill` paints it too. Write
`clip newpath`. Likewise `gsave ... grestore` restores the path that was
current at the `gsave`.

The same goes for artkit's `gradfill`: it paints the current path with a
gradient and leaves the path in place. Follow it with `newpath`, or the
next small `fill` repaints the whole gradient area in a flat colour.

## Things inside `faded` come out fainter than expected

The current `setalpha` applies to what is painted inside a layer, and
then the layer's own opacity applies to the result. `0.5 setalpha` before
`0.8 { ... } faded` gives 0.4. Set `1 setalpha` before a layer unless you
mean it.

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

## A frame depends on the frames rendered before it (strings)

Every frame starts with a `restore`, which rolls back arrays, dictionaries
and names, **but not the contents of a string**. A scene that `put`s into
a string made in setup (a buffer, a text it edits in place) changes that
string for every later frame. The picture then depends on which frames
were rendered before: a parallel render differs from a serial one, and
`psanim frame --at 5` differs from frame 150 of the full render.
`psanim check` reports it, naming the string
("a frame wrote into a string made in setup: `buf`").

```postscript partial
/buf 16 string def              % setup
/scene { buf 0 now cvi put  ... } def                         % wrong: the write stays
/scene { /buf buf dup length string copy def
         buf 0 now cvi put  ... } def                         % right: a copy that is the frame's own
```

Copy the string inside the scene (`s dup length string copy` makes a new
one) or build the string there. Reading a setup string is fine; so is
`def`ing a new string in the scene (that is rolled back). The same goes
for a string held in an array or a dictionary made in setup: `check`
looks inside them. It does not look inside a procedure body, so a literal
`(abc)` written into by the scene is not caught.

Stateful and persistent movies keep their state on purpose, and `check`
does not report this for them; but they should still not keep state in a
string, because seeking back replays ticks over a string that did not
roll back (modes.md).

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
- `check` flags an abrupt change with the frame number and, for a local
  one, where on the canvas (in canvas units). It compares each tile of
  the preview with the same tile in the frame pairs either side, so a
  small element that appears or vanishes in a busy frame is caught, and
  steady fast motion, fades and wipes are not. Counters and typing show up
  as one note, not many warnings. It scans a fixed set of about 230 frames
  of a pure movie (`--all` scans every one: do that before a long
  render). A deliberate cut is fine; ignore the warning for it.

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

## A character does not appear

The text fonts are ASCII. A string containing "π", "→", "≠" or any other
non-ASCII character shows a blank for it, with no error. Use pktype's
`/pi sym`, `/arrow sym`, `/neq sym`, or a run list (`showruns`).

## Single blank frames between sections

One section fades out by its end and the next fades in from its start,
so the frame between them is empty; only `check --all` sees it. Let
something persist across the cut (a rule, a progress bar, a heading
that changes on the cut) or overlap the sections.

## Hairlines between shapes that should touch

N pieces tiling a shape (wedges of a disc, slices of a bar) show faint
seams where their antialiased edges meet, worst in a sheet's small
thumbnails. Stroke each piece in its own colour, or draw the whole
shape underneath the pieces.

## A stray line into an arc

`show` leaves a current point at the end of the text. An `arc` after it
draws a line from there to the arc's start. Begin a new shape with
`newpath`.

## The movie became stateful, or an overlay appeared, by accident

`step` and `overlay` are names the engine looks for: defining a
procedure called `step` makes the movie a simulation and the engine
calls it every tick (usually failing with a stack underflow in your
procedure, "in step 0"). Rename yours.

## `undefined` for a word that sounds right

Words like `placetext-right` or `showcentered` are easy to imagine and do
not exist. `psanim api NAME` answers in a moment, and lists related
words when the name is close.

## Everything after one thing is translucent

`setalpha` stays set for the rest of the frame, like a colour. Wrap a
dimmed drawing in `gsave ... grestore`, or set `1 setalpha` after it.

## Text I drew is invisible

Text drawn before any colour is chosen is black. `ground` sets the
colour to the theme's `/fg`; if you paint your own background, or
change colour for one thing, say `/fg ink` before the next text.
`fitface` and `face` choose a font, never a colour.

## A name of mine broke something far away

`mark`, `count`, `index`, `copy`, `length`, `load`, `step`, `frame` and
`width` are operators or engine words. A procedure of yours with one of
those names changes what the libraries get when they call it, and the
error appears inside some other procedure. Prefix your names
(`markcell`, `nlooks`).
