# psanim catalog

Generated from `crates/psanim/catalog/` by `scripts/gen_reference.sh`. Do not edit by hand.

Ready-made elements, each a movie of its own. Place one as a component (see components.md): `/x /entry component` in setup, then `/x << /Box [x y w h] /At t >> place` in the scene; `/y /entry << /Params << ... >> >> instance` makes another with other parameters. `psanim catalog NAME` prints an entry's source, which is also the way to learn from or adapt it.

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
| /Plan | `?` | a plan made in the movie with sessionplan, used instead of Session (so the movie can read its marks) |
| /PowerOn | `0` | seconds the tube takes to warm up at the start (a bright line opening to the screen); 0 for none |
| /Prompt | `(hacker@vax:~$ )` | the prompt printed before each typed command |
| /Rows | `18` | lines on the screen |
| /Scanlines | `0.3` | 0 to 1: dark lines between the raster lines |
| /Session | `[...]` | the script: [/type (cmd)] [/out (text)] [/lines (text) gap] [/progress (frames) gap] [/wait s] [/clear] [/mark /name] |
| /Speed | `1` | typing speed, 1 is brisk |
| /Typos | `0.03` | chance of a slip per letter, noticed, backspaced and fixed |

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

