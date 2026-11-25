# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

## `/spinner`: Spinner

A loading indicator that loops: a ring with a sweeping arc, a circle of fading dots, or three bouncing bars.

120x120, 1.2 s, loops. `/x /spinner component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | the moving part |
| /Style | `/ring` | /ring /dots /bars |
| /Track | `[1 1 1]` | the ring's faint track |
| /Weight | `10` | stroke width, in its 120-unit square |

## `/twiddle`: Twiddle

Twiddle, the twiddle project's mascot: a round clay-orange staticling who dances to a real beat (a tempo or an event track), grins on a big kick, sings a beat late, waves on a splash and falls asleep when the music stops.

400x400, 32 s, loops. `/x /twiddle component`, then `/x << /Box [..] >> place`.

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

