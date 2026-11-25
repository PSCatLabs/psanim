# psanim library vocabulary

Generated from `crates/psanim/src/api.rs` by `scripts/gen_reference.sh`. Do not edit by hand.

The words of the libraries loaded with `use`. The chapters (stage.md, type.md, explain.md, ink.md) show them at work; this is the list. `psanim api NAME` prints one entry.

Motions (for `enters`, `leaves`, `onstage`, `kinetic` ...): /fade /rise /fall /slide-left /slide-right /pop /grow /zoom /drop /focus /spin /flip /unfold /unroll /wipe-right /wipe-left /wipe-up /wipe-down /mask-up /mask-down /mask-right

Emphases (for `emphasized`): /pulse /shake /wobble /jump /squash /blink /tada /dim

## pkstage: boxes, layout, themes, the camera by box

Load with `/pkstage use`.

### `canvasbox`

`-  canvasbox  box`

The whole canvas as a box, [0 0 width height]. A box is an array [x y w h], x y its bottom-left corner. Inside `within`, the enclosing box.

### `vmin`

`n  vmin  units`

n hundredths of the canvas's shorter side, in canvas units: a size that suits the frame whatever its shape. `4 vmin` is a comfortable text size, `6 vmin` a margin.

### `inset`

`box m  inset  box`

The box shrunk by m on every side; a negative m grows it.

### `insetxy`

`box mx my  insetxy  box`

The box shrunk by mx at the left and right and by my at the top and bottom.

### `subbox`

`box fx fy fw fh  subbox  box`

The part of a box that starts fx across and fy up and is fw of its width and fh of its height, all as fractions: `stage 0 0.6 1 0.4 subbox` is its top two fifths.

### `aspect`

`-  aspect  ratio`

The canvas's width divided by its height: above 1 is landscape. For a layout that changes with the frame's shape: `aspect 1 gt { wide } { tall } ifelse`.

### `shifted`

`box dx dy  shifted  box`

The box moved by dx, dy.

### `boxpoint`

`box /anchor  boxpoint  x y`

A point of the box: /c centre, /n /s /e /w the middles of its edges, /ne /nw /se /sw its corners, or [fx fy] as fractions across and up.

### `placed`

`box w h /anchor  placed  box`

A new box w by h, set inside the given box against that anchor: `canvasbox 600 200 /c placed` is centred, /sw sits in the bottom-left corner.

### `boxat`

`x y w h /anchor  boxat  box`

A box w by h whose anchor point is at (x, y): `640 360 300 120 /c boxat` is centred on that point.

### `cols`

`box n gap  cols  [ box ... ]`

Cut the box into n equal columns with gap between them, left to right.

### `rows`

`box n gap  rows  [ box ... ]`

Cut the box into n equal rows with gap between them, top to bottom.

### `cells`

`box ncols nrows gap  cells  [ box ... ]`

Cut the box into a grid of equal cells, row by row from the top left. gap is one number, or [gapx gapy] for different spacing across and down.

### `splitx`

`box f  splitx  left right`

Cut the box in two at fraction f of its width.

### `splity`

`box f  splity  top bottom`

Cut the box in two at fraction f of its height, measured from the top.

### `boxunion`

`boxA boxB  boxunion  box`

The smallest box that holds both: the frame around a group, or a camera target that takes in two things.

### `boxpath`

`box  boxpath  -`

Add the box to the current path as a rectangle.

### `roundpath`

`box r  roundpath  -`

Add the box to the current path with corners of radius r (limited to half its shorter side, so a large r makes a pill).

### `disc`

`x y r  disc  -`

Fill a circle in the current colour.

### `lookat`

`box  lookat  -`

Point the camera at a box: set the transformation so the whole box is in frame, centred, as large as fits. Wrap in gsave/grestore. The box can come from `keyframes`, `smoothkeys` or `between` over boxes, which is a camera move: `[ 0 whole  3 detail /cubic-inout ] keyframes lookat`.

### `visiblebox`

