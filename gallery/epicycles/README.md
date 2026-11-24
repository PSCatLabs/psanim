# Epicycles

A mathematical explanation: how a sum of rotating circles draws a square
wave. One circle traces a sine; odd harmonics are added one at a time and
the trace squares up; the terms fan out as a spectrum; then the formula.

## Techniques

- **Pure function of time.** The wave is not a recorded trail: every frame
  recomputes the last two periods from the series itself (`tip`), which is
  why any frame renders alone.
- **A fractional number of terms.** `keyframes` moves the term count
  smoothly from 1 to 9, and the newest circle is scaled in by the
  fractional part, so harmonics grow rather than pop.
- **`reel`** joins four clips with dissolves; each clip's caption fades
  in and out against its own `span`, so two captions never cross.
- **`glowing`** on the trace only; **`staggered`** bars with `/back-out`
  for the spectrum; a mixed-font formula set with a small `runs` helper
  (Times italic with a Symbol pi).

```sh
psanim render gallery/epicycles/epicycles.ps -o epicycles.mp4
```
