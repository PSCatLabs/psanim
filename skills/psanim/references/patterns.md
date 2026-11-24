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
- [Follow a path](#follow-a-path)
- [Morph one shape into another](#morph-one-shape-into-another)
- [A camera](#a-camera)
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

Both arrays are plain `[x y x y ...]`; compute them in setup when the
shapes do not change, and only call `morphpath` in the scene.

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

## Glow, blur, depth

```postscript
% light: draw bright things inside `glowing`
6 0.9 { 0.4 0.9 1 setrgbcolor 3 setlinewidth
        120 180 moveto 520 180 lineto stroke } glowing
% depth: far things blurred and dim, near things sharp
5 { 0.25 0.3 0.4 setrgbcolor 100 60 200 200 rectfill } blurred
1 0.8 0.3 setrgbcolor 360 110 120 120 rectfill
```

Finish a frame with `0.3 vignette` and a little `0.06 filmgrain` when the
piece wants a filmic surface. Both act on what is already drawn, so they
go last.

## Particles without state

Give each particle a birth time and properties from `hash01`; its position
is then a formula in its age. No simulation, any frame on its own.

```postscript
0 1 119 {
  /i exch def
  /born  i 0.03 mul def
  /life  1.2 [i 1] hash01 1.0 mul add def
  /age   now born sub life sub 0 lt { now born sub } { -1 } ifelse def
  age 0 ge {
    /u age life div def                              % 0..1 through its life
    /ang [i 2] hash01 360 mul def
    /spd 60 [i 3] hash01 140 mul add def
    1 u sub setalpha
    [i 4] hash01 0.15 mul 0.05 add 0.8 1 sethsbcolor
    320 ang cos spd mul age mul add
    120 ang sin spd mul age mul add  age age mul 90 mul sub      % gravity
    4 1 u sub mul 1 add 0 360 arc fill
  } if
} for
1 setalpha
```

## A seamless loop

Declare `/Loop true` and make everything a function of `progress` through
whole turns: `progress 360 mul` as an angle, `sin`/`cos` of it, or a
`looped` scope. `check` warns if the last frame does not lead into the
first.

```postscript
%!PS
<< /Size [480 480] /Duration 4 /FPS 30 /Loop true /Background [0.02 0.02 0.05] >> animation
/scene {
  240 240 translate
  0 1 35 { /i exch def
    gsave
      i 10 mul rotate
      /ph progress 360 mul i 20 mul add def      % each arm a little out of phase
      120 ph sin 60 mul add 0 translate
      i 36 div 0.7 1 sethsbcolor
      0 0 6 ph cos 3 mul add 0 360 arc fill
    grestore
  } for
} def
```

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
/wipe-down /iris /push-left /push-right /zoom`. Fade a clip's own text
out before its transition (see "Enter, hold, leave") so two captions
never overlap in a dissolve.

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
