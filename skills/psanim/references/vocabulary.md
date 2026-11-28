# psanim vocabulary

Generated from `crates/psanim/src/api.rs` by `scripts/gen_reference.sh`. Do not edit by hand.

Stack effects read left to right, topmost operand last, as in the PostScript Language Reference: `a b /name  tween  u` takes three operands and leaves one.

Easings (for `ease`, `tween`, `keyframes`): linear smooth smoother quad-in quad-out quad-inout cubic-in cubic-out cubic-inout quart-in quart-out quart-inout quint-in quint-out quint-inout sine-in sine-out sine-inout expo-in expo-out expo-inout circ-in circ-out circ-inout back-in back-out back-inout elastic-in elastic-out elastic-inout bounce-in bounce-out bounce-inout spring hold

Transitions (for `reel` and `storyboard`): cut dissolve dip wipe-left wipe-right wipe-up wipe-down iris push-left push-right zoom

## Declaring the movie

### `animation`

`dict  animation  -`

Declare the movie. Put it before anything that takes time or draws (definitions it needs, such as a reel array for `reelduration`, may come first). Keys, all optional: /Size [w h] (canvas units, default [1280 720]), /Duration seconds (5), /FPS (30), /Seed (1), /Background [r g b] or /transparent (black), /Title, /Description, /Loop bool, /Cuts [t ...] (seconds where the movie cuts on purpose: `psanim check` does not report an abrupt change there), /Scene /name (scene), /Step /name (step), /StepRate ticks per second (FPS), /Persist bool, /MotionBlur samples, /Shutter 0..1 (0.5).

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

### `steprate`

`-  steprate  num`

The rate the movie steps at: for a persistent movie its declared FPS, whatever `--fps` it is rendered or shown at (its canvas is painted one step a declared frame), for a simulation its /StepRate (in setup too, once the step procedure is defined), otherwise the frame rate. The same in setup, the scene and the overlay. `fps` is the rate frames are shown at.

### `stepnow`

`-  stepnow  int`

The last step run at this moment: in a persistent movie's scene the step being painted (its `frame`), in its overlay the last step the kept canvas holds, in a simulation's scene the last tick applied (-1 before the first), in a pure movie the frame. In the overlay, under any `--fps`, `t stepat stepnow le` says whether the scene has painted the step that reaches t. Inside a component, the step its own clock has reached, at the movie's step rate, so a delayed, sped-up or looping instance counts its own.

### `stepat`

`t  stepat  int`

The step on which movie time t is reached: the first whose time is at or after t (a t within a millionth of a step after a step's time counts as that step's), 0 for t at or before 0. `t stepat stepnow eq` in a persistent scene is true on exactly one step for any t: the way to do something once when a time is crossed, which `now dt sub` cannot say exactly in floating point.

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

k whole turns spread over the current window: k * 360 * now / (the window's length). The angle for anything that must come home when a loop closes; k must be a whole number (or a whole number divided by the part's rotational symmetry). Under `delayed` (and so `staggered`, `cascade`, `echoed`) the length is the undelayed window's, so it is the undelayed angle from that much earlier and a late copy still closes the loop.

## Time of day, for clocks that show it

### `parseclock`

`(HH:MM:SS) | seconds  parseclock  seconds`

A time of day as seconds since midnight. Text is (H:MM), (H:MM:SS) or (H:MM:SS.fff), the form `--param Start=12:34:56` gives; a number is passed through, so a parameter can be either. A malformed time is an error that says what is accepted.

### `daytime`

`start /signal|null  daytime  seconds`

The time of day to show, in seconds since midnight (fractions kept, wrapped into one day). A movie is a pure function of its own time and never reads the wall clock, so the host supplies the time one of two ways. With null: `start` (seconds, or text as for `parseclock`) plus the movie time, so a render shows the clock running from the start time. With a signal name (a name, or text from `--param`): that signal's value, which a live page sets to the viewer's local seconds since midnight before each frame (`engine.setSignal("clock", s)`); `start` is then ignored.

### `clockparts`

