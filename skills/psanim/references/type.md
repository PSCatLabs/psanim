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
- [Mathematics](#mathematics)
- [Hierarchy](#hierarchy)

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
