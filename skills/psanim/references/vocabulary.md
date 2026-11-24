# psanim vocabulary

Generated from `crates/psanim/src/api.rs` by `scripts/gen_reference.sh`. Do not edit by hand.

Stack effects read left to right, topmost operand last, as in the PostScript Language Reference: `a b /name  tween  u` takes three operands and leaves one.

Easings (for `ease`, `tween`, `keyframes`): linear smooth smoother quad-in quad-out quad-inout cubic-in cubic-out cubic-inout quart-in quart-out quart-inout quint-in quint-out quint-inout sine-in sine-out sine-inout expo-in expo-out expo-inout circ-in circ-out circ-inout back-in back-out back-inout elastic-in elastic-out elastic-inout bounce-in bounce-out bounce-inout spring hold

Transitions (for `reel`): cut dissolve dip wipe-left wipe-right wipe-up wipe-down iris push-left push-right zoom

## Declaring the movie

### `animation`

`dict  animation  -`

Declare the movie. The first statement of a program. Keys, all optional: /Size [w h] (canvas units, default [1280 720]), /Duration seconds (5), /FPS (30), /Seed (1), /Background [r g b] or /transparent (black), /Title, /Description, /Loop bool, /Scene /name (scene), /Step /name (step), /StepRate ticks per second (FPS), /Persist bool, /MotionBlur samples, /Shutter 0..1 (0.5).

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

### `attime`

`t proc  attime  -`

Run proc with now frozen at t: a still of a moving thing.

### `timewindow`

`t dur proc  timewindow  -`

Run proc with now set to t in a window dur seconds long.

### `sequence`

`[ dur proc dur proc ... ]  sequence  -`

Play procs one after another, each for its duration, each with its own clock from 0. A proc may be given by name: /intro.

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

### `smoothstep`

`e0 e1 x  smoothstep  u`

0 below e0, 1 above e1, a smooth S-curve between.

### `remap`

`x a b c d  remap  y`

Map x from the range a..b to the range c..d, linearly, unclamped.

### `wiggle`

`key freq amp  wiggle  v`

Smooth random motion: a value wandering in -amp..amp about freq times a second. Different keys give independent wanderings; the same key always gives the same one.

### `hash01`

`key  hash01  u`

A repeatable random number in 0..1 determined by key (a number or an array of numbers) and the movie seed. Stateless: the same key gives the same value on every frame, which keeps per-particle properties from flickering.

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

n points evenly spaced along the current path, as [x0 y0 x1 y1 ...]. Does not change the path.

### `morphpath`

`arrayA arrayB u  morphpath  -`

Replace the current path by the closed polygon u of the way from A to B, two arrays from resamplepath with the same n.

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

Add a soft halo around everything on the current layer: a blurred copy added back, strength 0..2.

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

Draw proc with a halo of light.

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

Fill the whole frame with a colour, whatever the transformation and clip are.

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

Run proc once per character with `(c) i` on the stack (the character as a one-character string, and its index) and the current point at that character's origin. The font must be set. proc consumes both operands; `{ pop show }` reproduces show.

## Reels: clips and transitions

### `reel`

`[ [dur proc] [dur /transition] [dur proc] ... ]  reel  -`

Play clips in order with transitions between them. A transition overlaps the end of the clip before it and the start of the clip after it. Transitions: /cut /dissolve /dip /wipe-left /wipe-right /wipe-up /wipe-down /iris /push-left /push-right /zoom.

### `reelduration`

`[ ... ]  reelduration  seconds`

The total length of a reel array: use it for /Duration.

### `transitions`

`-  transitions  dict`

The dictionary of transitions. Add `procA procB u` procedures under names starting with t- to define your own.

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

