# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

## `/chat`: Chat

A phone conversation of text messages: bubbles in two colours with tails, bouncing dots before each message, timestamps, delivered and read marks, reactions, emoji, and the list scrolling up as messages arrive. Its /Script is a list of [ sender (text) when ]: sender /me or /them, when a pause in seconds or a dictionary (/Typing /At /Time /React ...); the length of the movie follows it.

400x800, 16.5 s. `/x /chat component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Clock | `(9:41)` | the status bar's time |
| /Composer | `true` | show the field at the bottom where what I send is typed |
| /Contact | `(Sam)` | the name in the header |
| /Frame | `true` | true: a phone around it (bezel, status bar, home bar); false: the screen alone |
| /Header | `true` | show the contact's name bar |
| /Hold | `2.5` | seconds to rest on the finished conversation |
| /Lead | `0.8` | seconds before the first message starts |
| /Pace | `1` | speed of the default typing and pauses: 2 is twice as quick |
| /Receipts | `true` | Delivered, then Read, under my last message |
| /Script | `[]` | [ sender (text) when ] per message: see the top of this file; empty shows a demo |
| /Subtitle | `(online)` | under the name; "typing..." takes its place while they type |
| /Theme | `/light` | /light /dark /mint /sunset, or a dictionary of colours changing one of them: << /me [0.9 0.3 0.5] >> |

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

## `/counter`: Counter

A vanity metric that counts up: stars, commits, requests served, days without an incident. Grouped or short numbers, an icon, a settle at the end; or follows a host signal.

420x200, 5.0 s. `/x /counter component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Card | `true` | draw the panel's card behind it |
| /Count | `0.75` | the part of the duration spent counting; then it settles and holds |
| /Decimals | `0` | decimal places |
| /Ease | `/expo-out` | how the count runs: fast and slowing is the convention |
| /From | `0` | where the count starts |
| /Height | `200` | its height in units |
| /Icon | `/star` | /star /commit /check /bolt /flame, or /none |
| /Label | `(stars on GitHub)` | the words under the number |
| /Lag | `0` | with /Signal: read it this many seconds late, so it glides between updates |
| /Prefix | `()` | before the number, e.g. ($) |
| /Role | `/accent` | the number's colour (and the icon's) |
| /Short | `false` | true: 12.4k instead of 12,431 |
| /Signal | `null` | follow this host signal instead of counting from /From to /To |
| /Suffix | `()` | after it, e.g. (+) |
| /Theme | `null` | a theme name, or null for the movie's own |
| /To | `12431` | where it ends |
| /Width | `420` | its width in units: give it the shape of the box it is placed in |

## `/crawl`: Crawl

Text receding on a tilted plane towards a vanishing point, justified and fading with distance, over an optional starfield: an opening crawl.

1280x720, 40.0 s. `/x /crawl component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.98 0.84 0.46]` | text colour |
| /Fade | `[0.38 0.6]` | fractions of the way up to the vanishing point where the text starts to fade and where it is gone |
| /Font | `/Helvetica-Bold` | the font of the text and the title |
| /Horizon | `0.95` | height of the vanishing point, as a fraction of the frame |
| /Leading | `1.3` | line spacing, in multiples of /Size |
| /Size | `46` | type size where the plane meets the bottom edge |
| /Sky | `[0.008 0.01 0.03]` | the colour behind the stars, or /none for a transparent ground |
| /Speed | `0` | units a second along the plane; 0 fits the whole text into the entry's 40 s |
| /Stars | `350` | stars behind the text; 0 for none |
| /Text | `[(Across the outer reaches, the beacons have gone dark one by one. Nobody on the inner worlds noticed until the grain ships stopped coming.) (The Council of Ports has sent letters, then inspectors, then warships. None of them have come back, and the letters have stopped being answered.) (Now a single surveyor, her ship held together with wire and stubbornness, follows the last signal into the drift, where the old maps end.) (She carries a crate of spare lamps, a chart drawn by her grandmother, and a passenger who has not yet said why he wanted to come.) (What she finds there will either light the beacons again or explain why somebody wanted them out....)]` | paragraphs, justified |
| /Tilt | `58` | degrees the plane leans back: 0 faces you, 90 is a floor |
| /Title | `[(EPISODE ZERO) (THE QUIET ENGINE)]` | title lines, centred; [] for none |
| /TitleColor | `[1 0.93 0.72]` | title colour |
| /TitleSize | `0` | title size; 0 means 1.3 times /Size |
| /Width | `900` | column width where the plane meets the bottom edge |

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

