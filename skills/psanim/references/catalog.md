# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

## `/chat`: Chat

A phone conversation of text messages: bubbles in two colours with tails, bouncing dots before each message, timestamps, delivered and read marks, reactions, emoji, and the list scrolling up as messages arrive. Its /Script is a list of [ sender (text) when ]: sender /me or /them, when a pause in seconds or a dictionary (/Typing /At /Time /React ...); the length of the movie follows it.

400x800, 16.51 s. `/x /chat component`, then `/x << /Box [..] >> place`.

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

## `/spinner`: Spinner

A loading indicator that loops: a ring with a sweeping arc, a circle of fading dots, or three bouncing bars.

120x120, 1.2 s, loops. `/x /spinner component`, then `/x << /Box [..] >> place`.

| Param | Default | Meaning |
|---|---|---|
| /Color | `[0.36 0.78 0.98]` | the moving part |
| /Style | `/ring` | /ring /dots /bars |
| /Track | `[1 1 1]` | the ring's faint track |
| /Weight | `10` | stroke width, in its 120-unit square |

