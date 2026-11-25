# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

## `/fire`: Fire

A flame that loops: stylized, as flat tongues in two or three colours, or realistic, a hot body with glow, flicker, rising embers and faint smoke. Height, width, intensity and wind take it from a candle to a campfire to a torch.

480x600, 4 s, loops. `/x /fire component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Embers | `1` | how many embers rise, 0 for none |
| /Glow | `1` | the halo of light: 0 is a cheap painted one, above 0 a glowing layer (a few ms a frame, more for a larger box); stylized has none |
| /Height | `0.55` | flame height, as a fraction of its box's height |
| /Intensity | `1` | 0.3 a candle .. 1 a fire .. 1.6 a blaze: height, flicker, glow, embers |
| /Palette | `/fire` | /fire /blue /green /violet |
| /Smoke | `0.25` | realistic only: 0 none, 1 a heavy plume |
| /Style | `/realistic` | /realistic or /stylized |
| /Width | `0.42` | flame width, as a fraction of its box's width (up to 0.8; wide means several tongues) |
| /Wind | `0` | -1 .. 1: leans the flame, carries the embers and smoke |

## `/spinner`: Spinner

A loading indicator that loops: a ring with a sweeping arc, a circle of fading dots, or three bouncing bars.

120x120, 1.2 s, loops. `/x /spinner component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | the moving part |
| /Style | `/ring` | /ring /dots /bars |
| /Track | `[1 1 1]` | the ring's faint track |
| /Weight | `10` | stroke width, in its 120-unit square |