## `/gauge`: Gauge

A dashboard gauge: an arc filled to the value in its threshold's colour, the thresholds as a band round it, the number in the middle; from a host signal or an array.

300x260, 12.0 s, loops. `/x /gauge component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Card | `true` | draw the panel's card behind it |
| /Decimals | `0` | decimal places |
| /Height | `260` | its height in units |
| /Label | `()` | a word under the number |
| /Lag | `0` | read the signal this many seconds late, so live values glide between updates |
| /Range | `[0 100]` | the values at the two ends of the arc |
| /Signal | `null` | the host signal to show, e.g. /cpu; null plays /Values |
| /Theme | `null` | a theme name, or null for the movie's own |
| /Thresholds | `[/good 70 /warn 90 /bad]` | the colours by value |
| /Title | `(CPU)` | the panel's title; () for none |
| /Unit | `(%)` | written small after the number |
| /Values | `null` | without a signal: values played evenly across the duration (null: a demo) |
| /Width | `300` | its width in units: give it the shape of the box it is placed in |

## `/horizon`: Horizon chart

Many metrics, one row each, folded into bands of colour so a whole fleet fits on a screen; a time cursor and a readout of every row at it. From host signals or arrays.

900x520, 12.0 s, loops. `/x /horizon component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Bands | `4` | bands of colour per row |
| /Card | `true` | draw the panel's card behind it |
| /Clock | `null` | [c0 rate]: label times as a clock, c0 seconds (since midnight) at time 0, rate clock seconds per second |
| /Decimals | `0` | decimal places of the readout |
| /Extent | `null` | the value that fills every band (100 for percentages); null: each row's largest in view (set it when scrolling, or rows rescale as peaks pass) |
| /From | `0` | /replay: the time at the left edge |
| /Height | `520` | its height in units |
| /Labels | `null` | a name per row; null: the signal names |
| /Mode | `null` | /scroll or /replay; null: /replay for signals, /scroll for /Values |
| /NegRole | `/accent3` | the colour below zero |
| /Points | `240` | readings per row |
| /Role | `/accent2` | the colour above zero |
| /Signals | `null` | the host signals, one row each, e.g. [/web1 /web2]; null plays /Values |
| /Theme | `null` | a theme name, or null for the movie's own |
| /Thresholds | `null` | [ /good 70 /warn 90 /bad ]: a lamp per row from its value at the cursor |
| /Tick | `10800` | with /Clock: clock seconds between the time labels |
| /Title | `()` | the panel's title; () for none |
| /To | `null` | /replay: the time at the right edge; null: the duration |
| /Unit | `()` | after each readout |
| /Values | `null` | without signals: one array per row, a recording that loops over the duration (null: a demo) |
| /Width | `900` | its width in units: give it the shape of the box it is placed in |
| /Window | `4` | /scroll: seconds shown |

## `/ledmatrix`: LED matrix

A grid of round LEDs with a pixel font of its own: a scrolling marquee, a ticker, a news sign with entry effects and a clock, or any pattern a procedure computes. Raw mode gives the grid at true size for driving real LEDs.

600x108, 3.6 s, loops. `/x /ledmatrix component`, then `/x << /Box [..] >> place`.

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
| /Stages | `[(build) (test) (deploy)]` | the names, left to right |
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

## `/stat`: Stat panel

A dashboard tile: a big number coloured by thresholds, its unit, its change and a sparkline of its recent past, from a host signal or an array.