`seconds  clockparts  h m s frac`

Split a time of day, wrapped into one day: hours 0..23, minutes and seconds 0..59 (integers) and the fraction of the second, 0 up to 1. 45296.25 gives 12 34 56 0.25.

### `fmtday`

`seconds  fmtday  string`

A time of day on the 24 hour clock as text: (07:05:09). Build other forms from `clockparts` and `zeropad`.

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

Run proc with its clock d seconds behind: whatever it does happens d seconds later. A pure shift: `span` and `progress` see the window shortened by d, but `turns` and `loopnoise` are the undelayed values from d seconds earlier.

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

Smooth random motion in -1..1 (key is a number or an array of numbers) that returns exactly to its start after the current window's length: noise sampled around a circle. A larger radius wanders more. For organic drift in a seamless loop, where wiggle would not close. Under `delayed` the length is the undelayed window's, so a late copy still closes.

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

### `fadeticks`

`lo hi n  fadeticks  [ v alpha ... ]`

Ticks for a range that is moving: about n round numbers over lo..hi from the two nice steps either side of the ideal one, each with an opacity. Ticks both steps share stay at 1; the rest cross-fade as the range grows or shrinks, so labels never jump. `plotaxes` uses it with /Fade true.

### `seriespoints`

`[ v ... ] box [lo hi]  seriespoints  [ x0 y0 x1 y1 ... ]`

A series spread evenly across a box, lo at its bottom and hi at its top, clamped to the box: points for `polypath`. A reversed range [hi lo] turns it upside down. The fast way to draw a long series.

## Particles: emitters whose particles are functions of time

### `particles`

`spec proc  particles  -`

