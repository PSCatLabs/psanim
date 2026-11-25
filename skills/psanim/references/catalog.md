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
| /Plan | `?` | a plan made in the movie with sessionplan, used instead of Session (so the movie can read its marks) |
| /PowerOn | `0` | seconds the tube takes to warm up at the start (a bright line opening to the screen); 0 for none |
| /Prompt | `(hacker@vax:~$ )` | the prompt printed before each typed command |
| /Rows | `18` | lines on the screen |
| /Scanlines | `0.3` | 0 to 1: dark lines between the raster lines |
| /Session | `[...]` | the script: [/type (cmd)] [/out (text)] [/lines (text) gap] [/progress (frames) gap] [/wait s] [/clear] [/mark /name] |
| /Speed | `1` | typing speed, 1 is brisk |
| /Typos | `0.03` | chance of a slip per letter, noticed, backspaced and fixed |