`-  visiblebox  box`

The part of the world the frame shows under the current transformation (the bounding box, if the view is rotated): test against it to skip drawing what is off screen.

### `themes`

`-  themes  dict`

The built-in themes by name: /midnight /paper /blueprint /terminal /poster /dusk. Each maps roles to colours (/bg /surface /line /fg /muted /accent /accent2 /accent3 /good /warn /bad), fonts (/title /body /mono /display /serif) and sizes (/corner /stroke).

### `theme`

`-  theme  dict`

The theme in force: read a role with `theme /accent get`, change one with `theme /accent [1 0.3 0.2] put` (in setup).

### `usetheme`

`/name  usetheme  -`

Make a built-in theme the one in force (in setup, or inside a scene for a part of it); or give a dictionary of roles to change just those: `<< /accent [1 0.3 0.2] /title /Bookman-Demi >> usetheme`.

### `rgbof`

`role  rgbof  r g b`

The colour of a theme role (or of an [r g b] array, unchanged) as three numbers.

### `ink`

`role  ink  -`

Set the colour by theme role: `/accent ink`. Every pk word that takes a colour takes a role name or an [r g b] array.

### `mixink`

`roleA roleB u  mixink  [ r g b ]`

The colour u of the way from one role (or [r g b]) to another: `/muted /accent 1 2 ramp mixink ink` is a colour change over time.

### `face`

`/role size  face  -`

Select a font by theme role at a size: /title /body /mono /display /serif. Any font name works too (`/Palatino-Italic 30 face`).

### `ground`

`-  ground  -`

Fill the canvas (or the enclosing `within` box) with the theme's background colour, and set the current colour to the theme's text colour (/fg), so text drawn without choosing a colour is visible. The first line of most scenes.

### `spotlight`

`box-or-boxes amount  spotlight  -`

Wash everything drawn so far, except the box (or an array of boxes), toward the theme's ground colour by amount 0..1 (darker on a dark theme, paler on a light one): directs the eye to one part of a busy picture. Animate the amount; draw afterwards anything that should stay bright.

## pkmotion: entrances, exits, emphasis

Load with `/pkmotion use`.

### `entering`

`spec proc  entering  -`

Draw proc arriving by a motion at the start of the current time scope. spec is /name, [/name seconds] or [/name seconds /easing]; motions: /fade /rise /fall /slide-left /slide-right /pop /grow /zoom /drop /focus /spin /flip /unfold /unroll /wipe-right /wipe-left /wipe-up /wipe-down /mask-up /mask-down /mask-right. The motion needs no anchor: the drawing is measured, so /pop scales it about its own centre. proc must contain everything it needs: operands left outside the braces stay on the stack while the thing is absent. Inside a `cascade` body, `def` the item and index first.

### `leaving`

`spec proc  leaving  -`

Draw proc departing by a motion, run backward, over the last seconds of the current time scope (so the scope must have an end: inside `during`, `thenfor`, `inbeat`).

### `enters`

`t spec proc  enters  -`

Nothing before t; at t proc arrives by the motion; then it stays. Inside proc the clock starts from 0 at t (read a film-wide value with `globally`). The usual way to bring something on: `1.5 /pop { ... } enters`.

### `leaves`

`t spec proc  leaves  -`

proc is there until t, departing by the motion (run backward) so that it is gone at t.

### `onstage`

`a b specin specout proc  onstage  -`

proc arrives at a by one motion and has left by b by another; inside, the clock runs from 0. A whole appearance in one line: `2 6 /rise /fade { ... } onstage`.

### `staged`

`specin specout proc  staged  -`

proc arrives by one motion at the start of the current time scope and departs by another at its end: a whole appearance inside `inbeat`, `thenfor` or `during`. `/rise /fade { ... } staged`.

### `emphasized`

`t spec proc  emphasized  -`

proc is drawn throughout; at t it performs an emphasis and returns to rest. Emphases: /pulse /shake /wobble /jump /squash /blink /tada /dim. Nest it around an entrance to do both.

