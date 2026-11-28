# Storytelling: what to do with the vocabulary

The libraries have no `introduce` or `compare` word, on purpose: those
are shapes a film takes, and each is a few lines of the words you have.
This chapter is those shapes. Scene bodies run on a 640x360 canvas, 4
seconds, libraries loaded; `%!PS` blocks are whole movies.

## Contents

- [The grammar of an explainer](#the-grammar-of-an-explainer)
- [Introduce one idea](#introduce-one-idea)
- [Compare two alternatives](#compare-two-alternatives)
- [Follow a process](#follow-a-process)
- [Reveal the inside of something](#reveal-the-inside-of-something)
- [Cause and effect](#cause-and-effect)
- [Single out a detail](#single-out-a-detail)
- [Summarize](#summarize)
- [Join two beats with a transition](#join-two-beats-with-a-transition)
- [Planning a film](#planning-a-film)

## The grammar of an explainer

1. **One new thing at a time.** Each step adds one element or changes
   one. What is already there stays put, so the eye goes to the change.
2. **Say it, then show it, then mark it.** A short heading states the
   point; the picture makes it; an emphasis (an accent colour, a
   `spotlight`, a `note`) ties the words to the place.
3. **Things keep their places.** Lay boxes out once in setup. A node
   that moves between scenes is a different node to the viewer; if the
   view must change, move the camera (`lookat`), not the things.
4. **Motion means something.** Arrival: an entrance. Relationship: an
   edge drawing from one thing to another. Transfer: a pulse along it.
   Importance: an emphasis. Nothing else should move.
5. **Time to read.** A heading needs about 0.3 s per word plus a second;
   a diagram step, at least a second before the next.

Write the film as a `storyboard` of beats, each beat a `script` of steps.

## Introduce one idea

A name, a one-line definition, and the smallest picture that shows it.
The name arrives first and stays; the picture answers it.

```postscript
ground
canvasbox 10 vmin inset 0.42 splitx /pic exch def /words exch def
{
  0.8 { /display 34 face /fg ink words /nw boxpoint 34 sub moveto
        (Time scope) /glyphs 0.03 /mask-up kinetic } then
  1.0 { /body 15 face /muted ink
        (a procedure’s clock, re-based: the way translate re-bases space)
        words 0 -60 shifted << /Leading 1.5 >> /lines 0.15 /rise kineticparagraph } then
  0.4 { /pop { (now = 0) pic 150 60 /c placed << /Fill /accent /Ink /bg >> node } entering } then
} script
```

## Compare two alternatives

Draw **one** scene procedure that reads a setting, and place it twice
with `within`. The two halves are then guaranteed to differ only in what
you are comparing.

```postscript
/demo {            % reads /easing
  ground
  /muted ink /body 3 vmin 9 max face canvasbox 4 vmin inset /nw boxpoint 12 sub moveto easing 20 string cvs show
  /accent ink
  0.3 3.3 easing tween  width 0.15 mul width 0.85 mul between  height 2 div  height 0.07 mul disc
} def
canvasbox 0.5 splitx /r exch def /l exch def
l 2 vmin inset { /easing /linear def demo } within
r 2 vmin inset { /easing /back-out def demo } within
/line ink 1 setlinewidth 320 30 moveto 320 330 lineto stroke
```

For before and after of the same thing in the same place, use `swapped`
or a `codediff`. For a ranking, `hbars` with `/Ranks`.

## Follow a process

The steps are boxes in a row; one highlight moves along them while a
caption says what is happening. The highlight's place is a `keyframes`
over the boxes.

```postscript
/steps canvasbox 12 vmin inset 100 60 /c placed pop
       canvasbox 14 vmin inset 0.55 splity pop 4 4 vmin cols def
/names [ (parse) (plan) (run) (render) ] def
ground
0 1 3 { /i exch def names i get steps i get /surface node } for
0 1 2 { /i exch def steps i get steps i 1 add get /straight edge } for
% the moving frame: one box interpolated through the four
/accent ink 3 setlinewidth
[ 0 steps 0 get  1 steps 1 get /cubic-inout  2 steps 2 get /cubic-inout  3 steps 3 get /cubic-inout ]
keyframes -4 inset 10 roundpath stroke
% the caption for the current step
/body 16 face /fg ink
[ [0 1 (read the program)] [1 2 (decide what is live)] [2 3 (call scene)] [3 4 (paint the pixels)] ]
canvasbox 6 vmin inset << /Size 16 /Plate false >> captions
```

## Reveal the inside of something

Show the whole, then move the camera onto one part and let its contents
arrive. The outer picture stays underneath, so the viewer knows where
they are; `lookat` over two boxes is the move.

```postscript
/whole canvasbox def
/eng [ 250 140 140 80 ] def
/inner eng 8 inset 3 6 cols def
ground
gsave
  [ 0 whole  1 whole  2.2 eng -10 inset /cubic-inout ] keyframes lookat
  (source) [ 40 140 140 80 ] /surface node
  (frames) [ 460 140 140 80 ] /surface node
  1.4 /fade { (engine) eng /surface node } { /line ink 2 setlinewidth eng 10 roundpath stroke } swapped
  [ (clock) (scopes) (layers) ] 0.2 {
    /i exch def /nm exch def
    2.2 /pop { nm inner i get << /Fill /accent /Ink /bg /Size 7 >> node } enters
  } cascade
grestore
```

## Cause and effect

Something happens here; a pulse travels; something answers there. Time
the answer to the pulse's arrival.

```postscript
/a [ 60 150 150 70 ] def /b [ 430 150 150 70 ] def
ground
1.0 /squash { (request) a /surface node } emphasized
a b << /Pulse 1.2 2.2 /cubic-inout tween /PulseLen 0.3 >> edge
2.2 /pulse { (response) b << /Fill /surface /good 2.2 2.5 ramp mixink >> node } emphasized
```

## Single out a detail

Three strengths, weakest first: an accent colour on the one thing; a
mark on it (`circled`, `underline`, `note`); a `spotlight` that puts the
rest in shadow. Use one, and take it away again when the point is made.

```postscript
/cells canvasbox 12 vmin inset 4 2 3 vmin cells def
ground
0 1 7 { /i exch def i 8 string cvs cells i get /surface node } for
cells 5 get  1 2 /quad-out tween 3 3.6 /quad-in tween sub 0.65 mul  spotlight
/accent ink /body 15 face (this one) cells 5 get /s 1.6 2.2 ramp note
```

## Summarize

Bring the film's ideas back as a short list, each with the small picture
or colour it had, one at a time; then hold. The recap is the frame people
screenshot: make it complete on its own.

```postscript
ground
/title 26 face /fg ink 60 290 moveto (In short) show
[ (a movie is a program) (a frame is a function of time) (so any frame renders alone) ] 0.5 {
  /i exch def /line exch def
  [/rise 0.5] {
    chartroles i get ink 70 232 i 52 mul sub 7 disc
    /body 20 face /fg ink 92 225 i 52 mul sub moveto line show
  } entering
} cascade
```

## Join two beats with a transition

Between storyboard beats the picture cuts. Where a cut is only a cut
because nothing else was available, write a transition between the two
beats, as between a reel's clips: any name in `transitions`.

```postscript
%!PS
[ /ask 3  [0.8 /dissolve]  /answer 3  [0.6 /push-left]  /recap 2.5 ] storyboard
<< /Size [640 360] /Duration storylength /FPS 30 >> animation
/pkstage use /pkmotion use
/scene {
  % a beat with a transition paints its own ground, first
  /ask    { ground /title 34 face /fg ink 320 170 moveto (Why is the sky blue?) showcenter } inbeat
  /answer { ground /body 22 face /fg ink
            0.3 /rise { 320 190 moveto (short waves scatter more) showcenter } enters
            /accent ink 1.2 /pop { 320 130 24 disc } enters } inbeat
  /recap  { ground /title 30 face /accent ink 320 170 moveto (Rayleigh scattering) showcenter } inbeat
  % drawn after the beats, so it stays put through every transition
  /accent ink 0 0 width progress mul 4 rectfill
} def
```

What it does to time is what a reel does: the transition **overlaps**
the beats beside it. `answer` starts 0.8 s before `ask` ends, so
`/answer beat` is 2.2, the film is 3 + 3 + 2.5 - 0.8 - 0.6 = 7.1 s, and
`storylength` says so. Every beat keeps its full length and its own
clock: in the dissolve `ask`'s body sees `now` 2.2 to 3 and `answer`'s
0 to 0.8, so neither is drawn outside its beat. Inside a beat nothing
changes; only absolute times written against the old cut move.

Three rules make it work:

- **A beat with a transition paints its own ground** (`ground`, or
  `backdrop` with a colour), as a reel clip does. A dissolve, a wipe or
  an iris draws the outgoing beat, then the incoming one over it, and
  never fades the outgoing one: whatever the incoming beat leaves
  uncovered stays at full strength and vanishes when the transition
  ends. (`/dip`, the pushes and `/zoom` move or fade both sides.)
- **Write the `inbeat` calls in the storyboard's order.** Each draws its
  own side of the transition, so the incoming beat must be drawn second;
  the other way round is an error, `draw /ask before /answer`.
- **Draw what the beats share after them.** A progress bar, a frame, a
  `frombeat` element: they stay still while the beats change under them.
  Something drawn *between* two `inbeat` calls lands on the outgoing
  beat and under the incoming one, so it dissolves away with the
  outgoing beat; that cannot be detected. A `frombeat` body comes in
  with the transition into its beat and is never taken away by a later
  one.

Keep a deliberate cut a cut: a hard change of subject, a cut on the
music, or a hand-off where one beat's figure moves to its place in the
next. Transitions cost the overlap's length and the drawing of two
beats at once.

## Planning a film

Before writing PostScript, write the storyboard as a list: for each beat,
the one sentence it must leave in the viewer's head, what is on screen,
and how long. Then:

- lay out every box for the whole film in setup, so things keep their
  places from beat to beat;
- choose a theme and at most two entrance motions for the whole film
  (one for objects, one for text);
- write each beat as a procedure that draws its own things, called
  with `inbeat` (it leaves at the end) or `frombeat` (it stays);
- `psanim sheet --count 16`, read the sheet as a comic strip: if the
  story is not there in stills, motion will not add it.
