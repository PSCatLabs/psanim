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

## Painting over time: `paintplan`

A painting or a board is a list of strokes, each painted over a stretch
of time onto a persistent canvas (modes.md). `paintplan` takes the list
and does the bookkeeping, which is easy to get wrong by hand:

- **Each stroke is kept once, whole,** in `scene`, on the step that
  reaches its end (`plan paintkept`). Chalk, washes and dry brush are
  not opaque: a stroke regrown on the kept canvas every frame is laid
  down again each time (on the Chalk Lecture film, 5.8% more ink and up
  to 63 levels off), and built from pieces (`/From prev /To now`) it
  gains ink at its joins, 1.09 to 1.85 times at a light `/Touch`
  (PSCatLabs/pscat#1487).
- **Until then it is regrown from nothing in `overlay`** (`plan
  paintlive`), which is shown and never kept, so the growing stroke costs
  one stroke a frame and the kept canvas stays exact.
- **By step number, at any rate.** The scene steps at the declared FPS
  whatever `--fps` the film is rendered at, while the overlay runs at the
  output's; the plan counts the scene's own steps (`stepnow`, `stepat`),
  so a stroke is kept on exactly one step and shown in the overlay on
  every frame before it: it never vanishes for a frame, and seeking
  (`psanim frame --at`) gives the frame playing gives.
- **No accumulation.** The last frame is the board painted in one go
  (`plan paintall`) to the pixel; the strokes are kept in the order they
  end, and the overlay draws those in progress in the same order.

```postscript
%!PS
<< /Size [640 360] /Duration 4 /FPS 30 /Persist true /Background [0.17 0.25 0.21] >> animation
(lib/artkit.ps) run (lib/paintkit.ps) run (lib/drykit.ps) run
(lib/handscript.ps) run (lib/strokeglyphs.ps) run (lib/drychalk.ps) run
/pkink use
<< /Surface /greenboard >> dksurface
/chalk { % u ->   the stroke (its path is current) grown to u
  /ch-u exch def
  /white dcchalkcolor << /Width 4 /Hand 0.6 /From 0 /To ch-u >> dcchalk
} def
/plan [
  << /At 0.1 /For 1.3 /Path { 100 120 moveto 540 130 lineto } /Paint /chalk load >>
  << /At 1.6 /For 1.6 /Path { 320 200 80 270 630 arc } /Paint /chalk load /Ease /sine-inout >>
] paintplan def
/scene { plan paintkept } def        % kept: each stroke once, whole, when it is done
/overlay {                           % shown: the strokes in progress, and the chalk's tip
  plan paintlive
  plan painttip { pop pop 1 0.9 0.6 setrgbcolor 4 0 360 arc fill } if
} def
```

A stroke is a dictionary:

| key | | |
|---|---|---|
| `/At` | seconds | when it starts (movie time) |
| `/For` or `/Until` | seconds | how long it takes, or when it ends |
| `/Paint` | `u -> -` | draws the stroke as far as `u`; `1` is the stroke finished, and what is kept |
| `/Kind` | `/along` (default) | `/Path { ... }` builds its path, which is the current path when `/Paint` runs: `/From 0 /To u` grows a dry medium. With `/Reveal w` the library shows it through a band `w` wide along the path instead (a paint stroke that cannot grow), and `/Paint` gets 1 |
| | `/sweep` | `/Across [a b]`: a front moves from a to b (across x, or up y with `/Axis /y`) and `/Paint` (given 1) shows behind it: a wash crossing the page |
| | `/soak` | `/Passes n`: n passes laid one over another, evenly over its time, `/Paint` given (k+1)/n for pass k: a bloom that soaks outward, a glaze built up |
| `/Seed` | integer | given to `srand` just before `/Path` and `/Paint` (pass k of a soak: Seed + k), which pins every paintkit preset; default the stroke's index + 1. pkwash and pkpaper take their own `/Seed` |
| `/Ease` | name or `{ u -> u }` | paces an `/along` or a `/sweep` |
| lower case | anything | yours, kept: `plan /Strokes get k get /colour get` |

`plan painttip` leaves `x y angle k true` while a stroke is being drawn
(the tip on its path, or the middle of a sweep's front; `k` its index)
and `false` between strokes: a brush cursor in `overlay`. Errors name
the stroke (counting from 0) and the field: `paintplan: stroke 3: /For
is 0: how long the stroke takes, in seconds, more than 0`.

- **`/Paint 1` must be where `/Paint u` was heading.** The overlay shows
  `/Paint u` and the kept canvas gets `/Paint 1`; if they disagree the
  stroke jumps when it is kept. Dry media grown with `/To u` agree to the
  pixel; `trimpath` on a curve does not quite (pitfalls.md).
- **The procedures run with no dictionary of the library's open**, in
  their own `gsave`, after `newpath` (an `/along`: after its `/Path`),
  and must take `u` and leave nothing: one that does not is named.
- **Cost.** A frame pays for the strokes in progress, each drawn whole
  up to `u`, and for the units kept on its step; finished strokes cost
  nothing. Choosing them is a binary search and a pass over the plan's
  times, native.
- In a pure movie the same calls work, at the cost of repainting every
  finished stroke each frame: call `paintkept` then `paintlive` in
  `scene`. So does a component (components.md): its plan runs on its
  own clock, delayed, sped up or looping. A component has no kept
  canvas, so placed in a persistent movie it belongs in `overlay`; in
  the scene `paintkept` refuses.

`gallery/chalk-lecture/` (265 strokes of chalk in a hand's order) and
`gallery/evening-harbour/` (56 strokes of gouache: broad bands, sweeps, soaks, liner) are written this way.

A straight sweep and a bloom no longer need the tricks they did by hand
(the front on whole pixels, a dozen faint whole passes): a sweep in
progress is shown, not kept, and kept whole once; a soak is its passes.

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
