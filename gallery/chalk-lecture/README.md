# Chalk Lecture

A 38 second film (1280x720, 30 fps, 1140 frames): the simple pendulum
worked on a green board. A support, a string and a bob are drawn, the
angle and the length; then the weight on the bob, and the part of it
along the arc, `mg sin θ`. Beside the diagram, Newton's law along the
arc, the small-angle step, the harmonic equation and its solution, and
the period `T = 2π √(L/g)`, boxed, with what it gives for a metre of
string (`T ≈ 2 s`). The writing takes 33 seconds; the finished board
holds for the last four. A stick of chalk follows the writing.

## Strokes that grow

Every stroke is drawn the way a hand draws it, from its start, with
PSCat's `/From 0 /To u` on `dcchalk`. Since PSCat 0.8.0 a dry-media
stroke grown that way changes no pixel behind its tip, so the growing
line does not flicker. [Chalk Proof](../chalk-proof/) paints
each stroke whole and reveals it through a moving clip; here the chalk
itself grows: the stroke lands, tires and lifts where it would.

The structure was chosen by measuring it (see the numbers below):

- **Persistent canvas.** The board is painted once, on frame 0.
- **A stroke is kept once, whole, on the frame it is finished.** Until
  then it is regrown from 0 each frame in `overlay`, which is shown and
  never kept. Chalk is not opaque: regrown on the kept canvas, a stroke
  is laid down again on every frame it spans and ends heavier.
- **The commit is by step number.** The scene's steps are frames of
  the declared rate, named once in the program (`rate`). A stroke is
  kept on step `t1 rate mul 1e-6 sub ceiling cvi`: the first whose time
  is at or after its end (an end within a millionth of a step after a
  step's time counts as that step's). Not by testing whether the end
  lies between `now dt sub` and `now`: in floating point `now dt sub`
  is not exactly the previous frame's `now`, so that test can be true
  on two frames and the stroke is painted twice (PSCatLabs/psanim#121).
- **The overlay asks whether the scene has kept a stroke yet**, by the
  scene's steps: the last step run when `now` is shown is `now rate mul
  1e-9 add floor cvi`. Until it reaches the stroke's step, the overlay
  draws the stroke, at most whole. Rendered with `--fps`, the overlay
  runs at the output rate while the kept canvas advances at the
  declared 30, and `fps`, the overlay's `frame` and `dt`, and `dt` in
  setup are the output's (PSCatLabs/psanim#122); a rule built on them
  either keeps a stroke early or lets it vanish for a frame. Checked at
  `--fps` 24, 25, 29.97, 60 and 120 (the CLI takes a decimal, not
  30000/1001), rendering the 0.45 s around the end of eight strokes
  spread through the film (the first, the last, which ends exactly on a
  step, and six between) with the stick left out (`--param nostick=1`):
  the chalk in the stroke's area never goes down from one output frame to
  the next beyond texture noise (at most 0.03% of the stroke's ink, at
  120). The earlier rule, built on the overlay's `frame` and `dt`, lost
  19 to 26% of it for a frame on the four strokes tried at 60.
- **Strokes as data**, as in Chalk Proof: in setup every path (lettering
  from `sgpath`, formulas from `mkpath` and `mkalign`, the diagram) is
  split into its subpaths, one stroke each, because `/From` and `/To`
  cut every subpath of a path at once. Each stroke takes a moment plus
  its length over a speed; the chalk is off the board for a moment plus
  the distance to the next stroke. The plan is then scaled to fit the
  writing into 33 seconds.
- `dcchalk` seeds a stroke from its position on the board, so a stroke
  drawn on its own is the same stroke as in a whole board drawn at once.

## Measured

On the authoring machine (macOS, arm64), which was heavily
loaded by other work at the time (load average 10 to 40): treat these
as loose. The first check was rerun after the diagram was enlarged and
the algebra moved left; the other numbers are from the first cut of the
board, which had one stroke fewer.

- **The last frame against the same board painted in one go** (`--param
  oneshot=1` paints every stroke whole on frame 0): identical, 0 of
  921,600 pixels differ, at the declared 30 fps and with `--fps 24` and
  `--fps 60`.
- **The same film with the stroke in progress regrown in `scene`**, on
  the kept canvas: 13,648 pixels differ, by up to 63 levels, and the
  chalk carries 5.8% more ink than the board painted in one go.
- **Built from incremental pieces** (`/From` the last frame's `u`
  `/To` this frame's, on the kept canvas): 6,312 pixels differ, by up
  to 55 levels, at 0.998 of the ink. At the default touch that is close;
  at a lighter touch it is not: PSCat measured 1.09 to 1.32 times the
  ink at `/Touch 0.5` (50 to 300 pieces) and about 1.85 at `/Touch 0.3`
  (300 pieces) (PSCatLabs/pscat#1487).
- **Cost per frame** (`psanim bench --count 1140`, every frame in
  order, 1280x720): median 6.4 ms, mean 10.8 ms, p95 39 ms; frame 0
  (the board) about half a second. The cost does not grow as the board
  fills: by section (`psanim check --all --scale 1`), 19 ms mean while
  the diagram is drawn, 9 ms through the first equations, 21 ms for the large formula and the box, 1 ms in
  the hold. Regrowing in `scene` instead measured a mean of 19.9 ms.
- **A pure movie** would repaint the board and every finished stroke
  each frame: the finished board alone measured 7.3 s.
- **Render** (`NOGIF=1 scripts/build_gallery.sh chalk-lecture`): 1140
  frames in 18.8 s and 16.7 s in two runs, on one thread (persistent
  movies render in order), 0.8 MB MP4.

## Render

```sh
psanim check  gallery/chalk-lecture/chalk-lecture.ps
psanim frame  gallery/chalk-lecture/chalk-lecture.ps --at 37 -o board.png
psanim frame  gallery/chalk-lecture/chalk-lecture.ps --param oneshot=1 --frame 0 -o oneshot.png
psanim render gallery/chalk-lecture/chalk-lecture.ps -o chalk-lecture.mp4
```
