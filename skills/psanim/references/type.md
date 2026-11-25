# Type: text in boxes, kinetic type, counters, captions

```postscript partial
/pktype use        % loads pkstage and pkmotion too
```

Scene bodies below run on a 640x360 canvas, 4 seconds, libraries loaded.
`psanim api pktype` lists every word. Set a font before any of these
(`/title 40 face`): they measure with it.

## Contents

- [Text in a box](#text-in-a-box)
- [Paragraphs](#paragraphs)
- [Kinetic type](#kinetic-type)
- [Marking a word](#marking-a-word)
- [Numbers that count](#numbers-that-count)
- [Typing](#typing)
- [Captions](#captions)
- [End credits](#end-credits)
- [Mathematics](#mathematics)
- [Hierarchy](#hierarchy)
- [An opening crawl](#an-opening-crawl)

## Text in a box

`(text) box /anchor showin` sets a line in a box; `fitface` picks the
largest size that fits, so a headline fills its column whatever the
words.

```postscript
ground
canvasbox 8 vmin inset 0.45 splity /low exch def /high exch def
/fg ink (FITS THE BOX) high /display fitface  (FITS THE BOX) high /c showin
/body 20 face /muted ink (centred in the lower box) low /c showin
(top right) low /ne showin  (bottom left) low /sw showin
```

`fitface` fits the string's width to the box's width and its capital
height to the box's height, whichever binds first. `showin` does not
shrink or wrap: a string wider than its box overflows it (use `fitface`
first, or `paragraph`).

`(text) box /anchor placetext` moves to where `showin` would start,
without showing: that is how `kinetic`, `typewrite` or anything else
that starts at the current point is put in a box.

```postscript
ground
/b canvasbox 10 vmin inset def
/fg ink (IN A BOX) b /display fitface
(IN A BOX) b /c placetext (IN A BOX) /glyphs 0.06 /mask-up kinetic
```

`(text) textbox` is the box a string would cover at the current point,
and `(text) (part) partbox` the box of one word of it: targets for
plates, markers, notes and camera moves.

## Paragraphs

`(text) box opts paragraph` wraps and sets text. `opts` is `/left`,
`/center` or `/right`, or `<< /Align /center /Leading 1.4 /VAlign
/middle >>`. `\n` forces a break. `textlayout` returns the lines and
their places instead of drawing them.

```postscript
ground /body 20 face /fg ink
(A frame is a function of its time. Nothing is remembered from one frame to the next, so any frame can be drawn alone.)
canvasbox 12 vmin inset << /Align /center /VAlign /middle /Leading 1.5 >> paragraph
```

## Kinetic type

`(text) /unit gap spec kinetic` shows a line at the current point with
each unit (`/glyphs`, `/words`, `/whole`) entering by a motion, `gap`
seconds apart. It runs on the current clock: wrap it in `after` or a
script step to start it later.

```postscript
ground
/display 64 face /fg ink
60 210 moveto (MOVE) /glyphs 0.07 /mask-up kinetic
/title 34 face /accent ink
1 { 60 150 moveto (one word at a time) /words 0.15 /rise kinetic } after
/body 20 face /muted ink
2 { 60 100 moveto (and a whole line at once) /whole 0 [/wipe-right 0.8] kinetic } after
```

A wrapped paragraph: `kineticparagraph`, with `/lines` as a further unit.

```postscript
ground /title 30 face /fg ink
(Every line of this paragraph rises into place, one after the other.)
canvasbox 10 vmin inset << /Align /left /VAlign /middle /Leading 1.5 >>
/lines 0.25 /mask-up kineticparagraph
```

To take kinetic text away, put it inside a scope with an end and wrap it
in `leaving`:

```postscript
ground /display 56 face /fg ink
0.3 3.2 { [/fade 0.5] { 320 (GONE SOON) stringwidth pop 2 div sub 160 moveto
                         (GONE SOON) /glyphs 0.05 /pop kinetic } leaving } during
```

For effects `kinetic` does not have, use the iterators directly:
`eachglyph`, `eachword` (and the core `eachspaced`, `eachonpath`) call
your procedure per unit with `(unit) i` on the stack and the current
point set; combine with `staggered`, `rank` and any motion.

```postscript
ground /title 40 face
120 160 moveto
(from the middle out) {
  /i exch def /w exch def
  i 4 /center rank 0.25 { /pop { /accent2 ink w show } entering } staggered
} eachword
```

## Marking a word

```postscript
ground /title 36 face
90 170 moveto
/accent ink (the important word here) (important) partbox 1 2 /cubic-inout tween marker
/bg ink 90 170 moveto (the important word here) show
% (dark text over the marker; on the words outside it, draw the line again in /fg first)
/fg ink 3 setlinewidth 90 110 moveto (underlined as it is said) textbox 2 3 /cubic-out tween underline
/body 22 face 90 110 moveto (underlined as it is said) show
```

`marker`, `underline` and `strikethrough` (a straight line through a
word: `box u strikethrough`) keep the current point. For a hand-drawn ring
or a strike-through use pkdiagram's `circled` and `struck` on the same
boxes.

## Numbers that count

A counter is a tween through `fmtnum` (thousands separators: 2,907) or
`fmtfixed` (none: 2907; right for years and identifiers). `n width
zeropad` gives (07), `seconds fmtclock` gives (01:05). `showright` keeps the last digit still while the number
grows.

```postscript
ground
/display 70 face /fg ink
520 170 moveto  0.3 3 /cubic-out tween 0 1284500 between 0 fmtnum showright
/body 20 face /muted ink 520 130 moveto (frames rendered) showright
/title 30 face /accent ink
520 70 moveto  0.3 3 /cubic-out tween 0 99.2 between 1 fmtnum (%) strcat showright
```

## Typing

```postscript
ground /mono 24 face /good ink
60 180 moveto (psanim render film.ps -o film.mp4) 0.4 2.6 ramp typewrite
```

### Typing like a person

`typewrite` reveals a string at an even rate. A person does not: keys come
in bursts, words are separated by longer gaps, there is the odd thinking
pause, and now and then a finger slips to a neighbouring key and the
mistake is backspaced and fixed. `typingplan` makes that schedule from a
seed, once, in setup; `typedat` then answers what is on the line at any
time, so it stays a pure function of the clock. Another seed types the
same text another way; `/Typos 0` types it cleanly; `/Slips [4]` makes
the fifth character slip for certain.

```postscript
%!PS
<< /Size [640 360] /Duration 6 /FPS 30 /Background [0.03 0.05 0.04] >> animation
/pktype use
/line (git push origin main) 11 << /Think 0.4 /Slips [5] >> typingplan def
/scene {
  /mono 28 face /good ink
  40 200 moveto (> ) show
  line now typedat show
} def
```

A whole terminal session is data too. `sessionplan` takes prompts, typed
commands, output that arrives at once (`/out`) or line by line (`/lines`),
a line redrawn in place (`/progress`), waits and marks, and `screenat`
gives the character grid at any time (the screen scrolls when it fills).
Because the screen is a function of time, the screen a moment ago is
`t 0.1 sub screenat`: that is how the catalog's `/terminal` fades text
out like a phosphor. For a screen to look at, place that entry; to act
when the session reaches a point, give it the plan and ask `plan /name
planmark` (see [catalog.md](catalog.md)).

```postscript
%!PS
<< /Size [640 360] /Duration 7 /FPS 30 /Background [0 0 0] >> animation
/pktype use
/session [
  [/type (make test)] [/lines (cc a.c\ncc b.c\nlink) 0.25]
  [/mark /ran] [/out (all 12 tests passed)]
] 3 << /Prompt ($ ) >> sessionplan def
/scene {
  /mono 24 face /good ink
  session now 9 50 screenat pop pop
  /rows exch def
  0 1 rows length 1 sub { /i exch def 40 320 i 28 mul sub moveto rows i get show } for
  session /ran planmark now lt { /accent ink 40 30 moveto (the tests ran) show } if
} def
```

## Captions

`[ [t0 t1 (text)] ... ] box opts captions` shows each caption in its
time, at the bottom of the box, on a plate. `/Karaoke true` lights the
words up across the caption's time.

```postscript
ground
/accent ink 320 200 now 30 mul 40 add disc
[ [0.2 1.9 (captions sit at the bottom of a box)]
  [2.1 3.9 (and can light up word by word)] ]
canvasbox 5 vmin inset << /Karaoke true /Size 20 >> captions
```

## End credits

`sections opts credits` rolls credits upward at a constant rate, as at
the end of a film. `sections` is an array of entries, each an array that
starts with a name:

| Entry | Sets |
|---|---|
| `[/heading (Cast)]` | a heading, centred, in the title face and the accent colour; a third element scales it: `[/heading (THE END) 2.5]` |
| `[/pair (Director) (Name) (Name2)]` | the role right-aligned on the left of a gutter, its names left-aligned on the right |
| `[/name (Name) (Name2)]` | centred names, one to a line |
| `[/text (a note)]` | a centred note, smaller and muted |
| `[/space n]` | n lines of space |
| `[/proc { ... } h]` | a slot h high: the procedure runs with the origin at its centre, over the column's centre (a logo) |

`opts` is a dictionary, every key optional. The roll moves through
`/Box` (the canvas) and everything is sized from the box: `/Size` is the
size of the names (4% of the box's shorter side), `/HeadingSize`,
`/RoleSize`, `/TextSize`, `/Leading`, `/Gutter` and `/HeadingGap` follow it
unless given. `/HeadingFace /RoleFace /NameFace /TextFace` and
`/HeadingInk /RoleInk /NameInk /TextInk` are theme roles (or font names
and colours), so a theme change restyles the whole roll. Lines fade over
`/Fade` of the box's height at its top and bottom (0.12).

Timing comes from either end. `/Speed` is units a second; `/Duration` is
the seconds the roll takes, and the speed follows. The clock is the
current scope's, so `after` places it like anything else; the roll starts
with the first card or, with none, with the first line entering at the
bottom edge. `/Cards [ sections ... ]` shows each section list centred
for `/CardTime` seconds (4) before the roll. `/Final sections` is a card
at the end of the same roll that stops centred in the box; `/Settle`
units before it the roll slows to the stop (0 keeps the rate constant
all the way), and `/FinalGap` is the space before it (half the box's
height, so the last credits have gone when it arrives).

```postscript
%!PS
/pktype use
/film [
  [/heading (Cast)]
  [/pair (The Keeper) (Odette Brandvold)]
  [/pair (The Visitor) (Imre Lindqvist)]
  [/space 1]
  [/heading (Crew)]
  [/pair (Directed by) (Fenwick Alder)]
  [/pair (Music by) (Lucian Marsh)]
  [/space 1]
  [/name (Thanks to everyone who held a lamp)]
] def
/opts << /Box [0 0 640 360] /Speed 44 /Settle 50
         /Cards [ [[/text (Lamp Oil Films presents)]] ] /CardTime 2
         /Final [ [/heading (The End) 2] ] >> def
<< /Size [640 360] /Duration film opts creditsduration 1 add /FPS 30 >> animation
/scene { ground film opts credits } def
```

`creditsduration` (the cards, then the roll to its stop) works before
`animation` when `opts` has a `/Box`, so the movie can be as long as its
credits. The other way round, leave out `/Speed` and give `/Duration`
(the movie's length minus the cards and a hold at the end). `creditsspeed`
and `creditsheight` report the speed and the content's height;
`creditsat` takes the distance rolled instead of using the clock, for a
roll that is eased or scrubbed by the caller. The catalog's `/credits`
(`psanim catalog credits`) wraps all of this with the sections as a
parameter, to place as a component.

**No shimmer.** Type that moves a fraction of a pixel a frame is
re-rasterised at a new subpixel position each frame and its edges
flicker. `credits` moves its whole content by a distance that puts it on
a whole device pixel, measured through the current transformation, so it
holds at any render scale, inside a placed component and under a camera
(checked on pixels at scales 1, 1.5 and 2, placed, and under `lookat`),
and the lines never move relative to one another. A frame is then the one
before it shifted by whole rows, give or take colour rounding. The snap is
skipped if the transformation is rotated. The speed is constant to within
half a pixel, and steps of 3 and 4 pixels a frame alternate if it is not a
whole number of pixels a frame; to avoid that, give `/Speed` as pixels a
frame times the frame rate divided by the render scale (4 px a frame at
30 fps and scale 1 is 120). In the `/Settle` tail, where the roll slows
below a pixel a frame, the snap makes it hold and hop a pixel rather than
glide. `/Snap false` turns it off.

## Mathematics

The text fonts have no pi, no arrows, no "not equal": **a non-ASCII
character in a string shows as a blank, silently.** `/name sym` shows a
sign from the Symbol font at the current size (`/neq /le /ge /approx
/times /dot /arrow /infinity /pi /theta /Sigma` ... the list is in
`psanim api sym`).

A line of mathematics is a **run list**: an array of `(text)`, `/name`
(a sign), `[/it (r)]` (italic, as variables are set), `[/sup x]`,
`[/sub x]` and `[/frac a b]`. `showruns` sets it; `runs i showrun` sets
one term where the line puts it, so an equation arrives term by term;
`runs i runbox` is a term's box, for a ring, a marker or a pulse that
ties it to the picture.

```postscript
/eq [ (Area = ) [/frac (1) (2)] ( ) /dot ( 2) /pi [/it (r)] ( ) /dot ( ) [/it (r)] ( = ) /pi [/it (r)] [/sup (2)] ] def
ground /serif 34 face
/x0 320 eq runwidth 2 div sub def
/fg ink
% the terms in four groups, each arriving on its own
[ [0 0] [1 6] [7 10] [11 14] ] 0.6 {
  /k exch def aload pop /last exch def /first exch def
  [/rise 0.5] { first 1 last { /i exch def x0 180 moveto eq i showrun } for } entering
} cascade
% then the base of the triangle is singled out
/accent ink 2.5 setlinewidth x0 180 moveto
eq 4 runbox eq 6 runbox boxunion circled 0 2.8 3.5 /cubic-inout tween trimpath stroke
```

For displayed formulas beyond this (integrals, matrices, aligned
derivations), PSCat's `lib/mathkit.ps` lays a TeX subset out as stroke
paths (see the psart skill); a path takes `trimpath` and any motion.

## Hierarchy

The chapter examples are drawn on a 640x360 canvas, so their numbers are
half what a 1280x720 film wants. At 1280x720: headline 70 to 110,
section heading 40 to 54, body 28 to 36, labels on diagrams and axes no
smaller than 22. In `vmin` these are 10 to 15, 5.5 to 7.5, 4 to 5 and 3.


- Three sizes are enough: a headline (`fitface` or 8 to 12 vmin), body
  (3.5 to 4.5 vmin), a small label (2 to 2.5 vmin). One weight contrast.
- One accent colour for the one thing the frame is about; everything else
  `/fg` and `/muted`.
- Leave text on screen long enough to read twice: about 0.3 s per word
  plus a second.
- Moving type should be legible while it moves: short distances
  (`/rise`, `/mask-up`), 0.4 to 0.8 s, gaps of 0.03 to 0.08 s per glyph
  and 0.1 to 0.2 s per word.

## An opening crawl

Text that recedes on a tilted plane towards a vanishing point, fading as
it goes, is the catalog's `/crawl` entry: give it title lines and
paragraphs, and it justifies them once in setup and draws each line on
the plane with `onplane` (see
[patterns.md](patterns.md#perspective-drawing-on-a-plane)). Its clock
runs 40 seconds and fits the text into them; `/Speed` in `place` makes
the whole crawl slower or faster.

```postscript
%!PS
<< /Size [1280 720] /Duration 12 /Background [0 0 0] >> animation
/intro /crawl << /Params <<
  /Title [ (PART ONE) (A LONG WAY ROUND) ]
  /Text [ (The ferry was late, the map was wrong, and the only road north had been closed for a hundred years.)
          (Nobody told the cartographer, who set out anyway.) ]
  /Font /Palatino-Bold /Size 52 /Width 900
  /Color [0.9 0.95 1] /Stars 300
>> >> instance
/scene { /intro << /Speed 3 >> place } def
```

At 720p, keep `/Size` at 40 or more where the plane meets the bottom
edge. Letters shrink with distance in width and, faster, in height (with
the square of it): a third of the way up to the vanishing point they are
two thirds as wide and under half as tall, and half way up a quarter as
tall, a texture rather than words. That is where `/Fade` (fractions of
the way up) should take them away.
