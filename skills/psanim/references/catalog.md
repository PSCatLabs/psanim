# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

## `/crawl`: Crawl

Text receding on a tilted plane towards a vanishing point, justified and fading with distance, over an optional starfield: an opening crawl.

1280x720, 40 s. `/x /crawl component`, then `/x << /Box [..] >> place`.

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
| /Text | `[...]` | paragraphs, justified |
| /Tilt | `58` | degrees the plane leans back: 0 faces you, 90 is a floor |
| /Title | `[...]` | title lines, centred; [] for none |
| /TitleColor | `[1 0.93 0.72]` | title colour |
| /TitleSize | `0` | title size; 0 means 1.3 times /Size |
| /Width | `900` | column width where the plane meets the bottom edge |

## `/spinner`: Spinner

A loading indicator that loops: a ring with a sweeping arc, a circle of fading dots, or three bouncing bars.

120x120, 1.2 s, loops. `/x /spinner component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | the moving part |
| /Style | `/ring` | /ring /dots /bars |
| /Track | `[1 1 1]` | the ring's faint track |
| /Weight | `10` | stroke width, in its 120-unit square |

