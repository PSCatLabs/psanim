# Paper Boat

A wordless short film: 41.6 s, 1280x720, 24 fps, one PostScript program.

A sheet of paper folds itself into a boat on a table by a rainy window. A
child in a yellow raincoat sets it on the gutter stream. It rides through
the town at dusk; the weather turns, the street tips downhill, and the
drain takes it. In the dark it drops, steadies, is watched by a pair of
eyes, and finds the way out: a wide calm river, the rain leaving, the town
behind. The camera rises into the stars and finds the boat there too.

## Scenes

| Time (s) | Clip | What happens | Leaves by |
|---|---|---|---|
| 0.0 - 6.0 | `c-fold` | Table from above, rain shadows in the window light; sheet, half, hat, boat | dissolve, 1.0 s |
| 5.0 - 11.5 | `c-launch` | The child lowers the boat to the gutter, lets go, waves | `t-follow`: a rain-slanted wipe moving the way the boat goes, 0.9 s |
| 10.6 - 25.1 | `c-town` | One travelling shot: lamps, a passer-by, dusk sliding into storm, downhill to the drain | `t-drain`: an iris closing on the drain mouth, 1.3 s |
| 23.8 - 30.8 | `c-dark` | Pipe rings slide by, the boat tumbles in, eyes, the way out grows | dissolve, 2.2 s (the two boats are registered) |
| 28.6 - 41.6 | `c-river` | Pull back over the river, rise to the stars, the constellation, "fin" | fade to black |

## Techniques

- `reel` with `reelduration` for the length; two custom transitions put in
  the `transitions` dictionary (`t-follow`, `t-drain`). The iris reads the
  drain's position from the clip (`transform` into `/iris-dev`), so it
  stays on the drain while the camera is still moving.
- Boat, water, rain, ripples, houses, lamps are procedures against `now`.
  One wave function (`wave2`) draws every water surface and gives the
  boat its height and tilt (`floatpose`).
- `camera` per depth layer (`pcam`) for parallax, zoom, and the downhill
  roll; `keyframes` for every move.
- Palettes are dictionaries; `mixpal` blends dusk into storm with
  `between` on every colour at once.
- `morphpath` over four resampled outlines for the fold; `trimpath` for
  the constellation and the shooting star.
- `glowing` (windows, lamps, moon, eyes), `blurred` (far roofs, lamp
  cones, lamp pool), `faded` (groups, reflections), `vignette`,
  `filmgrain`; `hash01` and `wiggle` for everything random.
- Pure movie: no `step`, no Persist. About 32 ms/frame at full size
  (worst 66 ms).

## Render

```sh
psanim check  gallery/paper-boat/paper-boat.ps
psanim sheet  gallery/paper-boat/paper-boat.ps --count 24 -o out/paper-boat/sheet.png
psanim render gallery/paper-boat/paper-boat.ps -o out/paper-boat/paper-boat.mp4
```
