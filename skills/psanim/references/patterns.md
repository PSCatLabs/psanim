# psanim patterns

Recipes that work. Every block here is run by the test suite: a block that
starts with `%!PS` is a whole movie; any other block is the body of a
`scene` on a 640x360 canvas, 4 seconds long.

## Contents

- [Move, scale, rotate, fade](#move-scale-rotate-fade)
- [Enter, hold, leave](#enter-hold-leave)
- [A row of things, one after another](#a-row-of-things-one-after-another)
- [Draw a line on; write text on](#draw-a-line-on-write-text-on)
- [Letters that arrive one by one](#letters-that-arrive-one-by-one)
- [Spaced capitals, and type along a curve](#spaced-capitals-and-type-along-a-curve)
- [A value the whole film shares](#a-value-the-whole-film-shares)
- [Follow a path](#follow-a-path)
- [Morph one shape into another](#morph-one-shape-into-another)
- [A camera](#a-camera)
- [Perspective: drawing on a plane](#perspective-drawing-on-a-plane)
- [Glow, blur, depth](#glow-blur-depth)
- [Particles without state](#particles-without-state)
- [A seamless loop](#a-seamless-loop)
- [Several scenes: sequence and reel](#several-scenes-sequence-and-reel)
- [Keyframes](#keyframes)
- [Procedures that take time as a parameter](#procedures-that-take-time-as-a-parameter)

## Move, scale, rotate, fade

One `tween` per property. Each is 0 before its start, 1 after its end.

```postscript
gsave
  0.3 1.5 /cubic-out tween 80 320 between  180 translate    % x: 80 -> 320
  0.3 1.8 /back-out  tween 0.2 1 between dup scale          % pops in, overshoots
  0 4 /linear tween 360 mul rotate                          % one full turn
  1 0.7 0.2 setrgbcolor -30 -30 60 60 rectfill
grestore
```

To fade something made of several shapes, fade the group, not the
shapes (`setalpha` on each would show the overlaps):

```postscript
0.5 1.5 /quad-out tween {
  0.3 0.6 1 setrgbcolor 250 120 140 140 rectfill
  1 1 1 setrgbcolor 320 190 60 0 360 arc fill
} faded
```

## Enter, hold, leave

Put a thing's whole life in one `during`. Inside, the clock starts at 0
and `span` is the window's length, so the exit is written relative to
the end.

```postscript
0.5 3.5 {
  % in over the first 0.6 s, out over the last 0.5 s
  0 0.6 /cubic-out tween   span 0.5 sub span /cubic-in tween 1 exch sub   mul
  {
    /Helvetica-Bold findfont 48 scalefont setfont 1 setgray
    320 170 moveto (hold) showcenter
  } faded
} during
```

## A row of things, one after another

`i gap { } staggered` delays item i by `i * gap` seconds. Write the item
as if it were alone.

```postscript
0 1 6 {
  /i exch def
  i 0.12 {
    gsave
      110 i 70 mul add  0 0.7 /back-out tween 120 180 between translate
      i 7 div 0.6 1 sethsbcolor
      0 0 0 0.5 /cubic-out tween 22 mul 0 360 arc fill
    grestore
  } staggered
} for
```

## Draw a line on; write text on

`trimpath` keeps a fraction of the current path. It works on anything,
including the outline of text.

```postscript
% a curve that draws itself
newpath 60 80 moveto 200 320 420 20 580 260 curveto
0  0.2 2 /cubic-inout tween  trimpath
0.4 0.9 0.8 setrgbcolor 4 setlinewidth 1 setlinecap stroke

% a word: the outline is drawn, then the fill fades up under it
/Times-Bold findfont 72 scalefont setfont
320 140 moveto (ink) dup stringwidth pop 2 div neg 0 rmoveto true charpath
gsave 2.2 3.2 /quad-out tween { 1 setgray fill } faded grestore
0  1 2.6 /cubic-inout tween  trimpath
1 0.8 0.3 setrgbcolor 1.5 setlinewidth stroke
```

`gsave ... grestore` around the fill keeps the path for the stroke.
`trimeach` grows every subpath at once instead of one after another.

## Letters that arrive one by one

`eachglyph` runs a procedure per character with `(c) i` on the stack and
the current point at the character's origin.

```postscript
/Helvetica-Bold findfont 56 scalefont setfont
150 160 moveto
(KINETIC) {
  /i exch def /c exch def
  i 0.07 {
    gsave
      currentpoint translate
      0  0 0.6 /back-out tween 60 0 between  translate     % rises 60 units into place
      0 0.4 /quad-out tween {
        i 7 div 0.5 1 sethsbcolor 0 0 moveto c show
      } faded
    grestore
  } staggered
} eachglyph
```

### Spaced capitals, and type along a curve

`eachspaced` is `eachglyph` with letterspacing; `spacedwidth` measures it
for centring. `eachonpath` sets type along the current path, each letter
in its own rotated frame.

```postscript
% tracked capitals, centred, arriving letter by letter
/AvantGarde-Book findfont 34 scalefont setfont
320 (AFTERGLOW) 10 spacedwidth 2 div sub 200 moveto
(AFTERGLOW) 10 {
  /i exch def /c exch def
  i 0.06 { 0 0.5 /quad-out tween { 0.95 0.92 0.85 setrgbcolor c show } faded } staggered
} eachspaced

% a line of italic riding an arc
/Palatino-Italic findfont 20 scalefont setfont 1 0.75 0.4 setrgbcolor
newpath 320 -120 280 120 60 arcn
(the last light of the longest day) 0.5 3 /sine-inout tween 0.02 0.12 between { pop show } eachonpath
newpath
```

## A value the whole film shares

`tween` and `keyframes` read the *local* clock. Inside a `during` that is
what you want for the thing being animated, and wrong for something the
whole film shares (a sun setting behind every card). Read shared values
on the movie's clock with `globally`:

```postscript
/sun-y { { [ 0 300  4 60 /sine-inout ] keyframes } globally } def
0.9 0.5 0.2 setrgbcolor 480 sun-y 30 0 360 arc fill           % at top level
1 3 {                                                        % inside a scope: same sun
  1 setgray 480 sun-y 44 add moveto
  /Helvetica findfont 14 scalefont setfont (sol) showcenter
} during
```

## Follow a path

`u pathpoint` gives a point and heading on the current path without
changing it.

```postscript
newpath 60 100 moveto 220 340 420 20 580 240 curveto
gsave 0.45 setgray 1.5 setlinewidth [3 5] 0 setdash stroke grestore
% (the gsave kept the path: grestore brought it back for pathpoint)
0.3 3.5 /sine-inout tween pathpoint      % x y angle
3 1 roll translate rotate
newpath 1 0.6 0.2 setrgbcolor
14 0 moveto -10 8 lineto -10 -8 lineto closepath fill
```

## Morph one shape into another

Resample both shapes to the same number of points, then `morphpath`.

```postscript
gsave 320 180 translate
  % a circle, as 120 points
  newpath 0 0 100 0 360 arc closepath 120 resamplepath
  % a five-pointed star, as 120 points (built under a rotating CTM)
  gsave
    newpath 0 120 moveto
    4 { 36 rotate 0 48 lineto 36 rotate 0 120 lineto } repeat
    36 rotate 0 48 lineto closepath
    120 resamplepath
  grestore
  0.5 3 /cubic-inout tween morphpath
  1 0.75 0.3 setrgbcolor fill
grestore
```

`morphpath` carries point *i* of one outline to point *i* of the other, so
where the two start decides whether the shape melts or twists. For a pair
you will morph, `alignpath` turns the second array to start where it moves
least (and walks it the same way round); do it once in setup. A shape with
a hole is two contours, which `morphpath` cannot match; draw it as one
outline instead: a thick pen stroke turned into an outline with
`strokepath` is a single contour even for a 0 or an 8 (the pen path of a
0 starts and ends at one point, so the outline is cut by a slit of zero
width and fills as a ring). The `/clock-morph` catalog entry does exactly
this for its ten digits.

Both arrays are plain `[x y x y ...]` in the user space that was current
when they were sampled (y up, like everything). `morphpath` and `polypath`
place points in the user space current when *they* run, so an array made
once in setup can be drawn many times under different `translate`s and
`rotate`s: that is the cheap way to draw eight copies of a gear.

## A camera

Draw a world larger than the frame and move the view.

```postscript
gsave
  [ 0 [200 150]  2 [900 500] /cubic-inout  4 [1500 300] /cubic-inout ] keyframes aload pop
  0 4 /sine-inout tween 1 1.6 between      % zoom
  0                                        % no roll
  camera
  0 1 59 { /i exch def                     % a field of dots, placed once and for all by hash01
    [i 1] hash01 0.5 0.9 sethsbcolor
    [i 2] hash01 1800 mul  [i 3] hash01 800 mul  8 [i 4] hash01 22 mul add 0 360 arc fill } for
grestore
```

## Perspective: drawing on a plane

The CTM is affine: it can slant and squash, but it cannot make the far
end of a thing smaller than the near end. `proc plane onplane` can: it
runs proc and lays what it paints (fills, strokes, `show`, clips)
on a plane in 3D, seen through a camera. The plane's coordinates are the
user space onplane was called in, so draw as you would flat and let
`translate` inside proc move things along the plane.

```postscript
% a floor hinged on the bottom edge, leaning back 65 degrees
320 0 translate
{
  0.3 0.8 1 setrgbcolor 2 setlinewidth
  -300 0 600 900 rectstroke                               % its edges
  -300 60 300 { 0 moveto 0 900 rlineto } for stroke       % lines running away
  0 now 60 mul translate                                  % slide along it
  1 0.85 0.4 setrgbcolor
  /Helvetica-Bold findfont 64 scalefont setfont
  0 1 6 { /i exch def -190 i 130 mul moveto (ON A PLANE) show } for
} << /Tilt 65 /Distance 700 /Fade [500 1600] >> onplane
```

The plane dictionary: `/Tilt` (degrees about the x axis through the
origin; 90 is a floor seen edge-on), `/Turn` (degrees about the vertical
axis, after the tilt), `/Depth` (push the whole plane back), `/Eye [x y]`
(the point the eye looks straight at, default the origin), `/Distance`
(eye to screen; smaller is a wider lens and stronger perspective,
default the view's width), `/Fade [z0 z1]` (depth behind the screen over
which ink fades out). With the eye level with the hinge, as here, the
plane runs up to a horizon at `Distance` times the cotangent of `Tilt`.

What to know:

- **Each painted path takes one opacity** from `/Fade`, from the depth of
  its middle. A line of text on a tilted plane is all at one depth, so
  paint a line at a time (several `charpath`s, one `fill`), not a whole
  paragraph as one path.
- **Strokes are outlined first** (`strokepath`), so a line's width lies on
  the plane and narrows with distance, as paint on a floor would.
- **Only what proc paints directly is projected.** Words that were
  `bind`-ed before onplane ran (the prelude's, the pk libraries', and
  yours if you `bind` them) hold the built-in painting operators and draw
  flat. `image`, `shfill`, `kshow`, `xshow` and `glyphshow` are not
  projected.
- **Behind the eye is cut away**, and nothing comes closer than a
  twentieth of `Distance`, so a plane can run under the camera.
- `x y plane planepoint` gives where a point of the plane lands (and its
  fade), to place a flat thing on it; `plane projectpath` projects the
  current path without painting.

For an opening crawl, text receding over stars, place the catalog's
`/crawl` entry (`psanim catalog crawl`) with your own words: see
[type.md](type.md#an-opening-crawl).

## Glow, blur, depth

```postscript
% light: draw bright things inside `glowing`
6 0.9 { 0.4 0.9 1 setrgbcolor 3 setlinewidth
        120 180 moveto 520 180 lineto stroke } glowing
% depth: far things blurred and dim, near things sharp
5 { 0.25 0.3 0.4 setrgbcolor 100 60 200 200 rectfill } blurred
1 0.8 0.3 setrgbcolor 360 110 120 120 rectfill
```

A halo is the shape's own light spread out, so a solid shape glows at
`8 1` and a thin stroke needs something like `24 1.6` to show at all.
Radii are in canvas units whatever the CTM is.

Finish a frame with `0.3 vignette` and a little `0.06 filmgrain` when the
piece wants a filmic surface. Both act on what is already drawn, so they
go last. (Grain and fine gradients look patterned in a contact sheet's
small thumbnails; judge them with `psanim frame`.)

## Particles without state

`spec proc particles` draws `proc` once for every particle alive now.
The spec says where particles are born, how many, how fast and under
which forces; `proc` draws one at the origin (it is already moved to the
particle) and reads it with `particleu` (0 at birth, 1 at death),
`particleage`, `particlevel`, `particlekey`. Nothing is simulated from
frame to frame: each particle's state is worked out from its birth and
the time, so any frame renders on its own.

```postscript
/sparks << /At [320 60] /Rate 90 /Life [1 1.8] /Speed [220 320]
           /Angle 90 /Spread 35 /Gravity 300 /Drag 0.4 >> def
sparks {
  1 particleu sub setalpha                       % fade out over its life
  [particlekey 1] hash01 0.12 mul 0.05 add 0.9 1 sethsbcolor
  0 0 3 1 particleu sub mul 1 add 0 360 arc fill
} particles
```

- **Bursts**: `/Bursts [0.5 2.1] /Count 120` emits 120 at each time.
  `/Shape /sphere` spreads them like a 3-D shell seen from the side,
  `/even` spaces them evenly round the circle.
- **Forces**: `/Gravity` pulls down (units/s/s), `/Drag` slows (per
  second, so speeds settle at gravity / drag: stars hang, then drift
  down), `/Wind [x y]` pushes steadily, `/Gust [x y]` pushes with a force
  that wanders over time (`/GustFreq`), and `/Sway [dx dy]` gives each
  particle its own wandering offset (snow, leaves, confetti).
- **Per-particle constants** come from `[particlekey i] hash01`: a size, a
  hue, a spin rate. They are the same on every frame.
- **Streaks**: `0.15 6 particletrail stroke` strokes the particle's own
  track over the last 0.15 s, so fast sparks read as lines.
- **Several emitters**: give an array of specs; `particleemitter` says
  which one a particle came from, and a spec's own `/Draw { }` draws its
  particles instead of `proc`.
- **A moving emitter**: `/At` takes keyframes, `[0 [100 80] /quad-out 1.2
  [500 600]]`; `/Inherit 0.3` gives particles some of its velocity.
- **Glow** goes around the whole call, never inside the procedure (one
  layer, not one per particle): `14 1.3 { spec { ... } particles } glowing`.

A firework: a burst at 0.4 s, stars streaking, glowing, falling slowly.

```postscript
14 1.4 {
  << /At [320 260] /Bursts [0.4] /Count 140 /Life [1.4 2.2] /Speed [140 220]
     /Shape /sphere /Gravity 70 /Drag 1.5 >> {
    1 0.75 0.35 setrgbcolor
    1 particleu sub dup mul setalpha                % fades late
    0.12 5 particletrail 2 setlinewidth 1 setlinecap stroke
  } particles
} glowing
```

Confetti: flat pieces that tumble. A spin is a rotation, a flip is a
scale whose sign changes (`cos`), which is what makes a piece flash as
it turns over.

```postscript
<< /At [320 0] /Bursts [0] /Count 90 /Life [3.5 4] /Speed [450 700] /Angle 90 /Spread 50
   /Gravity 160 /Drag 2.2 /Sway [25 0] /SwayFreq 0.8 >> {
  [particlekey 1] hash01 6 mul cvi
  [[0.95 0.3 0.4] [1 0.8 0.2] [0.3 0.7 1] [0.4 0.9 0.5] [0.8 0.4 1] [1 0.55 0.2]]
  exch get aload pop setrgbcolor
  particleage [particlekey 2] hash01 400 mul 100 add mul rotate
  1 particleage [particlekey 3] hash01 600 mul 300 add mul cos scale
  -5 -3 10 6 rectfill
} particles
```

Snow that loops: give `/Period` the movie's length and the emission
(and the wandering forces) repeat exactly, so the last frame leads into
the first. `/From` is ignored: the sky is already full at t = 0.

```postscript
%!PS
<< /Size [640 360] /Duration 6 /FPS 30 /Loop true /Background [0.08 0.1 0.16] >> animation
/snow << /At [320 380] /Area [700 0] /Rate 40 /Period 6 /Life 11 /Speed [35 55] /Angle -90
         /Spread 20 /Sway [18 6] /SwayFreq 0.4 /Gust [8 0] >> def
/scene {
  snow { 1 setgray 0 0 [particlekey 1] hash01 2 mul 1 add 0 360 arc fill } particles
} def
```

For a finished celebration, place `/fireworks` or `/confetti` from the
catalog ([catalog.md](catalog.md)): shells, stars, tumbling paper and a
message that pulses on each burst, already tuned; `psanim catalog
fireworks` prints the source to learn from.

Under the hood a particle is born at a time fixed by its emitter (a
burst, or a slot of the continuous rate) with properties from `hash01`
of its key, and only the live ones are visited, so a long film costs no
more late on. To do it by hand (a particle that follows a path, say),
the same idea is a few lines: a birth time, properties from `hash01`,
position as a formula in the age.

## A seamless loop

Declare `/Loop true`. The last frame must lead into the first exactly as
any frame leads into the next, so every moving thing has to be back where
it started when `progress` reaches 1.

```postscript
%!PS
<< /Size [480 480] /Duration 4 /FPS 30 /Loop true /Background [0.02 0.02 0.05] >> animation
/scene {
  240 240 translate
  0 1 35 { /i exch def
    gsave
      i 10 mul rotate
      /ph 1 turns i 20 mul add def               % one whole turn over the loop, each arm out of phase
      120 ph sin 60 mul add  [i 7] 0.9 loopnoise 14 mul add  0 translate
      i 36 div 0.7 1 sethsbcolor
      0 0 6 ph cos 3 mul add 0 360 arc fill
    grestore
  } for
} def
```

The rules that make a loop close:

- **Whole turns.** `k turns` is `k × 360 × progress`: use it for every
  angle, with `k` a whole number. Different whole `k` for different parts
  gives relative motion that still closes.
- **Symmetry buys slowness.** A part with n-fold symmetry (a 12-tooth
  gear, a 6-pointed star) looks the same after 1/n of a turn, so
  `1 n div turns` closes too, and that is the only way to have slow
  motion in a short loop. Then *everything drawn on that part* must share
  the symmetry: one odd mark and the join shows.
- **A half turn is not home for a tilted thing.** An ellipse (a ring seen
  at an angle) comes back to the same outline after half a turn, but its
  near and far sides have swapped.
- **Noise must loop too.** `wiggle` does not come back. `key radius
  loopnoise` does: it walks noise around a circle. For a noise *field*,
  do the same by hand: `x y  1 turns cos R mul  gnoise3`-style, moving
  the third coordinate around a circle (two looping coordinates need two
  circles added together).
- **Prove it.** `psanim check` measures the join against the steps beside
  it and warns if it is larger. `psanim sheet --seam 6` shows the last six
  frames and the first six side by side.

## Several scenes: sequence and reel

`sequence` plays procedures back to back, each with its own clock.
`reel` adds transitions that overlap the clips on either side; get the
total from `reelduration`.

```postscript
%!PS
/film [
  [3 /opening] [0.8 /dissolve] [3 /middle] [0.6 /wipe-left] [3 /ending]
] def
<< /Size [640 360] /Duration film reelduration /FPS 30 >> animation
/card { % r g b (text)
  4 1 roll backdrop
  /Helvetica-Bold findfont 40 scalefont setfont 1 setgray
  320 170 moveto showcenter
} def
/opening { 0.10 0.12 0.30 (one)   card } def
/middle  { 0.30 0.10 0.14 (two)   card } def
/ending  { 0.08 0.24 0.16 (three) card } def
/scene { film reel } def
```

Transitions: `/cut /dissolve /dip /wipe-left /wipe-right /wipe-up
/wipe-down /iris /push-left /push-right /zoom`. A clip need not paint a
ground: draw the shared world first, then `film reel` with clips that
draw only type or a subject, and they transition over it. In a
`sequence`, `null` in place of a procedure is a pause. Fade a clip's own text
out before its transition (see "Enter, hold, leave") so two captions
never overlap in a dissolve.

### Writing a transition

A transition is a procedure in the `transitions` dictionary. It gets
`procA procB u`: draw the frame that is `u` (0 to 1, linear) of the way
from clip A to clip B. A and B are ready-made procedures that draw each
clip; both clips' clocks keep running through the transition. Clip,
transform or fade them as you like:

```postscript
%!PS
% B appears through horizontal slats that open. The `t-` prefix tells
% `reelduration` this entry is a transition even if it is defined later;
% any other name must be defined before `reelduration` is called, or the
% movie fails to load, naming the entry.
transitions /t-shutter {
  /cubic-inout ease /u exch def /b exch def /a exch def
  a
  gsave
    newpath 0 1 8 { 40 mul 0 exch moveto 640 0 rlineto 0 u 40 mul rlineto -640 0 rlineto closepath } for
    clip newpath
    b
  grestore
} put
/film [ [2.5 /day] [1.2 /t-shutter] [2.5 /night] ] def
<< /Size [640 360] /Duration film reelduration /FPS 30 >> animation
/day   { 0.85 0.75 0.45 backdrop 1 1 0.9 setrgbcolor 500 now 12 mul 250 add 40 0 360 arc fill } def
/night { 0.06 0.08 0.20 backdrop 0.9 0.9 1 setrgbcolor 140 now 10 mul 250 add 26 0 360 arc fill } def
/scene { film reel } def
```

During a transition both clips are on screen. Get a character out of
frame before a wipe or push, or put it at the same place in both clips
for a dissolve; otherwise the audience sees two of it.

### Parallax

Depth from one number: draw each layer under its own camera, offset by a
fraction of the travel. Far layers move less, are dimmer, and can be
`blurred`.

```postscript
/travel 0 4 /sine-inout tween 900 mul def            % how far the camera has gone
[ [0.15 0.28 0.36 0.48 60] [0.45 0.18 0.24 0.34 110] [1.0 0.09 0.11 0.16 170] ] {
  aload pop /top exch def /b exch def /g exch def /r exch def /depth exch def
  gsave
    travel depth mul neg 0 translate                 % far layers (small depth) slide less
    r g b setrgbcolor
    0 1 30 { /i exch def
      i 120 mul 0 moveto
      i 120 mul 60 add  top [i depth] hash01 60 mul add  lineto
      i 120 mul 120 add 0 lineto closepath fill
    } for
  grestore
} forall
```

## Keyframes

When a value visits several places, list them: time, value, and
optionally the easing used to arrive.

```postscript
[ 0 [80 80]   1 [320 280] /cubic-out   2.5 [560 80] /bounce-out   4 [320 180] /cubic-inout ]
keyframes aload pop translate
1 0.5 0.3 setrgbcolor 0 0 24 0 360 arc fill
```

## Procedures that take time as a parameter

Write motion once against `now`, then reuse it under different scopes:
that is the point of time scopes.

```postscript
%!PS
<< /Size [640 360] /Duration 6 /FPS 30 >> animation
% A ring that expands and fades over one second of *its own* clock.
/ripple {
  0 1 /quad-out tween /u exch def
  1 u sub setalpha 0.5 0.8 1 setrgbcolor 3 setlinewidth
  0 0 10 u 120 mul add 0 360 arc stroke
  1 setalpha
} def
/scene {
  gsave 160 180 translate  0 1 /ripple during                    grestore   % once, at the start
  gsave 320 180 translate  1.5 /ripple looped                    grestore   % again every 1.5 s
  gsave 480 180 translate  0.5 { 3 /ripple timescaled } delayed  grestore   % half a second late, 3x fast
} def
```
