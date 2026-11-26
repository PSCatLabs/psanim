# Simple Rules

A 58 second film (1280x720, 30 fps, pure) about cellular automata: simple
local rules that grow into structure nobody drew. Three movements, each a
rule that really runs.

1. **Rule 30** (0 to 23 s). One live cell. Each row below is made from
   the three cells above it; the rule's eight cases are on screen while
   the first ten rows arrive. Then the rows come faster and the camera
   pulls back to keep the whole triangle in view, until all 300
   generations are on screen: regular stripes down its left side, chaos
   on its right.
2. **The Game of Life** (23 to 38 s). Gosper's glider gun fires a glider
   every thirty generations, at 18 generations a second; cells that die
   fade from ember to ash, so the motion reads at film speed, and the
   camera eases back from the gun.
3. **Methuselahs** (37.5 s to the end). Diehard, the R-pentomino and the
   acorn side by side, each starting at four generations a second and
   gathering speed so that all three reach their known ends together:
   diehard is gone after 130 generations, the R-pentomino settles at
   1103, the acorn at 5206. The fields keep a ghost of everywhere life
   has been, with the tracks of the gliders that flew away. It ends on
   that hold.

## How it is built

Nothing is keyframed but the cameras. The cells come from the native
cellular automaton words:

- `cellrows` gives Rule 30's whole history (301 rows of 601 cells) once,
  in setup; `cellcolors` colours it row by row (gold, coral, violet), and
  the scene shows as many rows as the clock has reached, as one
  `colorimage` cropped to what the camera sees. Below 4 pixels a cell the
  image is drawn with `/Interpolate true` (bilinear), so a cell 1.85
  pixels wide is blended at its edges instead of being snapped to one or
  two whole pixels.
- The rows arrive two a second for the first ten; then the log of the
  row count follows one cubic that leaves at that rate and arrives at
  row 300 at rest. The camera never scrolls: the apex stays put and the
  view pulls back to fit the rows, so however fast they come a cell
  drifts a few pixels a frame (an earlier version followed the newest
  row down and strobed). Close up, cells are drawn as rectangles, one
  fill per run of live cells, so the slow zoom moves them smoothly
  rather than snapping to whole pixels as an image does; from 4 pixels
  a cell down, one image.
- Movements 2 and 3 place the catalog's `/automaton` entry: one instance
  for the gun, three for the methuselahs. Each has its own `/Clock`, a
  procedure from its time to its generation (for the methuselahs an
  exponential ramp, `gen = a (e^(bt) - 1)`, with `b` found by bisection
  in setup so that each reaches its end at the same moment). Inside the
  entry, `cellsteps` gives generation n of the starting grid; it is a pure
  function of n, and it remembers the states it computed, so playing
  forward costs one generation a frame.
- The methuselahs run on grids of 300 x 380 cells with `/Edge /open`: a
  lone glider is taken off as it nears the edge, so the grid behaves as
  the open plane. A dead edge would turn each glider into a block, and
  those blocks change the acorn's history. The test suite checks the
  result: the acorn on this grid has 568 cells at generation 5206, the
  plane's 633 less its 13 gliders.
- Each methuselah's camera frames `cellbounds`, the box of everywhere
  life has been (leaving out the thin tracks of gliders), between this
  generation and the next. The film runs the same grid as the instance
  to get it; the two share the cache.

Change a pattern (`/m-r /rpentomino ...` to `/pulsar`, or a picture of
your own) and the film is different and still right.

## Things worth knowing

- Measured on an Apple M4 with `check --all` (every frame, in order): a
  mean of 19.8 ms a frame at full size (slowest 76.6 ms) and 2.6 ms at
  preview size. A frame reached cold costs every generation before it:
  `psanim frame --at 55` runs the acorn's 5206 generations on 114,000
  cells first, about 0.6 s in all; the default `check`, which samples
  frames out of order, averages 32 ms a frame at preview size.
- `check` is clean, and so is `check --all` at `--scale 1` and
  `--scale 0.5`. At its default quarter-size preview, `check --all` warns
  at 15.1 and 18.7 s: there a cell is under half a preview pixel, and
  PSCat point-samples a shrunk image, so the slow zoom aliases in the
  preview only.

## Render

```sh
psanim check  gallery/automata/automata.ps
psanim sheet  gallery/automata/automata.ps -o out/automata/sheet.png
psanim render gallery/automata/automata.ps -o out/automata/automata.mp4
```
