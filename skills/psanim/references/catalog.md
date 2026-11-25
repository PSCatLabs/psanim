# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

## `/clock-analog`: Analog clock

A round clock face with tapered hour and minute hands and a red second hand that either ticks with a little overshoot, like quartz, or sweeps. Themes, optional numerals. Time of day from a start time and the movie's own time, or from a host signal.

640x640, 86400.0 s. `/x /clock-analog component`, then `/x << /Box [..] >> place`.

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

960x320, 86400.0 s. `/x /clock-digital component`, then `/x << /Box [..] >> place`.

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

960x160, 86400.0 s. `/x /clock-ledsign component`, then `/x << /Box [..] >> place`.

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

960x320, 86400.0 s. `/x /clock-morph component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Blink | `0` | 1 dims the colons for the second half of each second |
| /Melt | `0.6` | seconds (at most 1) a digit takes to melt into the next, finishing on the stroke of the second |
| /Panel | `1` | 1 paints a rounded panel behind the digits, 0 leaves them on whatever is there |
| /Seconds | `1` | 1 shows HH:MM:SS, 0 shows HH:MM |
| /Signal | `null` | name of a host signal that carries seconds since midnight (a live page); Start is then ignored |
| /Start | `(00:34:56)` | time of day at movie time 0: seconds since midnight, or (HH:MM:SS) text |
| /Theme | `/midnight` | /midnight /amber /paper /phosphor |

## `/confetti`: Confetti

Confetti that tumbles, sways and flutters down around a message, which arrives with the first blast and pulses on every one. Styles: cannons from both corners, one burst from behind the message, a steady rain. It lasts until the last pieces are gone (6.55 s for the cannons); place it with /After /hold to keep the message up after that. /Seed on the instance changes the pieces.

1280x720, 6.5 s. `/x /confetti component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Backdrop | `[0.99 0.97 0.93]` | [r g b] behind it all, or /none to fall over your own picture |
| /Bursts | `null` | blast times on its own clock (the event track); null: the style's |
| /Detail | `(The builds passed)` | a smaller line under it; () for none |
| /Ink | `[0.15 0.13 0.22]` | the message's colour |
| /Message | `(Huzzah!)` | the headline; () for none |
| /Palette | `/festive` | /festive, a theme name (its accents and fonts), or [[r g b] ...] |
| /Style | `/cannons` | /cannons (from both bottom corners), /burst (from behind the message), /rain (a steady fall) |

## `/credits`: Credits

End credits rolling upward at a steady speed: headings, roles beside names, centred names, then a final card that stops in the centre.

1280x720, 24.0 s. `/x /credits component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Canvas | `[1280 720]` | canvas size in units, [width height] |
| /CardTime | `4` | seconds each card is held |
| /Cards | `[]` | cards held one after another before the roll: an array of section lists |
| /Duration | `24` | seconds the roll takes, after any cards: the speed follows |
| /Fade | `0.12` | fraction of the box's height over which lines fade at its top and bottom (0 for none) |
| /Final | `[...]` | the card the roll stops on, centred (a section list; [] for none) |
| /Ground | `false` | true paints the theme's background behind the roll |
| /HeadingFace | `/title` | theme role or font name for headings |
| /NameFace | `/body` | theme role or font name for names and notes |
| /NameSize | `0` | size of the names in units (0 chooses one from the canvas) |
| /RoleFace | `/body` | theme role or font name for the roles in a pair |
| /Sections | `[...]` | the roll: entries [/heading (..)] [/pair (role) (name) ..] [/name (..) ..] [/text (..)] [/space n] [/proc { draw } h] |
| /Settle | `0.12` | fraction of the box's height over which the roll slows to its stop (0 for none) |
| /Theme | `/inherit` | a theme name (/paper /dusk ..), or /inherit for the movie's |

## `/fire`: Fire

A flame that loops: stylized, as flat tongues in two or three colours, or realistic, a hot body with glow, flicker, rising embers and faint smoke. Height, width, intensity and wind take it from a candle to a campfire to a torch.

480x600, 4.0 s, loops. `/x /fire component`, then `/x << /Box [..] >> place`.

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

## `/fireworks`: Fireworks

Shells rise on trails and burst into glowing stars that fall and fade behind a message, which arrives with the first burst and pulses on every one. Styles: a show, a single shell for a small success, a finale. It lasts until the last stars are gone (6.9 s for the show); place it with /After /hold to keep the message up after that. /Seed on the instance changes the shells.

1280x720, 6.9 s. `/x /fireworks component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Bursts | `null` | burst times on its own clock (the event track); null: the style's |
| /Detail | `(The builds passed)` | a smaller line under it; () for none |
| /Message | `(Huzzah!)` | the headline; () for none |
| /Palette | `/festive` | /festive, a theme name (its accents and fonts), or [[r g b] ...] |
| /Sky | `true` | paint a night sky; false to burst over your own picture |
| /Style | `/show` | /show (six shells), /single (one, for a small success), /finale (a crescendo) |
| /Weight | `1` | line weight of the stars and trails: more for a small placement |

## `/pipeline`: Pipeline

A pipeline of stages lighting up in turn: each runs with a sweeping ring, then passes green with a check, or fails red with a cross and shakes while the stages after it are skipped. Timed by a list of stage times, or driven by a value or signal.

