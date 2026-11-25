# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

## `/ledmatrix`: LED matrix

A grid of round LEDs with a pixel font of its own: a scrolling marquee, a ticker, a news sign with entry effects and a clock, or any pattern a procedure computes. Raw mode gives the grid at true size for driving real LEDs.

600x108, 3.63 s, loops. `/x /ledmatrix component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Between | `0.4` | seconds of dark between messages |
| /Cell | `null` | a procedure { col row -- level }: a pattern drawn under the text |
| /Clock | `[9 41]` | the time /time shows when the sign starts: hour minute |
| /ClockRate | `4` | seconds for the /time clock to advance a minute |
| /Colour | `/amber` | /red /amber /green /white /blue, or [r g b] |
| /Cols | `0` | LEDs across, instead of /Grid's (for --param, which cannot give an array) |
| /Effect | `/marquee` | /scroll /marquee /static /drop /wipe /sparkle /flash /fade |
| /Font | `0` | rows of the pixel font, 5 or 7 (0: by the grid) |
| /Gap | `16` | blank LEDs between one marquee round and the next |
| /Glow | `1` | halo round lit LEDs, 0 for none |
| /Grid | `[48 7]` | LEDs across and down |
| /Hold | `2` | seconds a message stays before it leaves (not for /scroll or /marquee) |
| /Items | `null` | instead of Text: [ [ text effect hold colour ] ... ], shown in turn (text may be /time or /temp) |
| /Length | `0` | seconds the entry lasts; 0 follows the messages (4 for a pattern with no text) |
| /Look | `/panel` | /panel (a lit panel), /sign (smoked glass in a housing) or /bare (the lit LEDs only, to lay over a picture) |
| /Pitch | `12` | canvas units from one LED to the next |
| /Raw | `false` | true: the movie is the grid itself, one canvas unit an LED, no glow |
| /Repeat | `true` | start the list again at the end |
| /Rows | `0` | LEDs down, likewise |
| /Speed | `30` | LEDs a second for /scroll and /marquee: a divisor of the frame rate keeps it even |
| /Temp | `68` | degrees for /temp |
| /Text | `null` | what to show: a string, or a picture (array of row strings of # and .); a demo message if none and no /Cell |
| /Unlit | `0.1` | how bright an unlit LED is |

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
| /Track | `null` | instead of From and To: a keyframes list [ t v t v /ease ... ] |
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

