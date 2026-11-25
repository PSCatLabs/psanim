# Canon

A 12 second loop (1294x800, a golden rectangle; 30 fps): one short scene
that contains itself. Each copy sits in the golden rectangle left beside
the one before it, a quarter turn round, in the next theme, and three
quarters of a second late. Seven voices, spiralling inward, singing the
same line.

## What it is for

It is the smallest demonstration of the idea the pk libraries are built
on: **a scene is a function of the things a caller can re-base.** psanim's
core re-bases the clock (`delayed`, `during`, ...). The libraries add the
canvas (`within`: `width`, `height`, `vmin` and `canvasbox` become the
box's) and the look (`usetheme`: colours and fonts by role). `voice` is
written against all three, so the same sixty lines can be placed in any
box, at any delay, in any theme, including inside themselves:

```postscript partial
/voice {
  voices level get usetheme
  ground  figure
  level voices length 1 sub lt {
    /level level 1 add def
    gsave width 0 translate 90 rotate
      lag { [ 0 0 height width height sub ] { voice } within } delayed
    grestore
    /level level 1 sub def
  } if
} def
```

## Techniques

- `within` under `rotate`: the next voice's canvas is the leftover
  rectangle turned a quarter turn, which is again a golden rectangle.
- `around` for the twelve petals and `brushstroke` (a tapered stroke) for
  each petal and for the runner's wake.
- A seamless loop under delay. `turns` measures against the current time
  window, which `delayed` shortens, so the piece defines `laps` against
  the film's `duration` instead: every late voice then closes too.
  `psanim check` confirms the join.

About 4 ms a frame at full size.

## Render

```sh
psanim check  gallery/canon/canon.ps
psanim sheet  gallery/canon/canon.ps --seam 6 -o seam.png
psanim render gallery/canon/canon.ps -o canon.mp4
```
