# Chalk Proof

A 37 second film (1280x720, 30 fps, 1110 frames): Pythagoras proved on a
green chalkboard. A square of side a + b is drawn, a tilted square of
side c inside it, and the four triangles left over are shaded. Then the
area is written two ways, 2ab is struck from both sides, and the result
is boxed. A stick of chalk follows the writing.

## PSCat's chalk, animated

The board, the chalk and the formulas come from PSCat's libraries,
unchanged: `dcboard` (the green board with its haze, eraser swipes and
last lecture's ghost), `dcchalk` (a stroke that lands heavy, tires and
sheds dust), `sgpath` (single-stroke handwriting) and `mkpath` /
`mkalign` (a TeX subset laid out as stroke paths). What psanim adds is
time:

- **Persistent canvas.** The board is painted once, on frame 0. Each
  frame adds only the chalk laid down since the last one, so each
  stroke is paid for about once.
- **Strokes as data.** In setup every path is split into its subpaths,
  one per stroke, measured with `pathlength`, and scheduled end to end
  at a writing speed with a 0.05 s lift between strokes.
- **A stroke across frames.** A stroke that is partly written in a
  frame is drawn whole, clipped to a fat outline (`trimpath`,
  `strokepath clip`) of the part covered in that frame. `dcchalk` is
  seeded per stroke, so the pieces drawn by successive frames are parts
  of the same stroke and join exactly.
- **`overlay`** draws the stick of chalk at the writing point, in the
  colour in use, moving to the next stroke during a lift. It is shown
  and never kept.

Chalk is expensive (it is thousands of small fills), and the canvas
persists, so the film renders in order on one thread: about a minute.

## Render

```sh
psanim check  gallery/chalk-proof/chalk-proof.ps
psanim frame  gallery/chalk-proof/chalk-proof.ps --at 36.5 -o board.png
psanim render gallery/chalk-proof/chalk-proof.ps -o chalk-proof.mp4
```
