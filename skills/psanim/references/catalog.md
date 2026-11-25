# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

## `/clock-analog`: Analog clock

A round clock face with tapered hour and minute hands and a red second hand that either ticks with a little overshoot, like quartz, or sweeps. Themes, optional numerals. Time of day from a start time and the movie's own time, or from a host signal.

640x640, 86400 s. `/x /clock-analog component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Numerals | `1` | 1 writes the twelve numerals, 0 leaves only the ticks |
| /Panel | `1` | 1 paints the face and bezel, 0 draws only the ticks and hands on whatever is there |
| /Second | `/tick` | the second hand: /tick (jumps once a second, with a little overshoot), /sweep (glides), /none |
| /Signal | `null` | name of a host signal that carries seconds since midnight (a live page); Start is then ignored |
| /Start | `(22:10:35)` | time of day at movie time 0: seconds since midnight, or (HH:MM:SS) text |
| /Theme | `/midnight` | /midnight /amber /paper /phosphor |

## `/clock-digital`: Digital clock

A seven-segment clock with the unlit segments faintly visible, a blinking colon, and a choice of 12 or 24 hours and of themes. Time of day from a start time and the movie's own time, or from a host signal.

960x320, 86400 s. `/x /clock-digital component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Blink | `1` | 1 blinks the colons once a second, 0 keeps them lit |
| /Hours | `24` | 24, or 12 (no leading zero, 12 at midnight) |
| /Panel | `1` | 1 paints a rounded panel behind the digits, 0 leaves them on whatever is there |
| /Seconds | `1` | 1 shows HH:MM:SS, 0 shows HH:MM |
| /Signal | `null` | name of a host signal that carries seconds since midnight (a live page); Start is then ignored |
| /Slant | `0.1` | lean of the digits (0 upright) |
| /Start | `(00:34:56)` | time of day at movie time 0: seconds since midnight, or (HH:MM:SS) text |
| /Theme | `/midnight` | /midnight /amber /paper /phosphor |

## `/clock-ledsign`: LED sign clock

The time on a scrolling LED sign: a 5 by 7 dot matrix with unlit dots faintly visible, glowing lit dots, and the time scrolling by a column at a time. Time of day from a start time and the movie's own time, or from a host signal.

960x160, 86400 s. `/x /clock-ledsign component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Gap | `12` | blank dot columns between one showing of the time and the next |
| /Panel | `1` | 1 paints the sign's dark housing, 0 leaves the dots on whatever is there |
| /Scroll | `14` | dot columns per second the text moves left (0 holds it still) |
| /Seconds | `1` | 1 shows HH:MM:SS, 0 shows HH:MM |
| /Signal | `null` | name of a host signal that carries seconds since midnight (a live page); Start is then ignored |
| /Start | `(00:34:56)` | time of day at movie time 0: seconds since midnight, or (HH:MM:SS) text |
| /Theme | `/amber` | /amber /midnight /paper /phosphor |

## `/clock-morph`: Morphing clock

A clock whose digits melt from one into the next, after the classic X Windows morphing-digit clock. Time of day from a start time and the movie's own time, or from a host signal. Not a loop: the movie time is the clock.

960x320, 86400 s. `/x /clock-morph component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Blink | `0` | 1 dims the colons for the second half of each second |
| /Melt | `0.6` | seconds (at most 1) a digit takes to melt into the next, finishing on the stroke of the second |
| /Panel | `1` | 1 paints a rounded panel behind the digits, 0 leaves them on whatever is there |
| /Seconds | `1` | 1 shows HH:MM:SS, 0 shows HH:MM |
| /Signal | `null` | name of a host signal that carries seconds since midnight (a live page); Start is then ignored |
| /Start | `(00:34:56)` | time of day at movie time 0: seconds since midnight, or (HH:MM:SS) text |
| /Theme | `/midnight` | /midnight /amber /paper /phosphor |

## `/spinner`: Spinner

A loading indicator that loops: a ring with a sweeping arc, a circle of fading dots, or three bouncing bars.

120x120, 1.2 s, loops. `/x /spinner component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | the moving part |
| /Style | `/ring` | /ring /dots /bars |
| /Track | `[1 1 1]` | the ring's faint track |
| /Weight | `10` | stroke width, in its 120-unit square |