360x200, 12.0 s, loops. `/x /stat component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Decimals | `0` | decimal places |
| /Delta | `2` | show the change in % over this many seconds (0: none) |
| /DownIsGood | `false` | true when a fall is good news (latency, errors) |
| /Fill | `/surface` | /surface, or /threshold to colour the card instead of the number |
| /Height | `200` | its height in units |
| /Lag | `0` | read the signal this many seconds late, so live values glide between updates |
| /Short | `false` | true: 12.4k and 3.2M instead of 12,431 and 3,200,000 |
| /Signal | `null` | the host signal to show, e.g. /cpu; null plays /Values |
| /Spark | `4` | seconds of history in the sparkline (0: none) |
| /Theme | `null` | a theme name, or null for the movie's own |
| /Thresholds | `null` | [ /good 70 /warn 90 /bad ]: the number's colour by value; null: /fg |
| /Title | `(Requests / s)` | the panel's title |
| /Unit | `()` | written small after the number |
| /Values | `null` | without a signal: values played evenly across the duration (null: a demo) |
| /Width | `360` | its width in units: give it the shape of the box it is placed in |

## `/status`: Status lights

One lamp per service, green, amber or red by thresholds, with its name and value; the worst ones pulse. From host signals or arrays.

300x260, 12.0 s, loops. `/x /status component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Card | `true` | draw the panel's card behind it |
| /Decimals | `0` | decimal places |
| /Height | `260` | its height in units |
| /Labels | `null` | a name per lamp; null: the signal names |
| /Lag | `0` | read the signals this many seconds late, so live values glide between updates |
| /Signals | `null` | the host signals, one lamp each; null plays /Values |
| /Theme | `null` | a theme name, or null for the movie's own |
| /Thresholds | `[/good 70 /warn 90 /bad]` | the lamp's colour by value |
| /Title | `(Services)` | the panel's title; () for none |
| /Unit | `()` | after each value |
| /Values | `null` | without signals: one array per lamp, played across the duration (null: a demo) |
| /Width | `300` | its width in units: give it the shape of the box it is placed in |

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

## `/timeseries`: Time series

A scrolling line chart of up to four series with a legend; the y range follows the data and its ticks cross-fade as it moves. From host signals or arrays.

640x300, 12.0 s, loops. `/x /timeseries component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Card | `true` | draw the panel's card behind it |
| /Clock | `null` | [c0 rate]: label times as a clock (see the horizon entry) |
| /Decimals | `0` | decimal places of the y labels |
| /Fill | `true` | shade under the first line |
| /Height | `300` | its height in units |
| /Labels | `null` | a legend entry per line; null: the signal names |
| /Least | `1` | with no /Range: the lowest the top of the range goes, so a quiet signal is not magnified |
| /Points | `120` | readings per line |
| /Range | `null` | [lo hi] fixed; null: 0 to just above the largest value shown |
| /Roles | `[/accent2 /accent /accent3 /good]` | the lines' colours |
| /Signals | `null` | the host signals, one line each; null plays /Values |
| /Theme | `null` | a theme name, or null for the movie's own |
| /Tick | `3600` | with /Clock: clock seconds between time labels |
| /Title | `(Requests / s)` | the panel's title; () for none |
| /Unit | `()` | after the y labels |
| /Values | `null` | without signals: one array per line, a recording that loops over the duration (null: a demo) |
| /Width | `640` | its width in units: give it the shape of the box it is placed in |
| /Window | `4` | seconds shown |

## `/twiddle`: Twiddle

Twiddle, the twiddle project's mascot: a round clay-orange staticling who dances to a real beat (a tempo or an event track), grins on a big kick, sings a beat late, waves on a splash and falls asleep when the music stops.

400x400, 32.0 s, loops. `/x /twiddle component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Accents | `[1 0.5 0.8 0.5]` | loudness of beat after beat, cycled (a bar, or a whole song), times /Level |
| /Beat | `/none` | an event track to dance to instead of the tempo (/beat, via --signals) |
| /From | `0` | seconds into his clock when the tempo's music starts (asleep before) |
| /Level | `0.9` | loudness 0..1, or a signal name to read it from (/level) |
| /Period | `0.5` | with /Beat: about how long a beat is, in seconds |
| /State | `/dance` | /dance (to the beat), /wave (a splash: no music), /sleep |
| /Tempo | `120` | beats a minute, from his clock; 0 is no music. Loops at multiples of 15 |
| /Until | `0` | seconds into his clock when the tempo's music stops (0: it never does) |

