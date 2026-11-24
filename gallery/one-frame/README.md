# One Frame, Any Time

A technical presentation: how psanim renders a movie, explained by a
movie. A movie is a program; setup runs once and memory is checkpointed;
every frame rolls back, sets the clock and draws; so frames can be made
in any order and shared among cores.

## Techniques

- **The idea drawn with itself.** `mini` is a tiny movie (a turning
  square) drawn into any cell of the diagram *at whatever time that cell
  stands for*. Forty-eight cells in shuffled order all show the right
  frame because the picture depends on its time alone.
- **`reel`** of six clips with dissolves; **`looped`** runs the
  restore/clock/draw cycle five times, and `between` on points flies each
  finished frame into the film strip.
- **`typed`** reveals the code listing line by line; **`trimpath`** draws
  the arrows on; `hash01` picks the shuffled order and which memory cells
  a frame dirties.
- Under a millisecond per frame: no layers except short fades.

```sh
psanim render gallery/one-frame/one-frame.ps -o one-frame.mp4
```
