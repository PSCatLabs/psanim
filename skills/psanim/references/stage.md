# Staging: boxes, themes, motions, scripts

The foundation the other libraries stand on. Read this one first.

```postscript partial
/pkstage use  /pkmotion use      % in setup, after `animation`
```

Examples here are scene bodies on a 640x360 canvas, 4 seconds long, with
the libraries loaded; a block starting `%!PS` is a whole movie. All of
them are run by the test suite. `psanim api pkstage` and `psanim api
pkmotion` list every word.

## Contents

- [The four separations](#the-four-separations)
- [Boxes: where things go](#boxes-where-things-go)
- [Themes: what things look like](#themes-what-things-look-like)
- [Motions: how things arrive and leave](#motions-how-things-arrive-and-leave)
- [Scripts: one thing after another](#scripts-one-thing-after-another)
- [Storyboards: naming the film's parts](#storyboards-naming-the-films-parts)
- [Rows of things](#rows-of-things)
- [The camera, by box](#the-camera-by-box)
- [Viewports: a scene inside a box](#viewports-a-scene-inside-a-box)
- [Trails](#trails)
- [Time in beats](#time-in-beats)
- [Any aspect ratio](#any-aspect-ratio)
- [Which words restart the clock](#which-words-restart-the-clock)
- [What goes wrong](#what-goes-wrong)

## The four separations

A good movie file keeps four things apart, so each can change alone:

| | is | lives in |
|---|---|---|
| **content** | what is drawn: a procedure that draws a thing at rest | `/card { ... } def` |
| **place** | where: a box `[x y w h]` | setup: `/left canvasbox ... def` |
| **look** | colour and type by role | the theme: `/accent ink`, `/title 40 face` |
| **time** | when, and by what motion | the scene: `2 /pop { card } enters` |

Draw every thing **at rest, as if it were always there**. Then say when
it arrives and how. You never write the arithmetic of an entrance.

## Boxes: where things go

A box is an array `[x y w h]` (bottom-left corner, size). Cut the canvas
up in setup and name the pieces; draw into them in the scene.

```postscript
/page canvasbox 6 vmin inset def                 % a margin all round
page 0.25 splity /body exch def /head exch def   % top quarter, and the rest
/cards body 3 2 vmin cols def                    % three columns
ground
/accent ink head 1 vmin roundpath fill
cards { /surface ink 1.5 vmin roundpath fill } forall
/fg ink cards 1 get /c boxpoint 3 vmin disc
```

- `n vmin` is n% of the canvas's shorter side: sizes written in it suit
  a landscape frame and a vertical one alike. `4 vmin` is body text,
  `6 vmin` a margin.
- `inset`, `insetxy` (different margins across and up), `subbox` (a
  fractional part: `box 0 0.6 1 0.4 subbox` is the top two fifths),
  `shifted`, `splitx`, `splity`, `cols`, `rows`, `cells`,
  `placed` (a w-by-h box set in another against an anchor), `boxat` (a
  box whose anchor is at a point), `boxunion`.
- `box /anchor boxpoint` gives a point: `/c`, `/n /s /e /w`, `/ne /nw
  /se /sw`, or `[fx fy]`.
- `box boxpath`, `box r roundpath` add the box to the path; then `fill`,
  `stroke`, `clip`.
- A box is a plain array of numbers, so **`between` and `keyframes`
  interpolate boxes**: a thing that moves from one box to another, or a
  camera that travels between them, is one line.

```postscript
/a [ 40 60 160 100 ] def  /b [ 380 200 220 120 ] def
ground
/accent2 ink
0.5 3 /cubic-inout tween a b between 12 roundpath fill
```

## Themes: what things look like

Name colours and fonts by what they are for.

```postscript
/paper usetheme                   % usually in setup; here, for this frame
ground
/title 44 face /fg ink     60 250 moveto (A title) show
/body 22 face  /muted ink  60 215 moveto (a line of supporting text) show
/accent ink 60 190 200 4 rectfill
/mono 18 face /accent2 ink 60 150 moveto (monospace) show
```

Roles: colours `/bg /surface /line /fg /muted /accent /accent2 /accent3
/good /bad`; fonts `/title /body /mono /display /serif`; sizes `/corner
/stroke`.

| theme | ground | `/title`, `/display` | `/body` | `/mono` |
|---|---|---|---|---|
| `/midnight` | near-black blue | Helvetica-Bold | Helvetica | Courier-Bold |
| `/paper` | warm white | Palatino-Bold, Bookman-Demi | Palatino-Roman | Courier |
| `/blueprint` | deep blue | AvantGarde-Demi | AvantGarde-Book | Courier-Bold |
| `/terminal` | black-green | Courier-Bold | Courier | Courier-Bold |
| `/poster` | yellow | Helvetica-Bold | Helvetica | Courier-Bold |
| `/dusk` | plum | Bookman-Demi | Bookman-Light | Courier |

Any of the standard 35 PostScript font names works as a role's value or
directly in `face`. Themes: `/midnight` (the default) `/paper /blueprint /terminal
/poster /dusk`. Change some roles and keep the rest:
`<< /accent [1 0.3 0.2] /title /Bookman-Demi >> usetheme`.

`/role ink` sets the colour; `/role size face` the font; `ground` paints
the background; `a b u mixink` is a colour between two roles (`/muted
/accent 1 2 ramp mixink ink`). Every pk word that takes a colour takes a
role or an `[r g b]` array.

## Motions: how things arrive and leave

A motion is a name. It applies to **any** drawing, because the engine
draws the body on a layer, measures what was painted, and moves that:
"scale about its own centre" needs no anchor from you. While a thing is
fully in place it is drawn directly and costs nothing extra.

```postscript
ground
/card { /surface ink [ 220 130 200 100 ] 12 roundpath fill
        /title 30 face /fg ink (hello) [ 220 130 200 100 ] /c showin } def
0.4 /pop { card } enters                                   % nothing, then it pops in and stays
/accent ink 1.5 /wipe-right { 220 110 200 5 rectfill } enters
2.5 /shake { /body 16 face /muted ink 320 80 moveto (and this shakes at 2.5) showcenter } emphasized
```

| word | stack | what |
|---|---|---|
| `enters` | `t spec { }` | absent before t, arrives at t, stays. Inside, `now` starts at 0 |
| `leaves` | `t spec { }` | there until t, gone by t (the motion run backward) |
| `onstage` | `a b in out { }` | arrives at a, has left by b |
| `staged` | `in out { }` | arrives at the start of the current time scope, leaves at its end |
| `emphasized` | `t spec { }` | drawn throughout; at t it does an emphasis |
| `entering` / `leaving` | `spec { }` | the same, at the start / end of the current time scope |
| `swapped` | `t spec { A } { B }` | A until t, B from t (B's clock starts at t; A's is the outer clock) |
| `motion` | `u spec { }` | you drive it: 0 absent, 1 in place |

A spec is `/name`, `[/name seconds]` or `[/name seconds /easing]`.

- **Entrances**: `/fade /rise /fall /slide-left /slide-right /pop /grow
  /zoom /drop /focus /spin /flip /unfold /unroll /wipe-right /wipe-left
  /wipe-up /wipe-down /mask-up /mask-down /mask-right`. An exit is an
  entrance run backward.
- **Emphases**: `/pulse /shake /wobble /jump /squash /blink /tada /dim`.
- The slides are named for the direction of travel: `/slide-left` comes
  from the right, moving left.
- A motion moves in the frame's own directions whatever transformation
  is in force: `/rise` under `90 rotate` still rises up the screen.
- `/mask-up` is the kinetic-type line reveal: the thing rises into its
  own box from below, cut off at the box's edge. `/unfold` grows from the
  bottom edge (bars); `/unroll` from the left (rules).
- Choosing: `/rise` or `/fade` for text, `/pop` for objects that should
  feel placed, `/wipe-right` for rules and underlines, `/drop` once per
  film at most.

Your own motion is a procedure in `motions` that takes u and returns a
dictionary (keys: `/dx /dy` in canvas units, `/dxrel /dyrel` in multiples
of the drawing's own size, `/scale /sx /sy /rotate /alpha /blur`,
`/wipe [l b r t]`, `/mask`, `/pivot [fx fy]`):

```postscript
motions /swing-in { 3 dict begin /u exch def
  << /rotate 1 u sub -70 mul /alpha u 0 1 clamp /pivot [0 1] >> end } put
ground
0.3 [/swing-in 1.2 /elastic-out] { /accent ink 240 140 160 80 rectfill } enters
```

## Scripts: one thing after another

Inside `{ ... } script` a **cursor** holds the time the next step starts.
`seconds { } then` runs its body from the cursor on (it stays) and moves
the cursor forward by `seconds`; `thenfor` runs it only for that long;
`wait` pauses. No times are written down, so inserting a step shifts
everything after it.

```postscript
ground /title 26 face
{
  0.5 { /fade { /fg ink 60 270 moveto (first this) show } entering } then
  0.5 { /rise { /fg ink 60 220 moveto (then this) show } entering } then
  0.3 wait
  1.2 { /fade /fade { /accent ink 60 170 moveto (this one only for a while) show } staged } thenfor
  0.5 { /pop { /good ink 90 110 14 disc } entering } then
} script
```

A film whose times are all absolute seconds is quick to write and slow
to change: inserting a two-second step means re-typing every later
number, and one missed number still passes `check`. If the film may be
re-timed, write its steps as a script (or its sections as storyboard
beats), and derive any time that depends on another (`/t-buckets 8.4
def ... t-buckets 3 gap mul add`) instead of copying it.

The seconds given to `then` are **the gap before the next step**, not how
long its body lasts: the body stays for good. Inside a body the clock
starts from 0, so `entering` is the natural entrance there. Scripts
nest (a step's body can be its own script).

## Storyboards: naming the film's parts

For a film in sections, name the sections and let their lengths set the
times. Retiming the film is then editing one list.

```postscript
%!PS
[ /title 2.5  /build 4  /payoff 3 ] storyboard
<< /Size [640 360] /Duration storylength /FPS 30 >> animation
/pkstage use /pkmotion use
/scene {
  ground /title 30 face /fg ink
  /title { /fade /fade { 320 180 moveto (a title card) showcenter } staged } inbeat
  /build { 0.3 /rise { 320 220 moveto (this arrives in the build) showcenter } enters } frombeat
  /payoff { /accent ink 0.2 /pop { 320 120 30 disc } enters } frombeat
} def
```

A beat that fades out followed by a beat that fades in leaves the frames
between them empty, and `check --all` reports blank frames. Carry
something across the cut: an element drawn outside the beats (a rule, a
progress bar: `/accent ink 0 0 width progress mul 4 rectfill`), a
heading that `swapped`s exactly on the beat, or a figure that moves
from its place in one beat to its place in the next (`u placeA placeB
between`, with times from `/name beat`).

`/name { } inbeat` runs the body during the beat (clock from 0, `span`
the beat's length); `/name { } frombeat` from its start to the end of the
film. `/name beat` is its start time and `/name beatspan` its start and
end, for arithmetic. `storyboard` goes before `animation`.

## Rows of things

`array gap { } cascade` runs the body per element with `item i` on the
stack, each one's clock `gap` seconds behind the last. Write the item as
if it were alone.

```postscript
ground
canvasbox 8 vmin inset 5 3 vmin cols 0.12 {
  /i exch def /b exch def
  /pop { i chartroles length mod chartroles exch get ink b 10 roundpath fill } entering
} cascade
```

For another order use `staggered` with `rank`: `i n /center rank 0.1 { }
staggered` starts the middle first (`/forward /reverse /center /edges
/random`).

## The camera, by box

`box lookat` frames a box: the whole of it in view, centred. Since boxes
interpolate, a camera move is a `keyframes` over boxes.

```postscript
/whole canvasbox def  /detail [ 420 60 160 90 ] def
ground
gsave
  [ 0 whole  1 whole  2.5 detail /cubic-inout  4 detail ] keyframes lookat
  /surface ink [ 60 60 220 240 ] 10 roundpath fill
  /accent ink detail 6 roundpath fill
  /body 8 face /bg ink (small print) detail /c showin
grestore
```

`lookat` fits the box to the whole canvas. To move a camera inside a
region (a stage under a fixed heading and above a caption), put it in
`within`, where the canvas is the region and drawing is clipped to it:

```postscript
canvasbox 0.2 splity /stage exch def /head exch def
ground /title 22 face /fg ink (the heading stays put) head /c showin
stage {
  gsave
    [ 0 canvasbox  1.5 canvasbox  3 [ 60 40 160 90 ] /cubic-inout ] keyframes lookat
    /surface ink [ 60 40 160 90 ] 8 roundpath fill
    /accent ink [ 380 150 160 90 ] 8 roundpath fill
  grestore
} within
```

`smoothkeys` instead of `keyframes` passes through each framing without
stopping: a tour. `boxA boxB boxunion` frames two things together.
`visiblebox` is what the frame shows, for skipping off-screen work.
`box amount spotlight` washes everything drawn so far, except the box,
toward the ground colour. It acts on what is already drawn: draw
afterwards anything that must stay at full strength.

## Viewports: a scene inside a box

`box { } within` is the spatial twin of `during`: inside it the origin is
the box's corner, drawing is clipped to the box, and `width` and `height`
are the box's. A scene written against `width` and `height` (and
`canvasbox`, `vmin`, `ground`) can be placed anywhere: side by side for a
comparison, small in a corner, full frame.

```postscript
/demo { ground /accent ink
  width 2 div  height 2 div  0 2 /sine-inout tween 6 20 between vmin  disc } def
canvasbox 4 vmin inset 0.5 splitx /r exch def /l exch def
l 1 vmin inset { /midnight usetheme demo } within
r 1 vmin inset { /paper usetheme demo } within
```

`box w h { } fitted` scales a scene drawn for a w-by-h canvas to fit the
box.

## Trails

`n gap { } echoed` draws its body as it was at n earlier moments, fading,
then as it is now. It works on anything that moves, because the body is
a function of its clock; nothing is remembered.

```postscript
ground
5 0.06 {
  /accent ink
  320 now 1.5 mul 57.3 mul cos 200 mul add  180 now 3 mul 57.3 mul sin 90 mul add  14 disc
} echoed
```

## What goes wrong

- **`item i` left on the stack in a `cascade`.** The body is handed
  `item i`; if it only runs a motion (`{ /pop { ... } entering }`) they
  stay there while the motion is absent. `def` them first: `{ /i exch def
  /item exch def /pop { ... } entering }`.
- **A film-wide value read inside a motion.** `enters`, `onstage`,
  `inbeat` and the rest restart the clock for their body, so a `tween` or
  `keyframes` written in film seconds gives the wrong answer there.
  Compute it into a name before the wrapper, or wrap it:
  `{ [ 0 100  9 400 ] keyframes } globally` (the value is left on the
  stack).
- **Operands outside the braces.** `(label) box 2 /pop { node } enters`
  leaves `(label) box` on the stack before t = 2, because the body is not
  run. Put everything the drawing needs inside: `2 /pop { (label) box
  /surface node } enters`.
- **A motion on something that covers the frame** (a `ground`, a
  full-canvas gradient) measures the whole canvas; `/pop` then scales
  the entire frame. Apply motions to things, not to grounds.
- **`leaving` at the top of a scene** departs at the end of the film,
  since the current scope is the whole movie. Use `leaves`, `onstage`, or
  put it inside `during` / `thenfor` / `inbeat`.
- **Many motions at once are many layers.** Thirty things entering in the
  same half second cost thirty layer composites for those frames (a few
  ms each at 720p). Fine for a render; if a preview crawls, put a group
  under one motion.
- **`use` inside `scene`** reloads the library every frame. Load in setup.
- **A theme set in setup is the film's theme**; `usetheme` inside a scene
  lasts for that frame only (every frame starts from setup's state), so
  call it at the same place every frame, or in setup.
- **Names**: the libraries keep their private names behind a dot or a
  prefix; yours should avoid the words listed by `psanim api`.

## Time in beats

For a film cut to a pulse, set a tempo and write times in beats.
`n counts` is n beats in seconds; `kick` is 1 on every beat and falls to
0 before the next; `countnow` is the beat the film is on.

```postscript
%!PS
100 tempo
[ /a 8 counts  /b 8 counts ] storyboard
<< /Size [640 360] /Duration storylength /FPS 30 >> animation
/pktype use
/scene {
  ground
  /surface /accent kick 0.5 mul mixink ink                    % the ground breathes on the beat
  canvasbox 6 vmin inset 2 vmin roundpath fill
  /display 60 face /fg ink
  /a { [ (ONE) (TWO) (THREE) (FOUR) ] 2 counts {
         /i exch def /w exch def
         0 2 counts [/pop 0.25] /fade { w canvasbox /c showin } onstage
       } cascade } inbeat
  /b { 0 counts [/mask-up 0.4] { (ON THE BEAT) canvasbox /c placetext (ON THE BEAT) show } enters } inbeat
} def
```

A sheet sampled exactly on a beat catches each entrance at its first
instant, when it is still absent: sample a little after (`--at 0.7,1.9`).
Hard cuts on the beat are reported by `check` as abrupt changes; that
warning is the cut, and is fine.

## Any aspect ratio

A film composed in boxes and `vmin` is a function of `/Size`. Take the
size from parameters and check the other shapes:

```postscript
%!PS
<< /Size [ /w 640 param /h 360 param ] /Duration 3 /FPS 30 >> animation
/pktype use
/scene {
  ground
  /page canvasbox 6 vmin inset def
  % one layout for wide frames, another for tall ones
  aspect 1 gt { page 0.5 splitx } { page 0.5 splity } ifelse /two exch def /one exch def
  /accent ink one 2 vmin inset 2 vmin roundpath fill
  /fg ink (FITS) two 2 vmin inset /display fitface (FITS) two 2 vmin inset /c showin
} def
```

`psanim sheet film.ps --param w=1080 --param h=1920`. What breaks first
is text fitted by width beside text placed by fraction: place a
neighbour relative to the fitted line (`fontsize`, `textbox`), not at a
fixed fraction of the page.

## Which words restart the clock

Inside these, `now` starts from 0 and a time written in film seconds is
wrong (use `globally`, or compute the value outside):

| word | inside its body, `now` is |
|---|---|
| `during`, `onstage`, `thenfor`, `inbeat` | seconds since the window opened; `span` is its length |
| `after`, `enters`, `then`, `frombeat` | seconds since the start time |
| `swapped` | B: seconds since the swap. A: unchanged |
| `delayed`, `staggered`, `cascade`, `echoed` | the outer clock, earlier by the delay (`span` is shorter by it too, so `turns` drifts: for a loop under a delay measure against `duration`) |
| `before`, `leaves`, `emphasized`, `entering`, `leaving`, `staged`, `motion`, `within`, `faded` | unchanged |