### `motion`

`u spec proc  motion  -`

Draw proc at stage u of a motion, driven by you: 0 is absent and 1 in place for an entrance, 0 and 1 both in place for an emphasis. For a motion tied to a signal, a scroll position or your own tween. The spec's time and easing are ignored.

### `swapped`

`t spec procA procB  swapped  -`

A until t, B from t: A leaves by the motion (gone at t) and B enters by it. One thing replacing another in the same place. Inside B the clock starts from 0 at t; inside A it is the outer clock.

### `motions`

`-  motions  dict`

Your own motions: put a procedure there that takes u and returns a dictionary of any of /dx /dy (canvas units) /dxrel /dyrel (multiples of the drawing's own width and height) /scale /sx /sy /rotate /alpha /blur /wipe [l b r t] (the part of its box shown) /mask (clip to its resting box) /pivot [fx fy]. Then use its name wherever a motion goes.

### `motionnames`

`-  motionnames  [ /name ... ]`

The names of the built-in motions, entrances first, for a catalogue or a test.

### `rank`

`i n /order  rank  j`

Where item i of n comes in a cascade: /forward, /reverse, /center (the middle first), /edges (the ends first) or /random. Use the result as the index for `staggered`.

### `echoed`

`n gap proc  echoed  -`

Draw proc as it was at n earlier moments, gap seconds apart, oldest faintest, then as it is now: a trail behind anything that moves. Works because proc is a function of its clock; there is no state.

## pktype: text in boxes, kinetic type, captions

Load with `/pktype use`.

### `fontsize`

`-  fontsize  size`

The size of the current font.

### `showright`

`(text)  showright  -`

Show the string so that it ends at the current point: right-aligned, which keeps a changing number steady.

### `sym`

`/name  sym  -`

Show a mathematical sign or Greek letter at the current point at the current font's size, keeping the font: /neq /le /ge /approx /pm /times /div /dot /infinity /degree /arrow /larrow /uparrow /downarrow /implies /sum /product /sqrt /integral /in /bullet, and /alpha /beta /gamma /delta /epsilon /theta /lambda /mu /pi /rho /sigma /tau /phi /omega /Delta /Sigma /Pi /Omega. The text fonts have no such characters: a non-ASCII character in a string shows as a blank. `(kiwi ) show /neq sym ( plum) show`.

### `symwidth`

`/name  symwidth  w`

The width `sym` advances by for that sign, for centring a line that contains one.

### `symbols`

`-  symbols  dict`

The signs `sym` knows, by name, as strings in the Symbol font.

### `showruns`

`runs  showruns  -`

Show a line of mixed type at the current point, in the current font and colour. A run list is an array of (text), /name (a sign from `symbols`: /pi /times ...), [/it (r)] (italic, as variables are set), [/sup x], [/sub x] and [/frac a b]: `[ (Area = ) /pi [/it (r)] [/sup (2)] ] showruns`.

### `showrun`

`runs i  showrun  -`

Show only run i of a run list, at the place it has in the line that starts at the current point: an equation brought in term by term, each term with its own time and motion. The current point is left for the next call to start from only if you reset it: `moveto` before each.

### `runwidth`

`runs  runwidth  w`

The width of a run list in the current font, for centring it.

### `runbox`

`runs i  runbox  box`

The box of run i in the line that starts at the current point: the target for a `marker`, a `circled`, a `note` or a `spotlight` on one term of an equation.

### `textbox`

`(text)  textbox  box`

The box the string would cover if shown at the current point in the current font: for a plate behind it, a marker over it, a camera move onto it.

### `partbox`

`(text) (part)  partbox  box`

The box of the first occurrence of part within text, were text shown at the current point: to mark, underline or point at one word of a line.

### `showin`

`(text) box /anchor  showin  -`

Show the string inside a box, set against an anchor: /c centres it both ways, /w sets it left and vertically centred, /nw top left.

### `placetext`

`(text) box /anchor  placetext  -`

Move to where `showin` would start the string, without showing it: the way to put `kinetic`, `typewrite`, `showruns` or a `charpath` in a box. `(TITLE) box /c placetext (TITLE) /glyphs 0.05 /rise kinetic`.

### `fitface`

`(text) box /role  fitface  -`

Select the role's font (or any font name) at the largest size at which the string's width fits the box's width and its capital height the box's height: a headline that fills its column whatever the words.

### `wraplines`

`(text) width  wraplines  [ (line) ... ]`

Break text into lines no wider than width in the current font; \n forces a break.

### `textlayout`

`(text) box opts  textlayout  [ [ (line) x y ] ... ]`

Wrap text into a box and return each line with where it goes. opts is /left, /center or /right, or << /Align /left /Leading 1.3 /VAlign /top >> (/VAlign /top /middle /bottom).

### `paragraph`

`(text) box opts  paragraph  -`

Set wrapped text in a box. opts as for `textlayout`.

### `eachword`

`(text) proc  eachword  -`

Like `eachglyph`, by words: proc runs once per word with `(word) i` on the stack and the current point at the word's origin.

### `kinetic`

`(text) /unit gap spec  kinetic  -`

Show text at the current point with each /glyphs, /words or the /whole entering by a motion, gap seconds apart: `(MOVE) /glyphs 0.05 /mask-up kinetic`. Its clock is the current scope's: wrap it in `after` or `then` to start it later, in `leaving` to take it away.

### `kineticparagraph`

`(text) box opts /unit gap spec  kineticparagraph  -`

A wrapped paragraph whose /glyphs, /words or /lines enter one after another by a motion. opts as for `textlayout`.

### `marker`

`box u  marker  -`

A highlighter bar over the first u (0..1) of a box, in the current colour: draw it before the text it marks, with the box from `textbox` or `partbox`. Keeps the current point.

### `underline`

`box u  underline  -`

A rule under a box, drawn on as u goes 0 to 1, in the current colour and line width. Keeps the current point.

### `strikethrough`

`box u  strikethrough  -`

A straight line through the middle of a box, drawn on as u goes 0 to 1, in the current colour and line width: a word struck out. Use it with `textbox` or `partbox`. Keeps the current point. pkdiagram's `struck` is the hand-drawn version.

### `typewrite`

`(text) u  typewrite  -`

Show the first u (0..1) of the string with a block caret after it; the caret blinks once the typing is done.

### `captions`

`[ [ t0 t1 (text) ] ... ] box opts  captions  -`

Subtitles: show the caption whose time has come, centred at the bottom of the box on a plate, fading in and out. opts: << /Size s /Role /body /Plate true /Karaoke false >>; with /Karaoke the words light up across the caption's time. Times are on the current scope's clock.

## pkdiagram: nodes, connectors, flows, a presenter's marks

Load with `/pkdiagram use`.

### `node`

`(label) box opts  node  -`

Draw a labelled node filling a box. opts is a fill role (/surface, /accent) or << /Fill /surface /Stroke /line /Ink /fg /Shape /box /Sub (second line) /Size s /Font /title >>; shapes: /box /pill /circle /diamond /cylinder /plain. The label is sized to fit.

### `group`

`(label) box opts  group  -`

Draw a container: a dashed outline around a box with its name on the top edge. opts is a role for the line, or << /Stroke /muted /Ink /muted /Size s >>. `boxA boxB boxunion -30 inset` is the box around two nodes.

### `connect`

`boxA boxB /route  connect  -`

Make the current path a connector from the edge of box A to the edge of box B. Routes: /straight, /elbow (right angles, sideways or vertical as the boxes' places call for), /hv and /vh (one corner: across then up, or up then across), /arc and /arc-right (a curve bulging left or right of the direction of travel), [/sideA /sideB] to name the side of each box it leaves and arrives by ([/s /n] goes down, across and down), or [ x y x y ... ], points to pass through (to go round a node). Being a path, it takes `trimpath`, `onpath`, `pathpoint`, `arrow`, `pulse`.

### `connectgap`

`-  connectgap  units`

The space left between a box and a connector's end (6). Redefine it in setup to change it.

### `arrow`

`u  arrow  -`

Stroke the first u (0..1) of the current path with an arrowhead riding its tip, in the current colour and line width: an arrow that draws itself. Consumes the path.

### `pulse`

`u len  pulse  -`

A comet at u (0..1) along the current path, len of the path long, in the current colour: a message in flight. Draws nothing for u outside 0..1. Keeps the path, so several can ride one connector.

### `pulsetrain`

`period n  pulsetrain  [ u ... ]`

n evenly spaced positions, each travelling 0 to 1 every period seconds: a steady flow for an edge's /Pulse.

### `edge`

`boxA boxB opts  edge  -`

Draw a connector between two boxes. opts is a route name, or << /Route /elbow /Draw u /Label (text) /Pulse u-or-array /Role /line /Head true /Dash false /Width w /LabelRole /muted /LabelSize s /PulseRole /accent /PulseLen 0.18 >>. Animate /Draw to draw it on; the label fades in over the second half; /Pulse puts comets at those places once it is drawn (a place of exactly 0 or 1, or outside, draws none, so a clamped tween shows one message and then nothing).

### `circled`

`box  circled  -`

Make the current path a loose hand-drawn ring around a box. Draw it on: `0 u trimpath stroke`.

### `struck`

`box  struck  -`

Make the current path a strike-through line across a box.

### `bracepath`

`box /side  bracepath  -`

Make the current path a curly brace along one side of a box (/n /s /e /w), opening away from it.

### `bracepoint`

`box /side  bracepoint  x y`

The tip of the brace `bracepath` draws on that side: where its label goes.

### `note`

`(text) box /side u  note  -`

A label beside a box with a leader line to it: the line draws on and the text fades in as u goes 0 to 1. /side is where the label sits (/n /s /e /w /ne /nw /se /sw). Uses the current font and colour.

## pkcode: source code on screen

Load with `/pkcode use`.

### `codeblock`

`lines box opts  codeblock  -`

Draw tokenized code (from `codetokens`) in a box as a panel. opts: << /Reveal u (typed so far, 0..1) /Focus [ line ... ] (keep these bright, dim the rest) /FocusAmount a (0..1, animate it) /Scroll n (lines scrolled off the top) /Numbers bool /Title (file.rs) /Size s /Leading 1.4 /Plate true >>. The size is chosen to fit unless given.

### `codelinebox`

`lines box opts n  codelinebox  box`

The box where line n (from 0) of a code block is drawn, given the same lines, box and opts: for notes, arrows, circles and camera moves onto a line.

### `codecolors`

`-  codecolors  dict`

Which theme role each kind of token is drawn in (/keyword /string /number /comment /type /call /name /punct /plain). Change an entry in setup to restyle.

### `diffrows`

`(old) (new) /language  diffrows  rows`

A line diff with each line tokenized, for `codediff`. Call it in setup.

### `codediff`

`rows box opts u  codediff  -`

Draw a change to code: the old version at u = 0 becomes the new at u = 1. Removed lines redden and close up, then added lines open in green, and unchanged lines slide to their new places. opts as for `codeblock` (/Title /Size /Leading /Plate).

## pkchart: data that moves

Load with `/pkchart use`.

### `plotframe`

`box [ x0 x1 ] [ y0 y1 ]  plotframe  -`

Say where the plot is and what range of data it spans. Every pkchart word after it draws in that frame. Call it in the scene (or in setup, if there is one chart).

### `plotbox`

`-  plotbox  box`

The current plot frame's box.

### `plotpoint`

`x y  plotpoint  X Y`

Data coordinates to canvas coordinates, in the current plot frame: for placing a note, a marker or a label at a data point.

### `plotaxes`

`opts  plotaxes  -`

Draw the axes, ticks, labels and grid of the current plot frame. opts: << /Draw u (drawn on, 0..1) /YTicks 5 /XTicks 5 /Grid true /Decimals 0 /XDecimals 0 /XNames [ (a) ... ] (names under evenly spaced slots, for bars) /XValues [ ... ] /YValues [ ... ] (ticks exactly there) /Fade true (while the range moves, ticks cross-fade between round steps instead of jumping) /Suffix (%) /XLabel (text) /YLabel (text) /Size s >>. The plot frame's box is the data rectangle; labels and titles are drawn outside it, about three times /Size to the left and below, so inset the box from the page.

### `bars`

`[ v ... ] opts  bars  -`

Columns across the plot frame, one per value, on the frame's y scale. opts: << /Grow u (bars rise one after another as u goes 0..1) /Stagger 0.6 /Role /accent or /Roles [ ... ] /Dim [ i ... ] (draw these muted) /Accent [ i ... ] (draw all but these muted) /Gap 0.3 /Values true /Count false (true: labels count up with the bars; otherwise they show the true value) /Decimals 0 /Prefix () /Suffix () /Size s >>. To move between datasets, interpolate the values: `u A B between`.

### `hbars`

`[ v ... ] opts  hbars  -`

Rows down the plot frame, on the frame's x scale. opts as for `bars`, plus /Names [ (a) ... ] written left of the rows and /Ranks [ r ... ], the row each bar sits in: `u A ranks B ranks between` slides bars past each other as their order changes.

### `plotline`

`[ [ x y ] ... ]  plotline  -`

Make the current path the line through the data points. Stroke it; `0 u trimpath` first draws it on.

### `seriesline`

`[ y ... ]  seriesline  -`

Like `plotline` for values spread evenly across the frame's x range.

### `plotarea`

`-  plotarea  -`

Close the current line (from `plotline`) down to the bottom of the plot frame, to fill the area under it.

### `dots`

`[ [ x y ] ... ] u radius  dots  -`

A dot at each data point, popping in one after another as u goes 0 to 1, in the current colour.

### `donut`

`[ v ... ] box opts  donut  -`

A ring of shares in a box, swept on clockwise from the top. opts: << /Grow u /Roles [ ... ] /Inner 0.62 /Gap 2 /Center (text) /CenterRole /fg >>.

### `inplot`

`proc  inplot  -`

Run proc clipped to the plot frame. Lines are not clipped by themselves, so use it whenever data can leave the frame's range, which is always when the range is animated (a zoom is `u [0 365] [0 80] between` as the x range).

### `plotrule`

`opts  plotrule  -`

A reference line across the plot frame at a data value: << /Y 50 (or /X v) /Label (even odds) /Draw u /Dash true /Role /muted /Size s >>.

### `plotmark`

`x y opts  plotmark  -`

Mark a data point: a dot, dashed drop lines to both axes, and a label on a leader line. opts: << /Label (23 people) /Side /ne /Draw u /Drop true /Role /accent /Radius r /Size s >> (/Side is where the label sits: /n /s /e /w /ne /nw /se /sw; /Size defaults to 3 vmin). The one-call form of the annotation every data story needs.

### `chartroles`

`-  chartroles  [ /role ... ]`

The roles marks cycle through when none are given.

### `thresholdrole`

`v [ role v1 role v2 role ... ]  thresholdrole  role`

The role for a value by thresholds, the way dashboards colour status: the first role below v1, the next from v1 up, and so on. `v [ /good 70 /warn 90 /bad ] thresholdrole ink`.

### `sparkline`

`[ v ... ] box opts  sparkline  -`

A small line chart filling the box, no axes: the shape of a recent past. opts: << /Range [lo hi] (the data's own, padded) /Role /accent2 /Fill 0.18 (opacity of the shading under it; 0 for none) /Width w /Dot true (a dot on the newest value) /Draw u (the part drawn, from the left) /From 0 /To 1 (where the first and last values sit, as fractions of the box's width, the rest clipped: for a window read on a fixed grid that slides by part of a step) >>.

### `horizon`

`[ v ... ] box opts  horizon  -`

A horizon chart in the box: the series folded into bands of colour, darker the higher, so a row a few pixels high still shows its shape and a fleet of rows fits on a screen. Values above /Base rise from the bottom in /Role; below it, they hang from the top in /NegRole. opts: << /Extent e (the value that fills every band; the data's largest by default) /Bands 3 /Base 0 /Role /accent2 /NegRole /accent3 /Ground /surface (the colour the bands rise from) /Mirror false (negatives from the bottom too) /Draw u (the part drawn, from the left) >>. The catalog's /horizon draws many rows with a cursor.

### `heatmap`

`[ [ v ... ] ... ] box opts  heatmap  -`

A matrix as cells in the box, first row at the top, coloured from a low role to a high one. opts: << /Range [lo hi] (the data's) /Roles [ /surface /accent ] (low high, or low mid high) /Gap g /Draw u (columns brought in from the left) >>.

### `statuslight`

`x y r role  statuslight  -`

A status lamp: a dot of radius r in the role with a soft halo and a glint. With `thresholdrole`, a status light: `x y 8 v [ /good 70 /warn 90 /bad ] thresholdrole statuslight`.

### `panel`

`box opts  panel  box`

A dashboard card: the theme's surface with rounded corners and a title; leaves the box inside it for the contents. opts: << /Title (text) /Status role (a lamp at the top right) /Fill /surface /Size s (title size; a fifteenth of the card's width, at most a seventh of its height) /Pad p >>.

### `stattile`

`v box opts  stattile  -`

A stat panel: a card with a title, the value large in its threshold's colour, its unit, a change with an up or down arrow, and a sparkline of its recent past along the bottom. opts: << /Title (text) /Unit (%) /Decimals 0 /Short false (12.4k instead of 12,431) /Thresholds [ /good 70 /warn 90 /bad ] (default /fg) /Delta d (the change to show) /DeltaDecimals 1 /DeltaSuffix (%) /DownIsGood false /Spark [ v ... ] /SparkRange [lo hi] /SparkFrom 0 /SparkTo 1 (as sparkline's /From /To) /Fill /surface, or /threshold to colour the card instead of the number >>.

### `dial`

`v box opts  dial  -`

A gauge: an arc from bottom left round to bottom right, filled to the value in its threshold's colour, the thresholds as a thin band round it, the number in the middle. opts: << /Range [0 100] /Thresholds [ /good 70 /warn 90 /bad ] (default all /accent) /Sweep 240 /Thick 0.17 (of the radius) /Unit (%) /Decimals 0 /Short false /Label (text, under the number) /Track /line >>.

## pkink: marks with a hand in them

Load with `/pkink use`.

### `revealalong`

`u width proc  revealalong  -`

Run proc clipped to a band of that width along the first u (0..1) of the current path: proc draws a whole mark (a paintkit brush stroke, a chalk line, a hatched ribbon) and only the part a pen moving along the path has reached shows. A textured stroke that draws itself. Consumes the path.

### `revealband`

`u0 u1 width proc  revealband  -`

Like `revealalong`, for the stretch of the path between u0 and u1 only. On a persistent canvas give it where the pen was a frame ago and where it is now, so a translucent medium is laid down once and not again on every frame.

### `around`

`n proc  around  -`

Run proc n times, each turned 360/n further about the origin, with the copy's index on the stack: petals, spokes, ticks.

### `brushstroke`

`[ w ... ]  brushstroke  -`

Fill the current path as a stroke whose width follows the list, in the current colour: `taperpath fill`.

### `boxgradient`

`box c0 c1 angle  boxgradient  -`

Fill a box with a gradient between two colours (roles or [r g b]) along an angle: 0 runs left to right, 90 bottom to top. `canvasbox /bg /surface 90 boxgradient` is a ground.

### `jitter`

`key amount  jitter  dx dy`

A small steady offset within amount either way, fixed by key: the hand-set unevenness of a row of marks. The same key always gives the same offset.

