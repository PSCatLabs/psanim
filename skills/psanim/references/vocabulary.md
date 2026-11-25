# psanim vocabulary

Generated from `crates/psanim/src/api.rs` by `scripts/gen_reference.sh`. Do not edit by hand.

Stack effects read left to right, topmost operand last, as in the PostScript Language Reference: `a b /name  tween  u` takes three operands and leaves one.

Easings (for `ease`, `tween`, `keyframes`): linear smooth smoother quad-in quad-out quad-inout cubic-in cubic-out cubic-inout quart-in quart-out quart-inout quint-in quint-out quint-inout sine-in sine-out sine-inout expo-in expo-out expo-inout circ-in circ-out circ-inout back-in back-out back-inout elastic-in elastic-out elastic-inout bounce-in bounce-out bounce-inout spring hold

Transitions (for `reel`): cut dissolve dip wipe-left wipe-right wipe-up wipe-down iris push-left push-right zoom

## Declaring the movie

### `animation`

`dict  animation  -`

Declare the movie. Put it before anything that takes time or draws (definitions it needs, such as a reel array for `reelduration`, may come first). Keys, all optional: /Size [w h] (canvas units, default [1280 720]), /Duration seconds (5), /FPS (30), /Seed (1), /Background [r g b] or /transparent (black), /Title, /Description, /Loop bool, /Scene /name (scene), /Step /name (step), /StepRate ticks per second (FPS), /Persist bool, /MotionBlur samples, /Shutter 0..1 (0.5).

## The clock

### `now`

`-  now  t`

The current time in seconds, in the innermost time scope. At the top of a scene it is the movie time.

### `progress`

`-  progress  u`

How far through the current time scope's window `now` is, 0 to 1, clamped. At the top of a scene: now / duration.

### `span`

`-  span  seconds`

The length of the current time scope's window.

### `globaltime`

`-  globaltime  t`

The movie time in seconds, whatever time scopes are open.

### `frame`

`-  frame  int`

The index of the frame being drawn (the tick index inside `step`).

### `fps`

`-  fps  num`

The movie's frame rate.

### `dt`

`-  dt  seconds`

The length of one frame; inside `step`, the length of one simulation tick.

### `duration`

`-  duration  seconds`

The movie's length.

### `width`

`-  width  num`

The canvas width in units.

### `height`

`-  height  num`

The canvas height in units.

### `movieseed`

`-  movieseed  int`

The movie's seed.

### `turns`

`k  turns  degrees`

