# Ridgelines

Five ranges of hills at dawn, 16 s at 1280x720, 30 fps.

Each range is forty overlapping peaks. For the first seconds they are
painted the obvious way, each peak with its own `setalpha`, and every
place two peaks overlap shows as a darker seam. Then, far range first,
the seams heal: each range becomes one transparency group, its peaks
opaque inside it, the whole range composited once at one alpha, so the
sky and the ranges behind show through it evenly. Fog banks of
overlapping puffs drift between the ranges in the same way. The sun
comes up through the far range, the sky warms, the drift slows to a stop
and the title rises and holds.

The movie is pure: every frame is a function of its time.

## Techniques

| Word | What it does here |
|---|---|
| `begingroup` / `endgroup` (PSCat) | Each range and each fog bank is one group with an `/Alpha`: the overlaps inside it cannot darken. |
| `setalpha` inside a group | The heal. Peaks at alpha m inside a group at alpha a/m; m goes from a (each peak on its own: seams) to 1 (one group: none), so a single peak keeps the same strength throughout. |
| `faded` inside a group | Fades each fog bank in. Inside a group psanim runs `faded` as a group of its own (a layer there would be lost). |
| `enters` with `/rise` around a group | The title and its line are one group inside the motion's body. The other way round, a motion inside a group, is refused while it moves. |
| `glowing` | The sun, outside any group. |
| `hash01` | Each peak's place, height and width, stable from frame to frame. |
| `tween`, `between` | The heal, the dawn, the sun's climb, the drift slowing to rest. |
| `shfill` (PostScript) | The sky, one axial shading. |

## Why a group, and not `faded`

In psanim's raster output, `a { ... } faded` around a range gives the
same picture: the group and the layer agree to one level in 255 (the
group composites at 16 bits, the layer at 8; `crates/psanim/tests/groups.rs`
measures it). The group is PSCat's own construct, part of the drawing
rather than a canvas swap in the host, and it nests inside other groups
and in places psanim's layers cannot go. It is also slower per frame
here (see `docs/benchmarks.md`). What a group cannot have inside it is a
psanim layer: `blurred`, `glowing`, a moving motion and the pixel effects
refuse to run in one (`groupnesting`), so the sun's glow is drawn
outside the ranges' groups.

## Render

```sh
psanim check  gallery/ridgelines/ridgelines.ps --all
psanim sheet  gallery/ridgelines/ridgelines.ps -o out/ridgelines/sheet.png
psanim render gallery/ridgelines/ridgelines.ps -o out/ridgelines/ridgelines.mp4
```
