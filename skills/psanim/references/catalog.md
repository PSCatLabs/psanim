# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

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

