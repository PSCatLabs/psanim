# Twenty-Three

A 67 second data story (1280x720, 30 fps) about the birthday problem: how
many people must be in a room before two of them probably share a
birthday.

1. **The question**, asked the way a person would.
2. **The guess**: 365 days, so about half of them: a straight line that
   reaches 50% at 183 people.
3. **The truth**: the exact curve arrives on the same axes and passes 50%
   at 23.
4. **The surprise, closer in**: the x axis tweens from 0..365 to 0..80; a
   counter walks up the curve room by room and stops at 23 (50.7%) while
   the rest of the chart is dimmed; then on to 57 (99.0%).
5. **Why**: 23 dots, and every pair of them drawn as a line: 253. A bar
   chart of people (5, 10, 15, 20, 23) turns into the pairs among them
   (10, 45, 105, 190, 253).
6. **A check**: 100,000 simulated rooms of 23; 50,686 had a match.
7. **The takeaway**: coincidences count pairs, not people.

## Where every number comes from

All of them are printed by `data.py` (Python standard library, exact
rational arithmetic, fixed seed). Model: 365 equally likely birthdays, no
leap day.

| On screen | Computation |
|---|---|
| the curve; 50.7% at 23; 47.6% at 22; 99.0% at 57; every counter reading | `1 - prod((365-k)/365 for k < n)`, n = 0..100, as fractions; `/bd-exact` in the film is this table in percent. For n > 100 the curve is drawn at 100 (it is above 99.99997%) |
| 183 (the guess) | first n with n/365 >= 1/2 |
| "the guess said 6.3%" | 23/365 |
| 253 pairs; 10, 45, 105, 190 | `comb(n, 2)` |
| 22 (the first person's lines in the ring) | 23 - 1 |
| 50.05% | `1 - (364/365)**253`: the 253 pairs treated as independent |
| "at least 308 days unused" | 365 - 57 |
| 100,000 rooms, 50,686 matches, 50.7% measured; 45 of the first 100; the lit squares | `random.Random(23)`, 23 `randrange(365)` draws per room; `/bd-rooms` is the first 100 outcomes |

The counters for rooms and matches in the simulation scene run on one
tween, so the pair on screen is an animation toward the measured totals,
not a running tally.

## Techniques

- `storyboard` with seven named beats; `inbeat` per scene; headlines on
  the film's clock with `onstage` (`/mask-up` in, `/fade` out).
- pkchart: `plotframe` whose x range is a `between` of two ranges (the
  zoom), `plotaxes`, `plotline` with `trimpath` for the draw-on,
  `plotpoint` for every mark, `bars` between two datasets with the y
  range interpolated alongside.
- pkdiagram `note` for labels with leaders; pkstage `spotlight` for the
  dimming at 23; `keyframes` driving the counter that reads the exact
  table; `fmtnum` and `showright` for steady numbers.
- Hand-drawn: axis titles (to keep the y title horizontal, above the axis), and where the libraries have no word, dashed
  reference and drop lines, clipping the lines to the plot, the ring of
  253 chords, the 10 by 10 grid of rooms (`cells` + `cascade`).

## Commands

```sh
python3 data.py                                   # the numbers and the data block
psanim check  birthday.ps --all
psanim sheet  birthday.ps -o sheet.png
psanim frame  birthday.ps --at 35 -o poster.png
psanim render birthday.ps -o birthday.mp4
```
