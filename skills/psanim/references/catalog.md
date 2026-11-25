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

## `/credits`: Credits

End credits rolling upward at a steady speed: headings, roles beside names, centred names, then a final card that stops in the centre.

1280x720, 24 s. `/x /credits component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Canvas | `[1280 720]` | canvas size in units, [width height] |
| /CardTime | `4` | seconds each card is held |
| /Cards | `[]` | cards held one after another before the roll: an array of section lists |
| /Duration | `24` | seconds the roll takes, after any cards: the speed follows |
| /Fade | `0.12` | fraction of the box's height over which lines fade at its top and bottom (0 for none) |
| /Final | `[...]` | the card the roll stops on, centred (a section list; [] for none) |
| /Ground | `?` | true paints the theme's background behind the roll |
| /HeadingFace | `/title` | theme role or font name for headings |
| /NameFace | `/body` | theme role or font name for names and notes |
| /NameSize | `0` | size of the names in units (0 chooses one from the canvas) |
| /RoleFace | `/body` | theme role or font name for the roles in a pair |
| /Sections | `[...]` | the roll: entries [/heading (..)] [/pair (role) (name) ..] [/name (..) ..] [/text (..)] [/space n] [/proc { draw } h] |
| /Settle | `0.12` | fraction of the box's height over which the roll slows to its stop (0 for none) |
| /Theme | `/inherit` | a theme name (/paper /dusk ..), or /inherit for the movie's |

## `/spinner`: Spinner

A loading indicator that loops: a ring with a sweeping arc, a circle of fading dots, or three bouncing bars.

120x120, 1.2 s, loops. `/x /spinner component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | the moving part |
| /Style | `/ring` | /ring /dots /bars |
| /Track | `[1 1 1]` | the ring's faint track |
| /Weight | `10` | stroke width, in its 120-unit square |

