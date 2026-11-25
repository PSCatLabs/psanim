# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

## `/confetti`: Confetti

Confetti that tumbles, sways and flutters down around a message, which arrives with the first blast and pulses on every one. Styles: cannons from both corners, one burst from behind the message, a steady rain. It lasts until the last pieces are gone (6.55 s for the cannons); place it with /After /hold to keep the message up after that. /Seed on the instance changes the pieces.

1280x720, 6.55 s. `/x /confetti component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Backdrop | `[0.99 0.97 0.93]` | [r g b] behind it all, or /none to fall over your own picture |
| /Bursts | `null` | blast times on its own clock (the event track); null: the style's |
| /Detail | `(The builds passed)` | a smaller line under it; () for none |
| /Ink | `[0.15 0.13 0.22]` | the message's colour |
| /Message | `(Huzzah!)` | the headline; () for none |
| /Palette | `/festive` | /festive, a theme name (its accents and fonts), or [[r g b] ...] |
| /Style | `/cannons` | /cannons (from both bottom corners), /burst (from behind the message), /rain (a steady fall) |

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

## `/spinner`: Spinner

A loading indicator that loops: a ring with a sweeping arc, a circle of fading dots, or three bouncing bars.

120x120, 1.2 s, loops. `/x /spinner component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | the moving part |
| /Style | `/ring` | /ring /dots /bars |
| /Track | `[1 1 1]` | the ring's faint track |
| /Weight | `10` | stroke width, in its 120-unit square |

