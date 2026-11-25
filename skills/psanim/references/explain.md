# Explaining: diagrams, code, charts

```postscript partial
/pkdiagram use   /pkcode use   /pkchart use      % each loads pkstage, pkmotion, pktype
```

Scene bodies below run on a 640x360 canvas, 4 seconds, libraries loaded;
a block starting `%!PS` is a whole movie. `psanim api pkdiagram` (and
`pkcode`, `pkchart`) list every word.

## Contents

- [Diagrams](#diagrams): nodes, edges, flow, groups, marks, notes
- [Code](#code): panels, typing, focus, diffs
- [Charts](#charts): frames, axes, bars, lines, rankings, donuts
- [Dashboards](#dashboards): stat tiles, gauges, horizon charts, live and recorded data
- [Putting them in a film](#putting-them-in-a-film)

## Diagrams

A diagram is **boxes you named in setup**. Lay them out with pkstage
(`cols`, `rows`, `cells`, `placed`), draw a `node` in each, and connect
boxes with `edge`.

```postscript
/g canvasbox 10 vmin inset 3 16 vmin cols def
/a g 0 get def /b g 1 get def /c g 2 get def
ground
0.0 /pop { (source) a /surface node } enters
0.3 /pop { (engine) b << /Fill /accent /Ink /bg /Sub (per frame) /SubInk /bg >> node } enters
0.6 /pop { (frames) c << /Shape /pill >> node } enters
a b << /Draw 1 1.6 /cubic-inout tween /Label (program) >> edge
b c << /Draw 1.5 2.1 /cubic-inout tween /Pulse 1.2 2 pulsetrain >> edge
```

- **`(label) box opts node`**: `opts` is a fill role or `<< /Fill /Stroke
  /Ink /Shape /Sub /SubInk /Size /Font >>`. Shapes `/box /pill /circle
  /diamond /cylinder /plain`. The label is sized to fit the box.
- **`boxA boxB opts edge`**: `opts` is a route or `<< /Route /Draw /Label
  /Pulse /Role /Head /Dash /Width >>`. Routes: `/straight`, `/elbow`
  (right angles), `/hv` `/vh` (one corner), `/arc` `/arc-right`. Animate
  `/Draw` 0 to 1 to draw the edge on; the label fades in over the second
  half. `/Pulse` is a position 0..1 (or an array of them) of a comet
  travelling the edge: `period n pulsetrain` gives a steady stream, a
  tween gives one message: `/Pulse 2 2.8 /cubic-inout tween`.
- Routes that must avoid something: `[/s /n]` names the side of each box
  the edge leaves and arrives by (down, across, down), and `[ x y x y
  ... ]` is a list of points to pass through.
- `/Label` is drawn at `/LabelSize` (default 2.1 vmin, which is small:
  give 3 vmin or more for a label that matters). `/PulseLen` is a
  fraction of the edge, so on a short edge a comet is a dot: lengthen
  it, or leave pulses to the long edges. A `/Pulse` position of exactly
  0 or 1 draws nothing, so a clamped `tween` shows one message and
  then none; `[ ]` is no pulses.
- `/cylinder` and `/pill` leave less room than `/box`: keep labels short
  or give `/Size`.
- **`(label) box opts group`**: a dashed container. `a b boxunion -3 vmin
  inset` is the box around two nodes.
- Bring nodes on with motions (`enters`, `cascade`), not by hand.
- Give nodes room: at least 16 vmin wide and 9 vmin tall, with gaps of 6
  vmin or more for an edge and its label. A label is sized to fit its
  box but never above 5 vmin; pass `/Size` to set it.

An edge is built on a lower level you can use directly: `boxA boxB /route
connect` makes the connector the **current path**, and then every path
word applies. `u arrow` strokes it with a head riding the tip; `u len
pulse` puts a comet on it; `u { } onpath` rides anything along it.

```postscript
/a [ 60 220 150 70 ] def /b [ 430 70 150 70 ] def
ground
(queue) a /surface node  (worker) b /surface node
/line ink 2.5 setlinewidth
a b /elbow connect  0.3 1.5 /cubic-inout tween arrow
a b /elbow connect                                   % again: `arrow` consumed the path
1.5 4 ramp { /accent ink -9 -6 18 12 rectfill } onpath   % a packet rides the connector
newpath
```

### A presenter's marks

Each makes a path to draw on with `trimpath`; `note` is a label with a
leader line.

```postscript
/t [ 200 150 240 60 ] def
ground /title 40 face /fg ink (the claim) t /c showin
/accent3 ink 3 setlinewidth 1 setlinecap
t circled             0  0.4 1.4 /cubic-inout tween trimpath stroke
t /s bracepath        0  1.6 2.2 /cubic-out tween trimpath stroke
/body 16 face (worth a second look) t /ne 2.2 3 ramp note
```

`box struck` is a strike-through (drawn by hand: it overshoots the box a
little and slopes). `box /side bracepoint` is the tip of a brace, where
its label goes. `box amount spotlight` (pkstage)
darkens everything but one box.

## Code

Tokenize in setup, draw each frame.

```postscript
%!PS
<< /Size [640 360] /Duration 6 /FPS 30 >> animation
/pkcode use /pkdiagram use
/src (fn area(r: f64) -> f64 {\n    let pi = 3.14;   // close enough?\n    pi * r * r\n}\n) /rust codetokens def
/panel canvasbox 8 vmin inset def
/opts << /Title (area.rs) /Numbers true >> def
/scene {
  ground
  src panel << opts { } forall
     /Reveal 0.2 2.2 ramp                           % typed in
     /Focus [1] /FocusAmount 3 3.5 ramp             % then line 1 is singled out
  >> codeblock
  /accent ink /body 16 face
  (an approximation) src panel opts 1 codelinebox /e 3.5 4.3 ramp note
} def
```

- `(source) /lang codetokens`: `/rust /python /javascript /typescript /c
  /go /postscript /shell /json /plain`. In a PostScript string write
  `\n` for a line break and escape unbalanced parentheses and
  backslashes (`\(`, `\\`).
- `lines box opts codeblock`: `/Reveal` (typing), `/Focus [lines]` with
  `/FocusAmount`, `/Scroll` (lines, fractional), `/Numbers`, `/Title`,
  `/Size` (otherwise it fits the box).
- `lines box opts n codelinebox`: the box of line n (from 0), for notes,
  `circled`, `spotlight`, `lookat`. Pass the same box and the layout
  options (`/Title /Numbers /Size`) you gave `codeblock`.
- `<< opts { } forall /More x >>` extends a dictionary of shared options,
  as above.

A change to code is a diff animated by one number:

```postscript
%!PS
<< /Size [640 360] /Duration 5 /FPS 30 >> animation
/pkcode use
/rows
  (def mean(xs):\n    total = 0\n    for x in xs:\n        total += x\n    return total / len(xs)\n)
  (def mean(xs):\n    return sum(xs) / len(xs)\n)
  /python diffrows def
/scene {
  ground
  rows canvasbox 8 vmin inset << /Title (stats.py) >> 1 4 ramp codediff
} def
```

## Charts

`box [x0 x1] [y0 y1] plotframe` says where the plot is and what it
spans; the other words draw in that frame. **Animating a chart is
animating its numbers**: every mark takes a `/Grow`, and `u A B between`
is the dataset part-way from A to B.

```postscript
/A [ 12 30 22 41 35 ] def  /B [ 40 18 33 25 52 ] def
ground
canvasbox 12 vmin inset [0 5] [0 60] plotframe
<< /Draw 0 0.8 ramp /XNames [ (Mon) (Tue) (Wed) (Thu) (Fri) ] /Size 12 >> plotaxes
2.2 3.2 /cubic-inout tween A B between                  % the data changes at 2.2 s
<< /Grow 0.4 1.8 ramp /Role /accent /Dim [1] /Size 13 >> bars
```

A line, its area, and a note at a data point:

```postscript
/pts [ [0 12] [1 18] [2 15] [3 34] [4 41] [5 68] ] def
ground
canvasbox 12 vmin inset [0 5] [0 80] plotframe
<< /Draw 0 0.8 ramp /XLabel (year) /Size 12 >> plotaxes
gsave /accent2 ink 0.18 0.8 2.4 ramp mul setalpha pts plotline plotarea fill grestore
/accent2 ink 3 setlinewidth 1 setlinejoin
pts plotline  0  0.8 2.4 /cubic-inout tween trimpath stroke
/fg ink pts 1.6 2.6 ramp 5 dots
/accent ink /body 15 face
(the jump) [ 5 68 plotpoint 0 0 ] /w 2.8 3.5 ramp note
```

A ranking that reorders (a bar-chart race) is `ranks` interpolated:

```postscript
/A [ 30 52 41 18 ] def  /B [ 64 40 58 71 ] def
ground
canvasbox 12 vmin inset 0.2 splitx exch pop [0 80] [0 1] plotframe
/u 1.5 3 /cubic-inout tween def
u A B between
<< /Grow 0.2 1.2 ramp /Names [ (north) (south) (east) (west) ]
   /Ranks u A ranks B ranks between /Roles chartroles /Size 15 >> hbars
```

The plot frame's box is the **data rectangle**: tick labels and axis
titles are drawn outside it, about three times `/Size` to the left and
below. Inset the box from the page or they fall off the canvas.

The annotations a data story needs are words: `plotrule` (a reference
line), `plotmark` (a dot with drop lines and a label), `inplot` (clip
to the frame, needed whenever data can leave the range), and `valueat`
(read a table at a moving position, for a readout that follows the
curve). Animating the *range* is a zoom.

```postscript
/tbl [ 0 0.3 1.6 4.1 7.4 11.7 16.7 22.3 28.4 34.6 41.1 47.6 53.9 60.0 65.6 70.6 75.3 79.4 83.2 86.3 88.9 ] def  % every 2 people
/pts [ 0 1 20 { dup 2 mul exch tbl exch get 2 array astore } for ] def
ground
/zoom 2.2 3.4 /cubic-inout tween def
canvasbox 16 vmin inset  zoom [0 120] [0 40] between  [0 100] plotframe      % the x range narrows
<< /YValues [0 50 100] /Suffix (%) /Size 12 /XLabel (people in the room) >> plotaxes
<< /Y 50 /Label (even odds) /Draw 0.3 1 ramp /Size 11 >> plotrule
{ /accent2 ink 3 setlinewidth pts plotline 0 0.4 2 /cubic-inout tween trimpath stroke } inplot
23 50.7 << /Label (23 people) /Draw 2 3 ramp /Size 13 >> plotmark
% a readout that reads the table as it goes
/n 0.4 2 /cubic-inout tween 0 23 between def
/fg ink /title 20 face canvasbox 8 vmin inset /ne boxpoint moveto
tbl n 2 div valueat 1 fmtnum (%) strcat showright
```

Bar labels show the true value, fading in; `/Count true` makes them
count up with the bar. `/Accent [i]` mutes every bar but those.

`[v ...] box << /Grow u /Center (64%) >> donut` is a ring of shares.
`x y plotpoint` converts data to canvas coordinates for anything else
you want to put on a chart.

A chart tells a story when one thing in it is singled out: `/Dim` the
other bars, put the `/accent` on one, add a `note`, and say the finding
in a headline above (`"Thursday doubled"`, not `"Sales by day"`).

## Dashboards

Telemetry panels in the manner of Grafana: a number with its trend, a
gauge, a fleet of horizon rows, lamps that go amber and red. The words
take numbers and a box, like every chart word; the colour of a value
comes from thresholds, `v [ /good 70 /warn 90 /bad ] thresholdrole`
(the first role below 70, `/warn` from 70, `/bad` from 90; every theme
has the three).

```postscript
ground
/u 0 3.5 /cubic-inout tween def
/hist [ 0 1 47 { 47 div 360 mul /a exch def a 2 mul sin 20 mul a 5 mul sin 8 mul add 50 add } for ] def
% a stat tile: title, number in its threshold's colour, unit, change, sparkline
u 40 92 between [16 196 300 148] << /Title (CPU) /Unit (%) /Decimals 1 /Delta 4.2
  /Thresholds [ /good 70 /warn 90 /bad ] /Spark hist >> stattile
% a gauge
u 20 78 between [332 196 292 148] << /Unit (%) /Label (disk) >> dial
% a horizon chart per host, and a lamp each
0 1 3 { /i exch def
  [ 0 1 47 { 47 div 360 mul i 40 mul add sin 45 mul 45 add } for ] [ 60 140 i 34 mul sub 560 28 ]
    << /Extent 100 /Bands 4 /Draw u >> horizon
  30 154 i 34 mul sub 6 i 2 mod 0 eq { /good } { /warn } ifelse statuslight
} for
```

`sparkline`, `heatmap`, `statuslight` and `panel` (a titled card that
returns the box inside it) are the rest. `fmtshort` writes 12.4k and
3.2M; `fmtnum` writes 12,431.

**Ready-made panels.** The catalog has them as components that read the
host's data themselves: `/stat`, `/gauge`, `/horizon` (many rows, a time
cursor, a readout), `/timeseries` (scrolling, with ticks that cross-fade
as the range moves), `/status` (lamps) and `/counter` (a vanity number
that counts up). Give each its box's shape with /Width and /Height, so
`place` fits it without bars:

```postscript
%!PS
<< /Size [960 540] /Duration 6 >> animation
/pkchart use
/boxes canvasbox 16 inset 3 2 12 cells def
% /new /entry box params sized -> /new /entry opts     an instance the shape of the box
/sized { dup /Width 3 index 2 get put dup /Height 3 index 3 get put exch pop << /Params 3 -1 roll >> } def
/a /stat boxes 0 get << /Title (Requests / s) /Short true /Decimals 1 >> sized instance
/b /gauge boxes 1 get << /Title (Disk) >> sized instance
/c /counter boxes 2 get << /To 48213 /Label (commits) /Icon /commit >> sized instance
/d /status boxes 3 get << >> sized instance
/e /timeseries boxes 4 get << >> sized instance
/f /horizon boxes 5 get << >> sized instance
/scene { ground 0 1 5 { /i exch def [ /a /b /c /d /e /f ] i get << /Box boxes i get /After /hold >> place } for } def
```

**Where the data comes from.** Never from a file the movie opens: from
the host, as signals (`--signals day.csv`, a CSV whose first column is
movie time; or a live host calling `Movie::record_signal` with each
reading). The panels take `/Signal /cpu` (or `/Signals [ ... ]`) and
read the history themselves; without one they play a `/Values` array
across their duration. By hand:

```postscript partial
/cpu signal                           % now
/cpu 0.5 signalago                    % half a second ago: live values glide between updates
/cpu 3 60 signalseries                % the last 3 seconds, 60 readings, for a sparkline
/cpu 0 60 360 signalbetween           % a fixed stretch: a recorded day behind a sweeping cursor
signaltime                            % the clock signals are read on, also inside a component
/cpu hassignal not { (no data) show } if
```

A recording replayed is the same as a live feed: the CSV's time column
is movie seconds (a day in a minute is 1 s = 24 min), and the movie
labels its clock from that (the panels' `/Clock [c0 rate]`).

Two rules keep a dashboard from flickering. Read a window on a grid
fixed in time (`signalbetween` from multiples of the step) rather than
`signalseries` at moving instants, or thin peaks shimmer as the samples
slide; the `/horizon` and `/timeseries` entries do this. And let a
moving axis cross-fade its ticks, `<< /Fade true >> plotaxes`
(`fadeticks` underneath), so a growing range never makes the labels
jump.

## Putting them in a film

Explainers are progressive disclosure: one new thing at a time, each
arriving by a motion, the previous things staying. A `script` reads like
the narration.

```postscript
%!PS
<< /Size [640 360] /Duration 8 /FPS 30 >> animation
/pkdiagram use
/g canvasbox 10 vmin inset 0.3 splity exch pop 3 14 vmin cols def
/a g 0 get def /b g 1 get def /c g 2 get def
/head canvasbox 10 vmin inset 0.2 splity pop def
/scene {
  ground
  {
    1.0 { /title 26 face /fg ink head /w boxpoint moveto
          (How a frame is made) /words 0.08 /rise kinetic } then
    0.6 { /pop { (program) a /surface node } entering } then
    0.6 { /pop { (clock) b /surface node } entering
          a b << /Draw 0.2 0.8 /cubic-inout tween >> edge } then
    1.0 { /pop { (picture) c << /Fill /accent /Ink /bg >> node } entering
          b c << /Draw 0.2 0.8 /cubic-inout tween /Pulse 1 2 pulsetrain >> edge } then
    0.5 wait
    2.0 { /body 15 face /accent ink
          (no memory between frames) b /n 0 0.8 ramp note } then
  } script
  b 5.5 6.5 /quad-out tween 0.55 mul spotlight
} def
```
