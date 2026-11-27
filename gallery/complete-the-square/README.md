# Complete the Square

x² + 6x + 5 = 0 solved by completing the square, in 36 seconds (1280x720, 30 fps). Each
line of the derivation becomes the next on screen, and the square that gives the method
its name is drawn beside the algebra.

1. **The question.** `x² + 6x + 5 = 0` fades in, with "Solve for x" under it.
2. **Take 5 from both sides.** The `+ 5` crosses the equals sign and arrives as `−5`; the
   `0` leaves. x² is drawn as a square, and 6x as two strips, 3 by x.
3. **Add 9 to both sides.** Two `+ 9`s arrive, and at the same moment the missing corner,
   3 by 3, fills the figure in.
4. **One square.** `x² + 6x + 9` regroups as `(x + 3)²` and `−5 + 9` becomes `4`, while
   the outline of the whole square, of side x + 3, is drawn round the figure.
5. **Square root, then take 3.** `x + 3 = ±2`, then `x = −3 ± 2`: the `+ 3` crosses the
   equals sign and becomes `−3`.
6. **Two answers.** `x = −1, x = −5`, each ringed, held for three seconds.

## Techniques

- **`morphruns`.** The whole derivation is seven run lists and one progress number. The
  number comes from `keyframes`, built in setup from the start times of the six moves, so
  each line holds and each move eases (`/cubic-inout`) without a time being written twice.
- **Pairing.** Terms that carry over are paired by content, in order. Terms that change on
  the way are tagged: `[/as /five ( + 5)]` in one line and `[/as /five [/minus (5)]]` in
  the next make one term that slides and cross-fades. A group (`[/minus (5)]`) moves as one
  term.
- **An aligned derivation.** Every line tags its first equals sign `/eq`, and `/eq` as the
  alignment keeps it still while everything else moves round it.
- **The figure** is pkstage boxes and pkmotion entrances (`/pop`, `/wipe-right`,
  `/mask-up`); its corner enters with the equation's `+ 9`s, the outline is drawn on with
  `trimpath`, and the figure `leaves` before the square root. The equation glides up to
  make room for it, and back, by a second `keyframes` track.
- The answers are ringed with pkdiagram's `circled` round `runbox`es of the last line.

## Commands

```sh
psanim check  gallery/complete-the-square/complete-the-square.ps --all
psanim sheet  gallery/complete-the-square/complete-the-square.ps -o out/complete-the-square/sheet.png
psanim frame  gallery/complete-the-square/complete-the-square.ps --at 33 -o out/complete-the-square/poster.png
psanim render gallery/complete-the-square/complete-the-square.ps -o out/complete-the-square/complete-the-square.mp4
```
