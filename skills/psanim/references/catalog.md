# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

## `/odometer`: Odometer

A number counting up on rolling digit drums: the units wheel spins, each wheel above turns only as the ones below carry, and the digits curve away at the top and bottom of the window.

305x88, 5.1 s. `/x /odometer component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Commas | `true` | mark the thousands |
| /Decimals | `0` | wheels after the point (drawn light on dark) |
| /Digits | `6` | wheels for the whole part |
| /Ease | `/cubic-inout` | how it speeds up and slows down (a tween easing name) |
| /Face | `[0.06 0.06 0.07]` | the wheels |
| /From | `0` | the number at the start |
| /Housing | `[0.2 0.2 0.22]` | the case round them |
| /Ink | `[0.96 0.94 0.86]` | the digits |
| /Over | `3.2` | seconds it takes |
| /Start | `0.4` | seconds before it begins to move |
| /Tail | `1.5` | seconds to hold at the end |
| /To | `482917` | the number it reaches |
| /Track | `null` | instead of From and To: [ [time value] ... ], eased by /Ease between the pairs |
| /Wheel | `[40 60]` | one wheel's window, width and height |

## `/spinner`: Spinner

A loading indicator that loops: a ring with a sweeping arc, a circle of fading dots, or three bouncing bars.

120x120, 1.2 s, loops. `/x /spinner component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | the moving part |
| /Style | `/ring` | /ring /dots /bars |
| /Track | `[1 1 1]` | the ring's faint track |
| /Weight | `10` | stroke width, in its 120-unit square |

## `/splitflap`: Split-flap board

An arrivals board of split-flap cells: rows of text that change on a schedule, each character flipping through the drum to the next, with the bounce and the shadow under the falling flap.

792x296, 9.3 s. `/x /splitflap component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Cell | `[44 64]` | one flap pair, width and height in canvas units |
| /Cols | `0` | characters across; 0 for as many as the longest row |
| /Drum | `( ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:.-/)` | the characters on a cell's drum, in order, the first blank; or an array of such strings, one per column (the last repeats) |
| /Flap | `[0.125 0.125 0.135]` | the flaps |
| /Gap | `4` | between cells |
| /Ink | `[0.96 0.93 0.82]` | the characters |
| /Pad | `10` | round the board |
| /Plate | `[0.035 0.035 0.04]` | the board behind them |
| /Rate | `14` | flaps a second |
| /Scatter | `0.35` | seconds: each cell starts late by up to this |
| /States | `[...]` | [ [time [row row ...]] ... ]: the board from that time on |
| /Tail | `1.5` | seconds to hold after the last flap lands |

