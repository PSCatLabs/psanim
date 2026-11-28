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

The structure was chosen by measuring it (see the numbers below), and is
now pkink's `paintplan`, which this film was the model for:

- **Persistent canvas.** The board is painted once, on frame 0.
- **A stroke is kept once, whole, on the step it is finished**
  (`paintkept`). Until then it is regrown from 0 each frame in
  `overlay` (`paintlive`), which is shown and never kept. Chalk is not
  opaque: regrown on the kept canvas, a stroke is laid down again on
  every frame it spans and ends heavier.
- **By step number.** The scene's steps are frames of the declared
  rate. A stroke is kept on the first step whose time is at or after its
  end, not by testing whether the end lies between `now dt sub` and
  `now`: in floating point `now dt sub` is not exactly the previous
  frame's `now`, so that test can be true on two frames and the stroke
  is painted twice (PSCatLabs/psanim#121). The overlay shows a stroke
  until the scene has kept it, counted in the scene's steps
  (`stepnow`): rendered with `--fps`, the overlay runs at the output
  rate while the kept canvas advances at the declared 30, and `fps` and
  the overlay's `frame` and `dt` are the output's (PSCatLabs/psanim#122).
  The first cut of the film did this arithmetic by hand, about 40 lines;
  with `paintplan` it is a plan of dictionaries and two calls. Every one
  of the 1140 frames is byte-identical to the hand-written version's at
  30 fps (and at `--fps` 24 and 60, below).
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

### On `paintplan` (#107)

Measured on the authoring machine (macOS, arm64), load average 3 to 14.

- **The same film.** Rewritten on `paintplan`, every frame of the
  rendered film is identical to the hand-written scheduler's: 1140 of
  1140 at 30 fps, 2280 of 2280 at `--fps 60`, and 911 of 912 at `--fps
  24`; the other (frame 771) differs in 4 pixels by one level, where two
  strokes in progress overlap in the overlay and `paintplan` composites
  each from the canvas's 8-bit pixels.
- **Shorter.** 277 lines to 256; the per-frame part (the schedule, the
  overlay and the stick) from 72 lines to 56, of which the stick is 17
  in both.
- **Cheaper.** The scheduling alone (`dcchalk` replaced by a procedure
  that draws nothing, the stick off, `psanim bench --count 1140`): median
  0.35 ms a frame by hand, 0.22 ms with `paintplan`, p95 0.42 against
  0.25, three runs each. With the chalk, both are about 6.5 ms median
  and 11 ms mean; the differences between runs (6.2 to 7.2 ms) were
  larger than between the two.

## Render

```sh
psanim check  gallery/chalk-lecture/chalk-lecture.ps
psanim frame  gallery/chalk-lecture/chalk-lecture.ps --at 37 -o board.png
psanim frame  gallery/chalk-lecture/chalk-lecture.ps --param oneshot=1 --frame 0 -o oneshot.png
psanim render gallery/chalk-lecture/chalk-lecture.ps -o chalk-lecture.mp4
```
