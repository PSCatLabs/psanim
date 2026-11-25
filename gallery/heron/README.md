# Heron

A haiku in ink and wash, 46 seconds, 1280x720, 30 fps.

> morning mist lifting / a heron waits in the reeds / for the sun to move

One sheet of paper is painted stroke by stroke: two hills in watercolour
wash, mist soaking in over their feet, dry-brush water, reeds in a liner
brush, a heron in tapered brush ink. The turn is the one accent: a
vermilion sun that soaks in pass over pass. The ending is motion graphics:
a vector ring leaves the painted sun and crosses the sky on a dotted arc
with a running clock, while the painting (and the heron) stay put.

## Why a persistent canvas with `overlay`

The picture only accumulates, and painted media cost tens of milliseconds
a stroke: as a pure movie every frame would repaint every finished wash.
With `/Persist true` each frame paints only the slivers the brushes
crossed during that frame. Everything that must move or stay crisp (the
frame rule, the kinetic haiku, the accent tick, the brush hairline and pen
ring, the sun ring, its arc and clock, the title) is in `overlay`, which
is drawn as a pure scene and never kept.

## Techniques

- One `plan` array of strokes: `[start dur seed /kind clipdata {body}]`,
  three ways to reveal a whole-stroke body over time:
  - `/sweep`: a wash revealed by a front moving in x; the sliver is
    snapped to even units so the translucent slivers tile without seams.
  - `/pen`: an opaque mark revealed along its path with pkink's
    `revealalong` (paintkit `pkliner`, `pkdry`, pkink `brushstroke`).
  - `/soak`: a wash laid as N faint passes, one every few frames (the
    sun growing from its centre, the mist drifting right to left).
- PSCat media: `pkpaper`, `pkwash`, `pkdry`, `pkliner`; pkink
  `brushstroke`, `revealalong`.
- Overlay: pktype `kinetic` (words, `/mask-up`), pkmotion `enters`,
  `keyframes`, `trimpath`, `pathpoint`, `eachspaced`; the overlay reads
  the same `plan` to put a hairline or ring at each working brush.

## Render

```sh
psanim check  gallery/heron/heron.ps
psanim sheet  gallery/heron/heron.ps --scale 0.5 -o out/heron/sheet.png
psanim render gallery/heron/heron.ps -o out/heron/heron.mp4
```

Measured on the authoring machine (macOS, one thread, as persistent
movies render): 1380 frames in 17.7 s, about 13 ms per frame, 1.2 MB MP4.
