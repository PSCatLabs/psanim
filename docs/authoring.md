# Writing an animation

A psanim movie is one PostScript file. This page is the tutorial; the
chapters after it go deeper:

- [Patterns](../skills/psanim/references/patterns.md): recipes for the things you will want.
- [Craft](../skills/psanim/references/craft.md): timing, easing, composition, reviewing motion.
- [Pitfalls](../skills/psanim/references/pitfalls.md): what goes wrong and why.
- [Modes](../skills/psanim/references/modes.md): simulations, persistent canvases, inputs, motion blur.
- [Vocabulary](../skills/psanim/references/vocabulary.md): every word.

Those chapters are the same files the agent skill uses, so people and
agents learn from one text, and every example in them is run by the test
suite.

## A first movie

```postscript
%!PS
<< /Size [640 360] /Duration 4 /FPS 30 /Background [0.05 0.06 0.09] >> animation

/scene {
  320 180 translate
  now 90 mul rotate                      % a quarter turn per second
  0.9 0.4 0.2 setrgbcolor
  -50 -50 100 100 rectfill
} def
```

```sh
psanim check  first.ps                  # does it load, draw and move?
psanim sheet  first.ps -o sheet.png     # twelve frames on one image
psanim render first.ps -o first.mp4
```

Two parts:

1. **The declaration.** `<< ... >> animation` says what the movie is.
   `/Size` is the canvas in *units*, origin at the bottom left, y up; how
   many pixels that becomes is decided at render time (`--scale 2`,
   `--width 1920`), so the same program makes a thumbnail or a 4K master.
2. **`scene`.** The engine calls it once per frame. It reads the time in
   seconds with `now` and draws. That is the whole contract.

Everything else in the file is ordinary PostScript and runs once, before
the first frame: load libraries, define procedures, compute tables.

## Each frame starts clean

Before every frame the engine rolls the interpreter back to how setup
left it. So:

- A frame cannot depend on the frame before. Frame 300 is the same
  picture whether you render it alone or after the first 299, which is why
  `psanim frame --at 10` is instant and a render can use every core.
- Nothing carries over inside the scene: no font, no path, no colour, no
  variable you `def`ed last frame. Select the font in the scene. Derive
  everything from `now`.

## From time to value: `tween` and `between`

Almost every animated quantity is "go from A to B between two times, with
some character":

```postscript partial
0.5 2.0 /cubic-out tween        % 0 before t=0.5, 1 after t=2, eased in between
100 400 between                 % ...mapped to 100..400
```

Read it left to right: *when* (0.5 to 2.0), *how* (`/cubic-out`), *what*
(100 to 400). `between` also takes arrays, so the same line moves a point
or blends a colour:

```postscript
gsave
  0.5 2.0 /cubic-out tween [80 80] [480 260] between aload pop translate
  0.5 2.0 /linear    tween [1 0.3 0.2] [0.2 0.6 1] between aload pop setrgbcolor
  0 0 30 0 360 arc fill
grestore
```

## Time scopes: the idea that makes it compose

PostScript lets you draw a shape at the origin and place it with
`translate`. psanim lets you write a motion starting at time zero and
place it in time:

```postscript
%!PS
<< /Size [640 360] /Duration 6 /FPS 30 /Background [0.05 0.06 0.09] >> animation

% A dot that pops in over half a second of its own clock.
/pop-in {
  0 0.5 /back-out tween 24 mul
  0 0 3 -1 roll 0 360 arc fill
} def

/scene {
  0.6 0.8 1 setrgbcolor
  gsave 160 180 translate   0.5 /pop-in after           grestore   % starts at t = 0.5
  gsave 320 180 translate   2.0 /pop-in after           grestore   % the same motion, later
  gsave 480 180 translate   1.5 /pop-in looped          grestore   % again every 1.5 s
} def
```

Inside `after`, `during`, `delayed`, `looped`, `timescaled` and the rest,
`now` is local. `pop-in` never learns when it was called. That is what
lets a row of twenty items each run the same procedure a little later
than its neighbour (`i 0.08 { ... } staggered`), and what `sequence` and
`reel` are made of.

## Paths that draw themselves, and layers

```postscript
/Times-Italic findfont 96 scalefont setfont
320 150 moveto (hello) dup stringwidth pop 2 div neg 0 rmoveto true charpath
gsave 2 3 /quad-out tween { 1 setgray fill } faded grestore        % the fill fades up...
0 0.3 2.4 /cubic-inout tween trimpath                               % ...under an outline that draws on
1 0.8 0.3 setrgbcolor 2 setlinewidth stroke
```

`trimpath` keeps a fraction of whatever path is current. `faded` draws
its body as one translucent group; `glowing` and `blurred` are its
siblings.

## The loop

Write a little, then look:

1. `psanim check movie.ps`: errors come with the frame number and a
   trace through your own procedures. It also tells you if frames are
   blank, if nothing moves, if something pops, and if a loop fails to
   close.
2. `psanim sheet movie.ps`: is each frame a picture? For a particular
   moment, `--from 2 --to 3 --count 12`.
3. `psanim frame movie.ps --at 2.4`: one frame at full size.
4. `psanim render movie.ps -o movie.mp4` when it is right.

To see a value, print it: `now ==` inside the scene shows up in `psanim
frame` and `psanim check`.

## Everything PostScript already has

A scene is PostScript on the PSCat interpreter, so its libraries are
yours: palettes, turtles and L-systems, tilings, text layout and
watercolour brushes in `(lib/artkit.ps) run` and `(lib/paintkit.ps) run`;
charts and graphs; Type 3 display faces; native noise (`fbm3` takes time
as its third coordinate); gradients (`shfill`); transparency
(`setalpha`). Load libraries in setup, use them in the scene. The
[gallery](gallery.md) shows what that looks like.
