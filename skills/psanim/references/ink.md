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
(a paintkit stroke, a hatched ribbon, a gradient) and only the
part the pen has reached shows. This is how a textured stroke that
cannot be trimmed (a paint stroke) is made to draw itself. Chalk,
crayon and charcoal can do better: they grow (below).

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
stroke on every frame: `N srand` immediately before the call pins every
paint preset. `/Seed` is ignored by the presets that draw from `rand`
(the brushes); it pins only `pkwash` and `pkpaper`.

Dry media (chalk, crayon, charcoal, drykit's `dkstroke`) do not need a
clip: they grow. See the next section.

## Dry media: grow the stroke itself

`dcchalk`, `dkstroke`, `dycrayon` and `dhstroke` take `/From` and `/To`
(0 to 1, along each subpath). Since PSCat 0.8.0, `/From 0 /To u` with a
growing `u` is exact: the part already drawn keeps every pixel from
frame to frame (only the last dozen widths or so behind the tip move), and
`/To 1` is the whole stroke. The stroke lands, tires and lifts where
the whole one does, which a clip over a whole stroke cannot show.

```postscript
%!PS
<< /Size [640 360] /Duration 3 /FPS 30 /Background [0.17 0.25 0.21] >> animation
(lib/artkit.ps) run (lib/paintkit.ps) run (lib/drykit.ps) run
(lib/handscript.ps) run (lib/strokeglyphs.ps) run (lib/drychalk.ps) run
<< /Surface /greenboard >> dksurface
/scene {
  /white dcchalkcolor
  newpath 140 120 moveto 260 260 380 80 500 230 curveto
  << /Width 4 /Hand 0.6 /From 0 /To 0.1 2.5 /linear tween >> dcchalk
} def
```

- **Grow from 0, never in pieces.** Drawing `/From prev /To now` each
  frame onto a kept canvas is not exact: each piece's round end is
  painted again by the next ones, and the anti-aliased clip gains
  coverage on every repaint. PSCat measured the ink of such a stroke
  against the whole one at 1.001 to 1.006 times at the default touch,
  1.09 to 1.32 at `/Touch 0.5` and about 1.85 at `/Touch 0.3` for 300
  pieces (PSCatLabs/pscat#1487).
- **One stroke, one subpath.** `/From` and `/To` cut every subpath of
  the path at once, so `/To 0.5` on a word grows all its letters
  together. To write in order, split the path into subpaths
  (`pathforall`) and give each its own times. `/Corners /separate` cuts
  a box into four strokes that also grow together: draw each side as
  its own path.
- **Seeds.** The stroking words (`dcchalk`, `dkstroke`, `dycrayon`,
  `dhstroke`) take nothing from your `rand`: with no `/Seed` a stroke's
  texture is seeded from its first point's position on the canvas, so
  a stroke drawn alone is the same as in a whole board drawn at once.
  With `/Seed` each subpath's seed comes from it and the subpath's
  index (`dcchalk`: `Seed + 7919 i`), so splitting a path changes the
  seeds: leave it out, or give each stroke its own. `sgpath` and
  `mkpath` take their jitter from their own `/Seed`. drycrayon's path
  builders (`dywobble`, `dyscribble`, `dyloop` ...) do draw from
  artkit's `frnd`: build those paths in setup, or `N srand` first.
- **Cost.** A grown part costs about its share of the stroke's ink,
  plus working out the whole stroke's profile. One stroke a frame is
  cheap; a whole board redrawn every frame is not (seconds a frame).

### On a persistent canvas

Regrowing a stroke from 0 on the kept canvas lays it down again every
frame: chalk is not opaque, so it comes out heavier (on the Chalk
Lecture film, 5.8% more ink and up to 63 levels off over 13,648 pixels,
against the same board painted in one go). Keep each stroke once,
whole, and regrow the one in progress where nothing is kept:

1. In `scene`, on the one frame a stroke is finished, draw it whole.
2. In `overlay` (shown, never kept), while it is in progress, draw it
   `/From 0 /To u`.

Count in the scene's own steps, which are frames of the declared rate,
and name that rate once (`rate` below):

- **Keep a stroke on one step, found by number.** Not by testing whether
  its end falls between `now dt sub` and `now`: in floating point `now
  dt sub` is not exactly the last frame's `now`, so that test can be
  true on two frames, and a stroke painted twice is heavier
  (PSCatLabs/psanim#121). `t rate mul 1e-6 sub ceiling cvi` is the first
  step whose time is at or after `t`, except that a `t` less than a
  millionth of a step after a step's time counts as that step's.
- **In the overlay, ask whether the scene has kept it yet.** Rendered
  with `--fps`, the overlay runs at the output rate while the kept
  canvas still advances at the declared one, and `fps`, `frame` and
  `dt` there (and `dt` in setup) are the output's (PSCatLabs/psanim#122).
  The last step the scene has run when `now` is shown is `now rate mul
  1e-9 add floor cvi`. Until that reaches the stroke's step, draw it in
  the overlay, at most whole; otherwise it vanishes for a frame between
  the end of its growth and its keeping.

With this split, the Chalk Lecture's last frame matches the board
painted in one go to the pixel, at its own rate and under `--fps`.

```postscript
%!PS
/rate 30 def                    % the declared frame rate, named once
<< /Size [640 360] /Duration 4 /FPS rate /Persist true /Background [0.17 0.25 0.21] >> animation
(lib/artkit.ps) run (lib/paintkit.ps) run (lib/drykit.ps) run
(lib/handscript.ps) run (lib/strokeglyphs.ps) run (lib/drychalk.ps) run
<< /Surface /greenboard >> dksurface
% [start end {path}]: one subpath each, one after another
/plan [
  [ 0.1 1.4 { 100 120 moveto 540 130 lineto } ]
  [ 1.6 3.2 { 320 200 80 270 630 arc } ]
] def
/keptat { rate mul 1e-6 sub ceiling cvi } def     % end time -> the step that keeps the stroke
/stepnow { now rate mul 1e-9 add floor cvi } def  % the last step the scene has run
/chalkto { % u {path} ->
  /cp exch def /cu exch def
  /white dcchalkcolor newpath cp << /Width 4 /Hand 0.6 /From 0 /To cu >> dcchalk
} def
/scene {                       % kept: each stroke once, whole, when it is done
  plan { aload pop /cp exch def /t1 exch def pop
    stepnow t1 keptat eq { 1 /cp load chalkto } if } forall
} def
/overlay {                     % shown, not kept: a stroke from its start until it is kept
  plan { aload pop /cp exch def /t1 exch def /t0 exch def
    now t0 gt stepnow t1 keptat lt and {
      now t0 sub t1 t0 sub div 0 1 clamp /cp load chalkto } if } forall
} def
```

`gallery/chalk-lecture/` is a whole lecture written this way, 265
strokes in order with a hand's pace and a stick of chalk.

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
