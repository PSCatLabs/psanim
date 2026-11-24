# Tidewater

A watercolour that paints itself: an estuary at low tide just after sunrise.
Paper, a warm wash pulled through sky and water, rose and cobalt laid into
it, the sun lifted out, clouds, the far shore, the mudflats that leave the
channel as bare reflected sky, then the darks: a grounded boat, withies,
birds, reeds, a dry-brush glitter and a signature. 24 s, 1280x720, 24 fps,
ending on a 2.3 s hold.

## How it works

- **`/Persist true`** keeps the canvas between frames, so a frame paints
  only what is new. Nothing is ever redrawn.
- **The plan.** Setup builds `plan`, an ordered array of stroke records
  `[kind t0 dur ease a b c seed proc]`. The words `sweep`, `bloom` and
  `rows` write records, so the plan reads top to bottom in painting order.
- **Travelling strokes.** `scene` runs `paintdue` on every record. For a
  stroke in progress it computes where the brush front was at `frame - 1`
  and where it is at `frame` (`frame`, `fps`, `ease`, `clamp`), clips to
  the sliver between the two, and runs the stroke's whole paint procedure.
  The slivers are disjoint and tile the stroke, so a translucent wash
  never doubles up. `sweep` moves a bellied front across a box; `bloom`
  grows a ring from a point (the sun's glow, the lift); `rows` lays a big
  wash as level passes that alternate direction and share a wandering
  edge (`rowclip`).
- **Determinism.** Each stroke `srand`s its own seed before painting, so
  it lays identical texture in every frame it spans. A frame rendered cold
  is byte-identical to the same frame reached in order.

## Paintkit words

| Word | Used for |
|---|---|
| `pkpaper` | the ground, once, on frame 0 |
| `pkwash` | every wash: graded sky (`graded` stacks thin washes), clouds, shores, mud with `/Bloom` rims and `/Grain`, lifted lights |
| `pkdry` | dry-brush edges of the mud, drags across the bank, the glitter |
| `pkliner` | mast and stay, withies and their reflections, birds, reeds, flourish |

Everything is clipped to the painted block (`rectclip`), which gives the
taped edge and the paper margin.

## Render

```sh
psanim check  gallery/tidewater/tidewater.ps
psanim sheet  gallery/tidewater/tidewater.ps -o out/tidewater/sheet.png
psanim render gallery/tidewater/tidewater.ps -o out/tidewater/tidewater.mp4
```

About 75 ms per frame at full size on one thread (a persistent movie
renders in order); the full film takes under a minute.