k whole turns spread over the current window: k * 360 * (now / span). The angle for anything that must come home when a loop closes; k must be a whole number (or a whole number divided by the part's rotational symmetry).

## Time scopes: re-basing the clock for a procedure

### `during`

`a b proc  during  -`

Run proc only while a <= now < b. Inside, now runs from 0 to b-a and progress from 0 to 1.

### `after`

`a proc  after  -`

Run proc only once now >= a. Inside, now starts from 0.

### `before`

`b proc  before  -`

Run proc only while now < b. Inside, progress runs 0 to 1 over 0..b.

### `delayed`

`d proc  delayed  -`

Run proc with its clock d seconds behind: whatever it does happens d seconds later.

### `staggered`

`i gap proc  staggered  -`

Run proc delayed by i * gap seconds: item i of a row starts gap after item i-1.

### `timescaled`

`k proc  timescaled  -`

Run proc with its clock running k times as fast (k < 1 is slow motion).

### `looped`

`period proc  looped  -`

Run proc with now wrapping to 0 every period seconds; progress runs 0 to 1 each cycle.

### `pingpong`

`period proc  pingpong  -`

Run proc with now running 0 to period, back to 0, and so on.

### `reversed`

`proc  reversed  -`

Run proc with now running backward through the current window.

### `globally`

`proc  globally  -`

Run proc with now set back to the movie time, whatever time scopes are open. For a value shared by things in different scopes (a sun that the whole film sees): `{ [ ... ] keyframes } globally`.

### `attime`

`t proc  attime  -`

Run proc with now frozen at t: a still of a moving thing.

### `timewindow`

`t dur proc  timewindow  -`

Run proc with now set to t in a window dur seconds long.

### `sequence`

`[ dur proc dur proc ... ]  sequence  -`

Play procs one after another, each for its duration, each with its own clock from 0. A proc may be given by name (/intro); `null` in place of a proc is a gap.

## Values over time

### `ease`

`x /name  ease  y`

Pass x (clamped to 0..1) through an easing curve. Names: linear smooth smoother hold spring, and quad cubic quart quint sine expo circ back elastic bounce each with -in, -out, -inout (as /cubic-out).

### `tween`

`a b /name  tween  u`

The eased progress of now from a to b: 0 before a, 1 after b. The usual way to animate anything: `1 2.5 /cubic-out tween 100 400 between`.

### `ramp`

`a b  ramp  u`

The linear progress of now from a to b, clamped to 0..1.

### `between`

`u v0 v1  between  v`

Interpolate from v0 (at u=0) to v1 (at u=1). v0 and v1 are numbers, or arrays of numbers of one length (points, colours); u is not clamped, so an overshooting easing overshoots.

### `keyframes`

`[ t v /ease t v ... ]  keyframes  v`

The value at now of a track through the keys. Each key is a time and a value (number or array), optionally followed by the easing used to arrive at it. Holds the first value before the first key and the last after the last.

### `keyat`

`t [ t v /ease ... ]  keyat  v`

Like keyframes, at time t instead of now.

### `clamp`

`x lo hi  clamp  y`

x limited to lo..hi.

### `fmod`

`x m  fmod  r`

x modulo m for reals, always in 0..m (PostScript's mod is integers only): the wrap-around every repeating particle needs.

### `smoothstep`

`e0 e1 x  smoothstep  u`

0 below e0, 1 above e1, a smooth S-curve between.

### `remap`

`x a b c d  remap  y`

Map x from the range a..b to the range c..d, linearly, unclamped.

### `wiggle`

`key freq amp  wiggle  v`

Smooth random motion: a value wandering in -amp..amp about freq times a second. Different keys (a number, or an array of numbers) give independent wanderings; the same key always gives the same one.

### `loopnoise`

`key radius  loopnoise  v`

Smooth random motion in -1..1 (key is a number or an array of numbers) that returns exactly to its start at the end of the current window: noise sampled around a circle. A larger radius wanders more. For organic drift in a seamless loop, where wiggle would not close.

### `hash01`

`key  hash01  u`

A repeatable random number in 0..1 determined by key (a number or an array of numbers) and the movie seed. Stateless: the same key gives the same value on every frame, which keeps per-particle properties from flickering.

### `smoothkeys`

`[ t v t v ... ]  smoothkeys  v`

The value at now of a smooth track through the keys (numbers, or arrays of one length: points, boxes, colours): it passes through every key without stopping or turning a corner, and starts and ends at rest. For camera paths and anything that should flow; use `keyframes` where each move should ease in and out on its own.

### `valueat`

`array x  valueat  v`

The array read at a fractional index: element x where x is whole, the linear blend of its neighbours between (clamped to the ends). For a readout that follows a table: `table people valueat 1 fmtnum show` while `people` is a tween.

### `ranks`

`[ v ... ]  ranks  [ r ... ]`

The rank of each value, 0 for the largest. Interpolating two rank arrays with `between` slides rows past each other: a bar-chart race.

### `niceticks`

`lo hi n  niceticks  [ v ... ]`

About n round numbers covering lo..hi (steps of 1, 2, 2.5 or 5 times a power of ten): tick positions for an axis.

## Paths: measuring, following, drawing on, morphing

### `pathlength`

`-  pathlength  len`

The length of the current path, all subpaths together.

### `pathpoint`

`u  pathpoint  x y angle`

The point u (0..1) of the way along the current path, and the direction of travel there in degrees. Does not change the path. For moving something along a curve: `u pathpoint 3 1 roll translate rotate`.

### `trimpath`

`u0 u1  trimpath  -`

Replace the current path by the part of it between fractions u0 and u1 of its length, subpaths taken one after another. `0 u trimpath stroke` draws a path on as u goes 0 to 1.

### `trimeach`

`u0 u1  trimeach  -`

Like trimpath, but every subpath is trimmed to the same fractions of its own length, so all of them grow together.

### `resamplepath`

`n  resamplepath  array`

n points evenly spaced along the current path, as [x0 y0 x1 y1 ...], in the user space current now (y up, like every coordinate). Does not change the path.

### `polypath`

`array  polypath  -`

Replace the current path by the open polyline through [x0 y0 x1 y1 ...], in the current user space. Follow with closepath for a polygon. The fast way to replay geometry computed in setup: build the array once, then translate/rotate and polypath it each frame.

### `morphpath`

`arrayA arrayB u  morphpath  -`

Replace the current path by the closed polygon u of the way from A to B, two arrays from resamplepath with the same n. The points are placed in the current user space, so the transformation in force applies to them as to any path.

### `onpath`

`u proc  onpath  -`

Run proc with the origin at the point u (0..1) of the way along the current path and the x axis along the direction of travel: draw the thing that rides the path as if at 0 0 facing right. The path is kept.

### `straightenpath`

`u anchor  straightenpath  -`

Unbend the current path by u (0 as it is, 1 a straight line), keeping its length: every turn is scaled by 1 - u. The point anchor (0..1 of the way along) stays where it is, with its direction. An arc that unrolls into a strip, a circle's circumference laid flat, a rope pulled taut: `morphpath` would shrink them on the way.

### `taperpath`

`[ w ... ]  taperpath  -`

Replace the current path by the outline of a stroke along it whose width follows the list, spread evenly along its length: [12 0] is a comet, [0 9 0] a brush mark. Fill the result.

## Layers and effects

### `beginlayer`

`-  beginlayer  -`

Start drawing on a separate transparent layer. The graphics state is unchanged.

### `endlayer`

`alpha  endlayer  -`

Finish the layer and composite it over what is beneath at opacity alpha.

### `endlayerblend`

`alpha /mode  endlayerblend  -`

Finish the layer and composite it with a blend mode: /normal /add /screen /multiply.

### `blurlayer`

`radius  blurlayer  -`

Blur everything drawn on the current layer so far (Gaussian, radius in canvas units).

### `glowlayer`

`radius strength  glowlayer  -`

Add a soft halo around everything on the current layer: a blurred copy added back, strength 0..2. The halo's light is the shape's own, spread out: a thin line needs a larger radius and a strength above 1 to show.

### `faded`

`alpha proc  faded  -`

Draw proc as one group at opacity alpha. Unlike setalpha, overlapping shapes inside do not show through each other.

### `blended`

`alpha /mode proc  blended  -`

Draw proc as a group composited with a blend mode.

### `blurred`

`radius proc  blurred  -`

Draw proc out of focus.

### `glowing`

`radius strength proc  glowing  -`

Draw proc with a halo of light. Try 8 1 for solid shapes, 24 1.6 for thin strokes.

### `crossfade`

`procA procB u  crossfade  -`

Draw A, then B over it at opacity u.

### `vignette`

`amount  vignette  -`

Darken the corners of everything drawn so far, 0..1.

### `filmgrain`

`amount  filmgrain  -`

Add film grain to everything drawn so far, 0..1. The grain changes every frame.

## The canvas and the camera

### `backdrop`

`r g b  backdrop  -`

Fill the canvas rectangle (0 0 width height) with a colour, in the current coordinates and inside the current clip. Call it first, before any camera. A clip that starts with a backdrop can be wiped, pushed or irised by a reel's transitions.

### `camera`

`cx cy zoom angle  camera  -`

Set the transformation so the world point (cx, cy) is at the centre of the frame, magnified by zoom and rotated by angle. Wrap in gsave/grestore.

## Text

### `typed`

`string u  typed  string`

The first u (0..1) of the string, by characters: a typewriter reveal.

### `showcenter`

`string  showcenter  -`

Show the string centred horizontally on the current point.

### `eachglyph`

`string proc  eachglyph  -`

Run proc once per character with `(c) i` on the stack (the character as a one-character string, and its index) and the current point at that character's origin. The font must be set. proc consumes both operands; `{ pop show }` reproduces show. The current point is set afresh before each call, so proc may move it, build paths and paint; graphics state it changes (colour, CTM) stays changed, so wrap transforms in gsave/grestore.

### `spacedwidth`

`string track  spacedwidth  w`

The width of the string when letterspaced: track extra units after each character but the last. For centring text drawn with eachspaced.

### `eachspaced`

`string track proc  eachspaced  -`

Like eachglyph, letterspaced: each character's origin is track units further along than its natural place. `(TITLE) 6 { pop show } eachspaced` sets spaced capitals.

### `eachonpath`

`string u0 proc  eachonpath  -`

Set the string along the current path, starting u0 (0..1) of the way along. proc runs per character with `(c) i` on the stack, in coordinates whose origin is the character's place on the path and whose x axis follows the path, with the current point at the character's origin. The path is left as it was. `{ pop show }` sets the text.

### `strcat`

`(a) (b)  strcat  (ab)`

Join two strings.

### `fmtnum`

`num decimals  fmtnum  string`

The number as text with that many decimal places and commas between thousands: 1234567.8 1 fmtnum is (1,234,567.8). For counters: `1 3 /cubic-out tween 0 48213 between 0 fmtnum show`.

### `fmtfixed`

`num decimals  fmtfixed  string`

The number as text with that many decimal places and no thousands separators (years, versions).

### `zeropad`

`int width  zeropad  string`

The integer as text, padded with zeros on the left to that width: 7 2 zeropad is (07).

### `fmtclock`

`seconds  fmtclock  string`

Seconds as a clock: (01:05), or (1:01:05) from an hour up.

### `splitwords`

`(text)  splitwords  [ (word) ... ]`

The words of a string, split at spaces and line breaks.

### `splitlines`

`(text)  splitlines  [ (line) ... ]`

The lines of a string, split at \n.

### `codetokens`

`(source) /language  codetokens  lines`

Source code as data, one array per line of alternating text and kind: [ (fn) /keyword ( ) /plain (main) /call ... ]. Kinds: /plain /keyword /string /number /comment /type /call /name /punct. Languages: /rust /python /javascript /typescript /c /go /postscript /shell /json /plain. Call it in setup; draw with pkcode.

### `difflines`

`(old) (new)  difflines  rows`

A line diff as data: an array of [ /same|/del|/add (line) oldindex newindex ] rows, in the order they would be printed.

## Reels: clips and transitions

### `reel`

`[ [dur proc] [dur /transition] [dur proc] ... ]  reel  -`

Play clips in order with transitions between them. A transition overlaps the end of the clip before it and the start of the clip after it. Transitions: /cut /dissolve /dip /wipe-left /wipe-right /wipe-up /wipe-down /iris /push-left /push-right /zoom. A clip need not paint a ground: clips that draw only their own subject transition over whatever the scene drew before the reel.

### `reelduration`

`[ ... ]  reelduration  seconds`

The total length of a reel array: the clips' durations added up, minus the transitions' (a transition overlaps the clips on both sides, so a clip's own duration includes the transitions into and out of it). Use it for /Duration: define the reel array first, then `<< /Duration film reelduration ... >> animation`.

### `transitions`

`-  transitions  dict`

The dictionary of transitions. To define your own, put a procedure there: `transitions /t-shutter { ... } put`, then use `[dur /t-shutter]` in a reel. Name it with a `t-` prefix, or define it before the reel is measured with reelduration: otherwise the entry is taken for a clip. It is called with `procA procB u`: draw the frame that is u (0 to 1, linear) of the way from A to B. A and B are ready-made procedures that draw each clip with its own clock still running; call them inside gsave/grestore with whatever clip, transform or layer the transition needs.

## Parameters, signals and events

### `param`

`/name default  param  value`

A value the host supplied for name (`--param name=value`), or default. Call it in setup.

### `signal`

`/name  signal  num`

The value of the host's signal name at the current movie time; 0 if there is none.

### `sinceevent`

`/name  sinceevent  seconds`

Seconds since the most recent event on track name; a million if there has been none.

### `eventcount`

`/name  eventcount  int`

How many events on track name have happened so far.

## Viewports and libraries

### `within`

`box proc  within  -`

Run proc inside the box [x y w h]: the origin moves to the box's corner, drawing is clipped to it, and `width` and `height` report the box's size. The spatial twin of `during`: a scene written against width and height can be placed in any box (a panel, one half of a comparison, a picture in a picture).

### `fitted`

`box w h proc  fitted  -`

Run proc, which draws for a canvas w by h, scaled to fit inside the box and centred there, clipped. Inside, `width` and `height` are w and h. For placing a finished scene in a panel whatever its aspect.

### `use`

`/library  use  -`

Load one of psanim's libraries, once: /pkstage /pkmotion /pktype /pkdiagram /pkcode /pkchart /pkink. Call it in setup, after `animation`. A library loads the ones it needs. `psanim api --libs` lists them; `psanim api --lib pkmotion` lists one's words.

## Storyboards, scripts and cascades

### `storyboard`

`[ /name seconds /name seconds ... ]  storyboard  -`

Name the stretches of the film, in order. Call it before `animation` and use `storylength` for /Duration; then place things by beat name instead of by second, so retiming the film is editing one list.

### `storylength`

`-  storylength  seconds`

The total length of the storyboard.

### `beat`

`/name  beat  t`

The movie time at which the named beat starts. Add to it for a moment inside the beat: `/build beat 0.5 add`.

### `beatspan`

`/name  beatspan  a b`

The start and end of the named beat, on the movie's clock: operands for `during` at the top level of a scene.

### `inbeat`

`/name proc  inbeat  -`

Run proc during the named beat, with the clock starting from 0 at the beat's start and `span` its length, whatever time scopes are open.

### `frombeat`

`/name proc  frombeat  -`

Run proc from the start of the named beat to the end of the film, with the clock starting from 0 there: for something a beat introduces that then stays.

### `script`

`proc  script  -`

Run proc as a script: a cursor starts at 0 and each `then`, `thenfor` and `wait` inside moves it on, so things happen one after another without any times written down. Scripts nest; the cursor outside is untouched.

### `then`

`seconds proc  then  -`

In a script: run proc from the cursor on (it stays for good, its clock starting from 0), and move the cursor seconds forward. The seconds are how long before the next step starts, not how long proc lasts.

### `thenfor`

`seconds proc  thenfor  -`

In a script: run proc for seconds starting at the cursor (it is gone afterwards; inside, `span` is seconds), and move the cursor past it.

### `wait`

`seconds  wait  -`

In a script: move the cursor forward, leaving a pause. Negative overlaps the next step with the last.

### `cursor`

`-  cursor  t`

The script cursor: the time, in the current scope, at which the next `then` will start.

### `tempo`

`bpm  tempo  -`

Set the tempo in beats a minute (in setup; 120 if never set), for films cut to a pulse.

### `counts`

`n  counts  seconds`

n beats at the tempo, in seconds: write times and storyboard lengths in beats (`[ /intro 8 counts /drop 16 counts ] storyboard`, `2 counts /pop { } enters`).

### `countnow`

`-  countnow  n`

The beat the film is on, as a number that grows by 1 every beat (movie clock): its whole part is the beat's index, `countnow 4 mod` the place in a bar of four.

### `kick`

`-  kick  u`

1 on every beat, falling quickly to 0 before the next: multiply a size, a brightness or a blur by it and the picture keeps time.

### `cascade`

`array gap proc  cascade  -`

Run proc once per element with `item i` on the stack, each element's clock gap seconds behind the one before: a row of things arriving one after another, written as if each were alone.

### `cascadespan`

`array gap  cascadespan  seconds`

How long after a cascade starts its last item starts: (n - 1) x gap. Use it for the time of the step that follows a cascade, or of item k (`k gap mul`), instead of working the number out and typing it: `buckets gap cascadespan 0.6 add wait`.