602x120, 6.2 s. `/x /pipeline component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | the stage that is running |
| /Fail | `-1` | the stage that fails (from 0), or -1; the stages after it are skipped |
| /FailColor | `[1 0.36 0.36]` | a stage that failed |
| /Idle | `[1 1 1]` | a stage not reached yet |
| /OkColor | `[0.3 0.85 0.5]` | a stage that passed |
| /Seconds | `5` | how long the whole run takes when Times is empty |
| /Signal | `()` | the name of a movie signal, 0 to the number of stages, instead of time |
| /Stages | `[...]` | the names, left to right |
| /Times | `[]` | N+1 times: stage i runs from Times[i] to Times[i+1]; empty for equal stages over /Seconds |
| /Value | `-1` | 0 to the number of stages: a still, instead of time (1.5 is build done, test half way) |

## `/progress`: Progress

A progress bar, ring or row of blocks. Determinate, driven by time, a value or a signal, with a failure look that stops and turns it red and a finished look that turns it green; or indeterminate, sweeping.

520x80, 4.4 s. `/x /progress component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | while it runs |
| /Ease | `/cubic-inout` | the pace of that fill (/linear for a steady one; use a monotonic easing with FailAt) |
| /FailAt | `-1` | the value at which it fails and goes red; -1 never |
| /FailColor | `[1 0.36 0.36]` | when it fails |
| /Label | `true` | the percentage (FAILED when it fails) |
| /Mode | `/time` | /time (fills over /Seconds), /value (shows /Value) or /indeterminate (sweeps) |
| /OkColor | `[0.3 0.85 0.5]` | when it reaches 1 |
| /Seconds | `3` | how long /time mode takes to fill |
| /Segments | `20` | how many blocks /segments has |
| /Signal | `()` | the name of a movie signal (0 to 1) to show, in place of Value or time |
| /Style | `/bar` | /bar /ring /segments |
| /Track | `[1 1 1]` | the empty part, faint |
| /Value | `0.6` | 0 to 1, shown in /value mode |

## `/spinner`: Spinner

A loading indicator that loops: a ring with a sweeping arc, a circle of fading dots, or three bouncing bars. Given a /Resolve time it stops and closes into a green check or a red cross.

120x120, 1.2 s, loops. `/x /spinner component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | the moving part |
| /FailColor | `[1 0.36 0.36]` | the cross |
| /OkColor | `[0.3 0.85 0.5]` | the check |
| /Resolve | `-1` | seconds at which it stops spinning and shows its Result; -1 never (it loops) |
| /Result | `/ok` | /ok (a check) or /fail (a cross), shown from /Resolve on |
| /Style | `/ring` | /ring /dots /bars |
| /Track | `[1 1 1]` | the ring's faint track |
| /Weight | `10` | stroke width, in its 120-unit square |

## `/terminal`: Terminal

An old video terminal: phosphor text with glow, scanlines, a bowed screen, persistence and grain, and a session typed as a person types it, slips and backspaces included, with output scrolling up the screen.

960x720, 9.3 s. `/x /terminal component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Bezel | `true` | the monitor's frame (false: only the screen) |
| /Cols | `50` | characters across |
| /Curve | `0.035` | how much the glass bows |
| /Font | `/Courier-Bold` | a monospace face: the one parameter a bitmap terminal font will replace |
| /Glow | `1` | phosphor bloom, 0 for none |
| /Grain | `0.1` | 0 to 1: film grain and flicker |
| /Hold | `1.5` | seconds to rest on the last screen |
| /Persistence | `0.6` | 0 to 1: how long text lingers as it scrolls, is erased or the cursor blinks off |
| /Phosphor | `/green` | /green /amber /white, or [r g b] |
| /Plan | `null` | a plan made in the movie with sessionplan, used instead of Session (so the movie can read its marks) |
| /PowerOn | `0` | seconds the tube takes to warm up at the start (a bright line opening to the screen); 0 for none |
| /Prompt | `(hacker@vax:~$ )` | the prompt printed before each typed command |
| /Rows | `18` | lines on the screen |
| /Scanlines | `0.3` | 0 to 1: dark lines between the raster lines |
| /Session | `[...]` | the script: [/type (cmd)] [/out (text)] [/lines (text) gap] [/progress (frames) gap] [/wait s] [/clear] [/mark /name] |
| /Speed | `1` | typing speed, 1 is brisk |
| /Typos | `0.03` | chance of a slip per letter, noticed, backspaced and fixed |

## `/testdots`: Test dots

A grid of test results arriving: dots fill in green or red, each at its own pace, with a failure ringing out and a tally underneath.

480x132, 4.5 s. `/x /testdots component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Cols | `12` | dots per row |
| /Count | `24` | how many tests |
| /FailColor | `[1 0.36 0.36]` | a failing test |
| /FailRate | `0` | or: the chance that each of the others fails, by seed |
| /Fails | `[7 18]` | which tests fail, by position from 0 |
| /Idle | `[1 1 1]` | a test not run yet, faint |
| /Label | `true` | the tally under the dots |
| /Mode | `/time` | /time (they arrive over /Seconds) or /value (shows /Value, a still) |
| /Pass | `[0.3 0.85 0.5]` | a passing test |
| /Seconds | `3` | how long the run takes in /time mode |
| /Signal | `()` | the name of a movie signal (0 to 1) to show instead |
| /Value | `0.5` | 0 to 1, how many are in, in /value mode |

