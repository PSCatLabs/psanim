# Midnight

A morphing-digit clock, 70 seconds, 1280x720, 30 fps. It starts at
23:58:57, so there are two events to watch: the minute turns at 3 seconds
(23:58:59 to 23:59:00, three digits melt) and the day turns at 63 seconds
(23:59:59 to 00:00:00, all six digits melt at once, and a ring opens from
the clock). Each digit finishes forming on the stroke of the second.

After the classic X Windows morphing-digit clock; the outlines are our own.

## It never reads a clock

A movie is a pure function of its own time, so it cannot ask what time it
is. The film is *given* a time: `/Start (23:58:57) param` (try
`psanim render ... --param Start=11:59:50`) plus its own time, which is
what `start null daytime` computes. The same two numbers always give the
same film. A live page would instead supply a signal (see
`skills/psanim/references/modes.md`, "Time of day"); the clock entry takes
either.

## Techniques

- The clock is the `/clock-morph` catalog entry, placed as a component with
  its own parameters (`Panel 0` leaves the digits on this film's sky). Its
  digits are ten pen paths turned into outlines with `strokepath`, each
  one closed contour even for 0, 6, 8 and 9 (the pen path of a 0 starts and
  ends at one point, so its outline is cut by a slit of zero width and
  fills as a ring). `morphpath` carries one outline into another, and
  `alignpath` turns the target to start where it moves least, once, in
  setup.
- The film reads the same time the clock does (`tod`) for the seconds
  hairline and for the rings that open at each minute and at midnight.
- The sky is 48 flat bands; the stars are `hash01` points that twinkle with
  `wiggle`.

About 3.6 ms a frame at full size, median (the whole 2100 frames render in about 15
seconds on ten threads).

## Render

```sh
psanim check  gallery/morph-clock/morph-clock.ps
psanim sheet  gallery/morph-clock/morph-clock.ps -o sheet.png
psanim render gallery/morph-clock/morph-clock.ps -o morph-clock.mp4
```