Draw proc once for every particle alive now, with the origin moved to the particle (inside gsave) and the caller's stack; `particleage`, `particleu`, `particlevel`, `particlekey` and the rest read the particle. spec is an emitter dictionary or an array of them, all keys optional: /At [x y] or keyframes [t [x y] /ease t [x y] ...] (a moving emitter; the view's centre), /Area [w h] (scatter births over a box centred on /At), /Rate per second with /From and /To (continuous), /Bursts [t ...] each emitting /Count (50) at once, /Life seconds or [lo hi] (2), /Speed or [lo hi] (100), /Angle degrees (90, up) and /Spread degrees (360), /Shape /random /even (evenly round) /sphere (a 3-D burst seen from the side), /Gravity (downward units/s/s, or [gx gy]), /Drag (per second: speeds settle to (gravity+wind)/drag), /Wind [ax ay] (a steady force), /Gust [ax ay] with /GustFreq (a force that wanders over time, the same for every particle), /Sway [dx dy] with /SwayFreq (each particle's own wandering offset: snow, leaves, confetti), /Inherit (share of a moving emitter's velocity), /Period seconds (continuous emission and noise repeat: give the loop's length in a /Loop movie; with /Life no longer than it every particle has its own key; longer, a particle is the one born a period earlier, same key and flight), /Seed (this emitter's randomness), /Draw proc (draws this emitter's particles instead of proc). A particle's state is computed from its birth and the time alone, so frames render in any order.

### `particleage`

`-  particleage  seconds`

Inside a particles procedure: seconds since this particle was born.

### `particleu`

`-  particleu  u`

Inside a particles procedure: how far through its life the particle is, 0 at birth to 1 at death. `1 particleu sub setalpha` fades it out.

### `particlelife`

`-  particlelife  seconds`

Inside a particles procedure: how long this particle lives in all.

### `particlepos`

`-  particlepos  x y`

Inside a particles procedure: where the particle is, in the coordinates `particles` was called in (the origin is already there).

### `particlevel`

`-  particlevel  vx vy`

Inside a particles procedure: the particle's velocity, units a second. `particlevel exch atan rotate` turns to its heading.

### `particlekey`

`-  particlekey  n`

Inside a particles procedure: a whole number standing for this particle, the same on every frame: `[particlekey 1] hash01` is its own constant (a size, a colour, a spin). With /Period, keys repeat every period, which is what closes the loop: if /Life is longer than /Period, two live particles can share a key (one is the other, a period older) and fly the same path a period apart.

### `particleemitter`

`-  particleemitter  i`

Inside a particles procedure: the index of the emitter it came from, in the spec array (0 for a single dictionary).

### `particleburst`

`-  particleburst  i`

Inside a particles procedure: the index in /Bursts of the burst it came from, or -1 for continuous emission.

### `particletrail`

`seconds n  particletrail  -`

Inside a particles procedure: replace the current path with the particle's own track over the last seconds (from its birth at most), n segments, oldest first, ending at the origin: `0.2 6 particletrail stroke` is a streak. The track is where the particle was on the page, drawn in the space you are in: it stays right after a `rotate` or `scale` inside the procedure.

## Cellular automata: grids of cells that run a rule

### `cellgrid`

`cols rows opts  cellgrid  cells`

A new grid for `cellsteps`: a string of one byte per cell, row after row from the top. opts is a pattern or a dictionary: /Pattern, /Density (0.3), /Seed (the movie's). A pattern is a name (/glider /blinker /block /beacon /lwss /rpentomino /acorn /diehard /gun (Gosper's glider gun) /pulsar /single /empty, or /random: each cell alive with chance /Density, from /Seed), a picture (an array of strings, `O` or `#` alive, as LifeWiki's plaintext), RLE text ((bo$2bo$3o!)), or a list of placements [ [pattern x y] ... ] with x y the cell its centre goes on, from the top left. One pattern alone is centred. A byte is 128 and up for a live cell (128 + its age), 1 to 127 for a dead one that once lived (127 the generation it died), 0 for never alive.

### `cellsteps`

`cells cols opts n  cellsteps  cells'`

Generation n of a grid `cols` wide: the rule applied n times, natively. opts is the rule or a dictionary of /Rule and /Edge: /dead (the default: dead cells beyond the edge, where a glider becomes a block), /wrap (a torus), or /open (an open plane as near as a grid gets: a lone glider that nears the edge is taken off, as it would fly away for ever; exact while nothing else reaches the edge). A rule is a name (/life /highlife /seeds /daynight /maze /replicator /lifewithoutdeath, or /brain for Brian's Brain) or births and survivals as text ((B3/S23)). A pure function of its operands: a film asks for `now rate mul` and gets that generation however it seeks. States already computed are remembered, so the next frame's generation costs one step, not all of them. Capped at 2048 x 2048 cells and about a billion cell updates a frame, and charged to the budgets.

### `cellrows`

`row rule rows  cellrows  cells`

The history of an elementary automaton: `rows` rows, the first being `row` (a grid of one row, as `cols 1 /single cellgrid` makes), each next cell bit 4l+2c+r of the rule (0 to 255: 30, 90, 110) from the three cells above it. rule may be a dictionary of /Rule and /Edge (/dead or /wrap). Live cells 255, dead 0, `cols` wide: draw it with `cellcolors` and `colorimage`.

### `cellcolors`

`cells palette frac  cellcolors  rgb`

Three bytes a cell, for `cols rows 8 [cols 0 0 rows neg 0 rows] rgb false 3 colorimage` in a unit square (row 0 at the top, square cells). palette is a dictionary of colours [r g b] or grey levels: /Alive, /Born and /Mature (a newborn cell is /Born and takes /Mature generations to become /Alive), /Trail and /Fade (a cell that dies starts at /Trail and fades over /Fade generations to /Ghost), /Ghost (where a cell once lived: /Ground unless given), /Ground. frac (0 to 1, how far the clock is to the next generation) keeps a dying cell fading smoothly between generations.

### `cellbounds`

`cells cols share  cellbounds  [x0 y0 x1 y1] | null`

The box, in cells from the top left (inclusive), of the cells that are or once were alive, counting only rows and columns that hold more than `share` of what the fullest one holds (0.2 leaves out the thin tracks of gliders that flew away; 0 counts every one); null if none ever lived. It grows as a pattern spreads, so a camera framed on it pulls back smoothly.

### `cellmask`

`rgb cols ground  cellmask  mask`

A stencil for the colours `cellcolors` made: one bit a cell, rows padded to whole bytes, set where the colour is not the ground [r g b]. As the /MaskDict of an `ImageType 3` image (/BitsPerComponent 1, /Decode [1 0]) it paints the cells and leaves the ground transparent.

### `cellcount`

`cells  cellcount  n`

How many cells of a grid are alive.

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

### `alignpath`

`arrayA arrayB  alignpath  arrayB2`

B's points (from resamplepath, a closed outline) started from the place that lies closest to A's, as a new array; if B runs the other way round it is also reversed, since a morph between opposite senses collapses on the way. Morph A into the result with `morphpath` and every point travels the least, so a shape melts into the next rather than twisting. Do it once in setup for each pair you will morph, not per frame (it costs n squared).

### `projectpath`

`plane  projectpath  -`

Replace the current path by its projection onto a plane in perspective (the plane dictionary as onplane takes it), in the current user space. Curves are cut into chords about a pixel long on the screen; straight edges stay straight, as they do in perspective; open subpaths stay open, and the part of the path behind the eye is cut away.

### `planepoint`

`x y plane  planepoint  x' y' opacity`

Where the point (x, y) of a plane (as onplane takes it) appears on the screen, and the opacity the plane's /Fade leaves there. For placing a flat thing (a label, a sprite) at a place on the plane. An error if the point is behind the eye.

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

Start drawing on a separate transparent layer. The graphics state is unchanged. Not inside a PSCat begingroup (groupnesting).

### `endlayer`

`alpha  endlayer  -`

Finish the layer and composite it over what is beneath at opacity alpha.

### `endlayerblend`

`alpha /mode  endlayerblend  -`

Finish the layer and composite it with a blend mode: /normal /add /screen /multiply.

### `blurlayer`

`radius  blurlayer  -`

Blur everything drawn on the current layer so far (Gaussian, radius in canvas units). Only inside the current clip: what lies outside it is left as it was.

### `glowlayer`

`radius strength  glowlayer  -`

Add a soft halo around everything on the current layer: a blurred copy added back, strength 0..2. The halo's light is the shape's own, spread out: a thin line needs a larger radius and a strength above 1 to show. Only inside the current clip, like blurlayer.

### `faded`

`alpha proc  faded  -`

Draw proc as one group at opacity alpha. Unlike setalpha, overlapping shapes inside do not show through each other. Inside a PSCat begingroup it runs as a group of its own; giving the outer group an /Alpha is cheaper.

### `blended`

`alpha /mode proc  blended  -`

Draw proc as a group composited with a blend mode. Inside a PSCat begingroup, /normal and /multiply run as a group of their own; /add and /screen stop with groupnesting.

### `blurred`

`radius proc  blurred  -`

Draw proc out of focus. The blur stays inside the clip in force when it begins (a within box, a rectclip, any clip path). Not inside a PSCat begingroup (groupnesting): put the group inside proc.

### `glowing`

`radius strength proc  glowing  -`

Draw proc with a halo of light. Try 8 1 for solid shapes, 24 1.6 for thin strokes. The halo stays inside the clip in force when it begins. Not inside a PSCat begingroup (groupnesting): put the group inside proc.

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

### `onplane`

`proc plane  onplane  -`

Run proc with its painting laid on a plane in 3D and seen in perspective: fill, eofill, stroke, rectfill, rectstroke, clip, eoclip, rectclip and the show family (show, ashow, widthshow, awidthshow) are projected when proc calls them. Procedures bind-ed before (the prelude's, the pk libraries') hold the built-ins and paint flat. The plane's coordinates are the user space onplane was called in; proc's own translate, scale and rotate move things on the plane. plane is a dictionary: /Tilt degrees the plane is tipped back about the x axis through the origin (0 faces you, 90 is a floor seen edge-on; +y goes away), /Turn degrees then turned about the vertical axis, /Depth how far the origin is pushed behind the screen, /Eye [x y] the point the eye is straight in front of (default [0 0]), /Distance how far the eye is from the screen (default the view's width), /Fade [z0 z1] the depth behind the screen over which ink fades out (each painted path takes one opacity, from the depth of its middle). Strokes go through strokepath, so line widths lie on the plane and shrink with distance. Ink behind the eye is cut away. image, shfill, kshow, xshow and glyphshow are not projected.

## Text

### `typed`

`string u  typed  string`

The first u (0..1) of the string, by the characters the current font draws: a typewriter reveal. In a font that reads UTF-8 (any `face` font) a character is a whole grapheme, so a letter and its accent, a flag or an emoji sequence are never cut; in a font that reads bytes it is a byte, as PostScript has it.

### `typingplan`

`(text) seed opts  typingplan  plan`

Plan how a person would type a string, from a seed: a key every ~0.1 s with jitter, bursts and slow words, longer gaps after a space, a thinking pause now and then, and an occasional slip to a neighbouring key that is noticed a character or two later, backspaced and retyped. opts: << /Speed 1 (keys-rate multiplier) /Typos 0.02 (chance of a slip per letter) /Slips [4] (letters that always slip) /Pauses 1 /Think 0 (seconds before the first key) >>. Make the plan in setup; `plan t typedat` then gives the text typed by time t, a pure function of t. Same text and seed always type the same way.

### `sessionplan`

`[ item ... ] seed opts  sessionplan  plan`

Plan a terminal session, from a seed. Items: [/type (cmd)] or [/type (cmd) << /Speed 1.4 /Slips [3] /Think 1 >>] (prompt, human typing, Enter); [/out (text\nmore)] (lines appear at once); [/lines (text) gap] (one line every ~gap s, a little uneven); [/progress (frame1\nframe2 ...) gap] (a line redrawn in place); [/wait s]; [/clear]; [/prompt (> )]; [/mark /name] (note the time: `plan /name planmark`). opts: << /Prompt ($ ) /Speed /Typos /Think 0.6 /Latency 0.18 (Enter to output) /Gap 0.07 /Start 0 /FinalPrompt true /Tail 0 >>. Output follows its command after the latency. The plan is data: `plan t rows cols screenat` is the grid at any time.

### `planlength`

`plan  planlength  seconds`

The time of the last thing a typing or session plan does (plus the session's /Tail): how long to give it.

### `planmark`

`plan /name  planmark  seconds`

The time a session plan reached `[/mark /name]`, or -1 if it has no such mark: to start something else (a progress bar, a flash) when the session gets there.

### `typedat`

`plan t  typedat  (string)`

The line under the cursor at time t: for a `typingplan`, what has been typed so far, backspaces and all. `(text) 7 << >> typingplan` in setup; `plan now typedat show` in the scene.

### `screenat`

`plan t rows cols  screenat  [ (row) ... ] col last`

What a character grid shows at time t: its rows (at most `rows`, top to bottom; long lines wrap at `cols`; the screen scrolls when full), the cursor's column in the last row, and the time of the latest keystroke or line before t (-1 if none), for a cursor that stays solid while typing and blinks at rest. Pure in t, so the screen a moment ago (`t 0.1 sub screenat`) is phosphor persistence.

### `showcenter`

`string  showcenter  -`

Show the string centred horizontally on the current point.

### `charcount`

`string  charcount  n`

How many characters the string has read as UTF-8 text, as a reader counts them: its grapheme clusters, so an accented letter, a flag, a skin-toned or ZWJ emoji is one; a byte that is not part of a well-formed sequence counts as one; ASCII is its length. It looks at the text only, not the current font: the words that draw (`typed`, `eachglyph`, `eachspaced`, `eachonpath`, `spacedwidth`, `kinetic`'s /glyphs) step the way the current font reads, which for a font chosen with findfont is by byte.

### `unicodefont`

`font  unicodefont  font'`

A copy of the font that reads its strings as UTF-8: accents, Greek, Cyrillic, signs and colour emoji, with the characters its face lacks taken from PSCat's fallback faces. `face` does this already; this is for a font chosen with findfont: `/Helvetica findfont unicodefont 24 scalefont setfont`. Symbol, ZapfDingbats and Type 3 fonts come back unchanged. Korean, Japanese, Chinese, Thai, Arabic, the Indic scripts and others have no glyphs yet: they draw an empty box, and `check` names each.

### `eachglyph`

`string proc  eachglyph  -`

Run proc once per character with `(c) i` on the stack (the character as a string, and its index; a character is a whole grapheme in a font that reads UTF-8, a byte in one that reads bytes) and the current point at that character's origin. The font must be set. proc consumes both operands; `{ pop show }` reproduces show. The current point is set afresh before each call, so proc may move it, build paths and paint; graphics state it changes (colour, CTM) stays changed, so wrap transforms in gsave/grestore.

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

### `fmtshort`

`num decimals  fmtshort  string`

The number in a dashboard's short form: 950, 12.4k, 3.2M, 1.1B, 4.0T, with that many decimal places (below a thousand too, so a tween keeps its width). A value that rounds up takes the next unit: 999.96 1 fmtshort is (1.0k).

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

Play clips in order with transitions between them. A transition overlaps the end of the clip before it and the start of the clip after it. Transitions: /cut /dissolve /dip /wipe-left /wipe-right /wipe-up /wipe-down /iris /push-left /push-right /zoom. Under /dissolve, the wipes and /iris the incoming clip is drawn over the outgoing one, which is not faded: each clip must paint its own ground (`ground`, `backdrop`), or what the incoming clip does not cover stays at full strength through the transition and vanishes at its end. /cut, /dip, /push-left, /push-right and /zoom also work with clips that draw only their own subject over whatever the scene drew before the reel.

### `reelduration`

`[ ... ]  reelduration  seconds`

The total length of a reel array: the clips' durations added up, minus the transitions' (a transition overlaps the clips on both sides, so a clip's own duration includes the transitions into and out of it). Use it for /Duration: define the reel array first, then `<< /Duration film reelduration ... >> animation`. It runs before the clips and your own transitions exist, so it takes the built-in transitions and any name starting `t-` for transitions and every other name for a clip; once the program has loaded, a measurement that a custom transition (any other name) would have changed is an error naming it, not a longer film.

### `transitions`

`-  transitions  dict`

The dictionary of transitions. To define your own, put a procedure there: `transitions /t-shutter { ... } put`, then use `[dur /t-shutter]` in a reel (or between a storyboard's beats, defined before the storyboard). Name it with a `t-` prefix, or define it before the reel is measured with reelduration: otherwise reelduration counts the entry as a clip, and loading the movie fails, naming it. It is called with `procA procB u`: draw the frame that is u (0 to 1, linear) of the way from A to B. A and B are ready-made procedures that draw each clip with its own clock still running; call them inside gsave/grestore with whatever clip, transform or layer the transition needs. Between storyboard beats each beat's `inbeat` draws its own side (A with an empty B, then an empty A with B), so a transition used there must draw A's part and then B's part, neither depending on the other, as every built-in one does.

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

### `sincelast`

`[t ...]  sincelast  seconds`

Seconds since the most recent of the times that has passed, in the current scope; a million if none has. An array of times is an event track you own: `bursts sincelast 6 mul neg exp 0.1 mul 1 add` is a scale that kicks on each one and settles.

### `eventcount`

`/name  eventcount  int`

How many events on track name have happened so far.

### `signalago`

`/name dt  signalago  num`

The host's signal name as it was dt seconds ago, interpolated between readings: the recent past of a recording, or of a live signal its host records (`Movie::record_signal`). `/cpu 0.5 signalago` reads half a second late, so a value updated once a second glides instead of stepping.

### `signalseries`

`/name span n  signalseries  [ v ... ]`

n readings of signal name spread over the last span seconds, oldest first, the last one now: the history for a sparkline. The readings move with now; for a window that must not shimmer use `signalbetween` on a grid.

### `signalbetween`

`/name t0 t1 n  signalbetween  [ v ... ]`

n readings of signal name evenly spaced from movie time t0 to t1, both included: a fixed stretch of a recording (a day replayed behind a cursor), or a window aligned to a grid so samples do not slide.

### `signaltime`

`-  signaltime  t`

The movie time signals are read at: the movie's own clock, also inside a component (whose `globaltime` starts at its /At).

### `hassignal`

`/name  hassignal  bool`

Whether the host gave a signal of that name: to draw a "no data" state instead of a row of zeros.

## Viewports and libraries

### `within`

`box proc  within  -`

Run proc inside the box [x y w h]: the origin moves to the box's corner, drawing is clipped to it, and `width` and `height` report the box's size. The spatial twin of `during`: a scene written against width and height can be placed in any box (a panel, one half of a comparison, a picture in a picture).

### `fitted`

`box w h proc  fitted  -`

Run proc, which draws for a canvas w by h, scaled to fit inside the box and centred there, clipped. Inside, `width` and `height` are w and h. For placing a finished scene in a panel whatever its aspect.

### `use`

`/library  use  -`

Load one of psanim's libraries, once: /pkstage /pkmotion /pktype /pkdiagram /pkcode /pkchart /pkink /pkfigure /pkpixel. Call it in setup, after `animation`. A library loads the ones it needs. `psanim api --libs` lists them; `psanim api --lib pkmotion` lists one's words.

## Components: movies placed inside movies

### `component`

`/name dict-or-(file.ps)-or-/entry  component  -`

Define a component and set up its first instance under the same name. A name is an entry of the catalog of ready-made elements (`psanim catalog` lists them: /busy /spinner component). A component is written like a movie: from a file, the file's own `animation` declaration, setup and `scene`; inline, a dictionary with the declaration's keys (/Size, /Duration, /Loop, /Seed, /Background) and /Setup { } and /Scene { }. Its setup runs once, in a dictionary of its own, so its names do not reach the movie or another instance; inside it `width`, `height`, `duration`, `globaltime`, `movieseed` and `param` are the component's. A file name is read from the movie's own directory. Call it in setup.

### `instance`

`/new /name << /Seed s /Params << ... >> >>  instance  -`

Another instance of a component or a catalog entry, set up again: /Seed for its randomness, /Params for its `param` lookups (a name not given gets the default the component wrote). Inside a component, `param` reads only these, never the host's --param values. Call it in setup.

### `place`

`/name opts  place  -`

Draw an instance into the scene. opts (all optional): /At the movie time it starts (0), /Speed (1), /Box [x y w h] its declared size is fitted into (the whole view), /Before /hide or /hold (its first frame), /After /hide /hold (its last frame) or /loop (default /loop if it declared /Loop true, else /hide). Inside, `now` and `globaltime` run from 0 at /At. It paints its /Background only if it declares one. Blur and glow radii inside it are in its own units, so they scale with its box. Place the same instance as often as you like: each placement is drawn from its own time.

### `componentduration`

`/name  componentduration  seconds`

The /Duration an instance declared, for timing the movie around it.

### `componentsize`

`/name  componentsize  w h`

The /Size an instance declared, for choosing a box with its aspect.

## Playlists: scenes that cycle, for signage

### `playlist`

`/name entries-or-(file) opts  playlist  -`

Define a playlist, in setup (before `animation` too, to use `playlistduration` for /Duration). It is a component: `/name << /Box [x y w h] >> place` draws it, and it loops for ever, the last scene handing over to the first. entries is an array of [hold scene] and [seconds /transition], or the name of a file holding that array (read from the movie's directory, like a component file; it is data: write the array and nothing else). A scene is a catalog entry or component (/spinner), a component file ((events.ps)), a component dictionary, or a procedure (a name, or { }) that draws for `width` by `height`; [hold scene << options >>] adds /Params and /Seed (as for `instance`), /Fill true (the component is laid out for the whole box instead of its declared /Size), /Background (this scene's ground), /Days and /From /To (see below) and /Orientation /landscape or /portrait. hold is seconds fully on screen; the transition after a scene (any in `transitions`; [0 /cut] or none is a plain cut; none written gets /Transition) is extra, so a round takes the sum of both for every scene. Every scene is drawn over its own ground, full bleed, so transitions work as in a reel. opts, all optional: /Size [w h] (the playlist's canvas; default the movie's), /Safe 0.05 (the fraction of each edge kept clear, for a TV that crops; scenes are placed inside it, grounds and transitions are not), /Background [r g b] or a procedure (default a dark blue-black), /Transition [1 /dissolve] (the default), /Guides true (draw the safe area), /Seed. Schedule: a scene on the list only /Days (/mon .. /sun, /weekdays, /weekend, /daily, 0 Monday to 6, or an array) and between /From and /To (hours 9.5, or (9:30), crossing midnight allowed). The set is chosen when a round starts, so a scene is never cut off, and for a scheduled playlist the loop is one week (/Duration to change). The playlist knows the time of day from /Day (default /mon) and /Time (hours or (H:MM), default 0): the time at its own time 0, which the movie takes from `param`, never the wall clock. /Clock /signal makes `weektime` inside the scenes read the host's `weektime` signal instead of /Day and /Time plus the clock, for a live display whose page sets it every frame (the schedule still follows /Day and /Time).

### `playlistduration`

`/name  playlistduration  seconds`

The length of one round of a playlist at its start time: the holds and transitions of the scenes on the list added up. For a playlist with no schedule that is the loop, and `<< /Duration /name playlistduration /Loop true >> animation` makes a movie that loops without a seam. For a scheduled one it is the round the movie opens with (0 if nothing is scheduled then); the set can change later in a longer run. A scheduled playlist's own loop is a week: `componentduration` says so.

### `weektime`

`-  weektime  seconds`

The time the schedule is at, as seconds since Monday 00:00. Inside a scene of a playlist: its /Day and /Time plus its own clock, so a clock tile can read the time without owning one (`weektime 86400 fmod` is the time of day). Outside a playlist: the host's `weektime` signal, 0 if there is none.

## Storyboards, scripts and cascades

### `storyboard`

`[ /name seconds [seconds /transition] /name seconds ... ]  storyboard  -`

Name the stretches of the film, in order. Call it before `animation` and use `storylength` for /Duration; then place things by beat name instead of by second, so retiming the film is editing one list. Between two beats the picture cuts, unless a transition is written between them: `[ /intro 4 [0.6 /dissolve] /main 8 ]`, any name in `transitions`, defined before the storyboard. As in a reel it overlaps the two beats: the next beat starts that many seconds before this one ends, so the film is shorter by it, and each beat keeps its full length and its clock. Under /dissolve, the wipes and /iris each beat on either side must paint its own ground (`ground` or `backdrop` first), as a reel's clips must. The `inbeat` calls come in the storyboard's order: drawing the outgoing beat after the incoming one in a transition is an error. Draw what the beats share after all of them. A beat lasts at least the transitions into and out of it together.

### `storylength`

`-  storylength  seconds`

The total length of the storyboard: the beats added up, less the transitions between them.

### `beat`

`/name  beat  t`

The movie time at which the named beat starts. Add to it for a moment inside the beat: `/build beat 0.5 add`.

### `beatspan`

`/name  beatspan  a b`

The start and end of the named beat, on the movie's clock: operands for `during` at the top level of a scene. With a transition the spans of two beats overlap by its length.

### `inbeat`

`/name proc  inbeat  -`

Run proc during the named beat, with the clock starting from 0 at the beat's start and `span` its length, whatever time scopes are open. In a transition into or out of the beat, proc is that side of it: the incoming beat drawn over the outgoing one, both on their own clocks.

### `frombeat`

`/name proc  frombeat  -`

Run proc from the start of the named beat to the end of the film, with the clock starting from 0 there: for something a beat introduces that then stays. It comes in with the transition into the beat, if there is one, and no later transition takes it away.

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

