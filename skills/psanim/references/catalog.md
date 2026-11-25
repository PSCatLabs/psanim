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

## `/stat`: Stat panel

A dashboard tile: a big number coloured by thresholds, its unit, its change and a sparkline of its recent past, from a host signal or an array.

360x200, 12 s, loops. `/x /stat component`, then `/x << /Box [..] >> place`.

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

## `/gauge`: Gauge

A dashboard gauge: an arc filled to the value in its threshold's colour, the thresholds as a band round it, the number in the middle; from a host signal or an array.

300x260, 12 s, loops. `/x /gauge component`, then `/x << /Box [..] >> place`.

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

900x520, 12 s, loops. `/x /horizon component`, then `/x << /Box [..] >> place`.

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

## `/counter`: Counter

A vanity metric that counts up: stars, commits, requests served, days without an incident. Grouped or short numbers, an icon, a settle at the end; or follows a host signal.

420x200, 5 s. `/x /counter component`, then `/x << /Box [..] >> place`.

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

## `/status`: Status lights

One lamp per service, green, amber or red by thresholds, with its name and value; the worst ones pulse. From host signals or arrays.

300x260, 12 s, loops. `/x /status component`, then `/x << /Box [..] >> place`.

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

## `/timeseries`: Time series

A scrolling line chart of up to four series with a legend; the y range follows the data and its ticks cross-fade as it moves. From host signals or arrays.

640x300, 12 s, loops. `/x /timeseries component`, then `/x << /Box [..] >> place`.

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

