# Ink: marks with a hand in them, and PSCat's media over time

```postscript partial
/pkink use                       % loads pkstage
(lib/artkit.ps) run (lib/paintkit.ps) run     % PSCat's own media, when wanted
```

Scene bodies below run on a 640x360 canvas, 4 seconds, libraries loaded.
`psanim api pkink` lists every word. For the media themselves
(watercolour, oil, chalk, charcoal, hatching, handwriting) see the
`psart` skill: everything it describes works inside a scene, and
[modes.md](modes.md) covers the persistent canvas that paintings want.

## A stroke with a shape

`[w ...] taperpath` turns the current path into the outline of a stroke
whose width follows the list along its length; `brushstroke` fills it.
`[0 9 0]` is a brush mark, `[14 0]` a comet, `[2 10 3 8 0]` an uneven
hand. Trim first and the mark draws itself.

```postscript
ground /fg ink
newpath 60 120 moveto 200 330 420 20 580 240 curveto
0  0.3 2.5 /cubic-inout tween trimpath
[ 1 12 4 10 0 ] brushstroke
```

## Anything, drawn on by a pen

`u width { ... } revealalong` runs a body clipped to a band along the
first `u` of the current path. The body draws a whole mark in any medium
(a paintkit stroke, chalk, a hatched ribbon, a gradient) and only the
part the pen has reached shows. This is how a textured stroke, which
cannot be trimmed, is made to draw itself.

```postscript
ground
newpath 70 180 moveto 220 300 400 60 570 180 curveto
0.3 3 /cubic-inout tween 46 {
  % the "medium": a ribbon of fine hatching along the same curve
  /accent ink 1.2 setlinewidth
  0 1 60 { /i exch def
    newpath 70 180 moveto 220 300 400 60 570 180 curveto
    i 60 div pathpoint /a exch def /y exch def /x exch def newpath
    gsave x y translate a 62 add rotate -22 0 moveto 22 0 lineto stroke grestore
  } for
} revealalong
```

With paintkit, put the brush call in the body and make it the same
stroke on every frame: `seed srand` before it for the presets that draw
from `rand`, and a constant `/Seed` in its options where it takes one.

On a **persistent** canvas, repainting the first `u` of a stroke every
frame is harmless for opaque ink and wrong for a translucent wash: each
frame would lay the wash down again. Paint only what the pen covered
since the last frame, with `u0 u1 width { } revealband`:

```postscript
%!PS
<< /Size [640 360] /Duration 4 /FPS 30 /Persist true /Background [0.96 0.94 0.90] >> animation
/pkink use
/pen { 0.5 sub 3 div 0 1 clamp /cubic-inout ease } def      % the pen's progress at a time
/scene {
  now dt sub pen  now pen  2 copy lt {
    newpath 60 120 moveto 220 330 420 30 580 240 curveto
    60 { 0.15 setalpha 0.2 0.35 0.6 setrgbcolor 0 0 640 360 rectfill 1 setalpha } revealband
  } { pop pop } ifelse
} def
```

A straight sweep (a wash crossing the page) wants its moving edge on
whole pixels, or the antialiased edges of successive slivers leave faint
seams in a translucent medium: round the edge to a multiple of 2 units.
A bloom (a wash soaking outward) is better built from a dozen faint
whole passes than from a growing clip.

## A shape that unrolls

`u anchor straightenpath` unbends the current path while keeping its
length, which `morphpath` does not: an arc becomes a strip of the same
length, a circumference is laid out flat. `anchor` is the point (0..1
along) that stays put.

```postscript
ground
/u 0.5 3 /cubic-inout tween def
0 1 5 { /i exch def
  /r 30 i 18 mul add def
  i 6 div 0.6 1 sethsbcolor 14 setlinewidth
  % each ring is cut at the top and opens about its lowest point
  newpath 320 250 r 90 450 arc  u 0.5 straightenpath stroke
} for
```

## Radial repetition, gradients, an uneven hand

```postscript
canvasbox /bg /surface 90 boxgradient
gsave 320 180 translate
  0 4 /linear tween 30 mul rotate
  12 { /i exch def
    i 12 div 0.6 1 sethsbcolor
    newpath 40 0 moveto 70 30 110 -30 150 0 curveto [ 0 10 0 ] brushstroke
  } around
grestore
/fg ink 0 1 9 { /i exch def
  60 i 56 mul add 40  i 6 jitter 3 -1 roll add 3 1 roll add exch  5 disc
} for
```

- `n { } around` runs the body n times, each turned 360/n further, with
  the copy's index on the stack.
- `box c0 c1 angle boxgradient` fills a box between two colours (roles
  or `[r g b]`).
- `key amount jitter` gives a small steady `dx dy`, the same for the
  same key on every frame: hand-set unevenness that does not flicker.

## Media and motion together

The libraries compose with PSCat's media in two ways.

- **In a pure movie**, a painted or drawn element is a procedure like any
  other: give it to `enters`, `kinetic`'s cousins, `revealalong`. Cost is
  the medium's, per frame, so keep painted areas small or few.
- **In a persistent movie** (`/Persist true`) the canvas accumulates, so
  each frame paints only what is new, and motion-graphics elements that
  must not stay (a caption, a pointer, a moving highlight) go in
  `overlay`. See modes.md.
