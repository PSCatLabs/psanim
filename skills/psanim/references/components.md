# Components: movies inside movies

A component is a movie written on its own and placed inside another: a
flame, a clock, a title card, a spinner. Write it once, check and sheet it
by itself, then place it in any box, at any time, as often as you like.

Each instance has its own:

- **names**: its setup and scene run in a dictionary of their own, so its
  `/r 30 def` never meets the movie's `r` or another instance's;
- **clock**: inside, `now` and `globaltime` run from 0 at the moment it is
  placed, `duration` is its own /Duration, and `globally` returns to its
  start, not the movie's;
- **canvas**: `width` and `height` are its declared /Size, which `place`
  fits into the box you give;
- **seed**: `hash01`, `wiggle`, `loopnoise` and `movieseed` follow its
  /Seed, so two instances of one flame flicker differently;
- **parameters**: `param` reads the instance's /Params, never the movie's
  `--param` values. A name it is not given gets the default it wrote.
- **words**: PostScript's operators and the prelude's and libraries' words
  mean inside it what it was written against, even if the movie defines
  `/scale` or `/between` for itself. The movie's other names are visible
  to it, below its own.
- **theme and plot**: `usetheme` and `plotframe` inside it change its own
  copy, not the movie's.

## Three ways to get one

From the catalog of ready-made elements: give an entry's name.
[catalog.md](catalog.md) lists them with their parameters, and
`psanim catalog NAME` prints one's source to read or adapt.

```postscript partial
/busy /spinner component
/dots /spinner << /Params << /Style /dots /Color [1 0.7 0.3] >> >> instance
```


An entry takes its data the same way. The `chat` entry is a phone showing
a conversation: its `/Script` is a list of `[ sender (text) when ]`, where
the sender is `/me` or `/them` and `when` is a pause in seconds or a
dictionary (`/Typing`, `/At`, `/Time`, `/React`, see
`psanim catalog chat`). The length of the movie follows the script, so
`componentduration` times the film around it.

```postscript
%!PS
<< /Size [640 400] /Duration 12 /Background [0.9 0.92 0.96] >> animation
/pktype use
/talk /chat << /Params << /Contact (Ada) /Theme /mint /Script [
  [ /them (did the render finish? :eyes:) ]
  [ /me (yes! :rocket:) << /React /heart >> ]
  [ /them (show me) 0.6 ]
] >> >> instance
/scene { /talk << /Box [220 10 200 380] /After /hold >> place } def
```

From a file: any movie is a component. The file keeps its declaration,
setup and `scene`; read it with a name relative to the movie's own file.

```postscript partial
/flame (flame.ps) component
```

Inline, as a dictionary with the declaration's keys plus /Setup and
/Scene:

```postscript
%!PS
<< /Size [800 400] /Duration 4 /Background [0.05 0.05 0.08] >> animation

/spinner << /Size [100 100] /Duration 1.5 /Loop true
  /Setup { /Arms 5 param /arms exch def
           /Color [1 0.7 0.2] param /col exch def }
  /Scene { 50 50 translate 1 turns rotate col aload pop setrgbcolor 6 setlinewidth
           arms { 0 0 moveto 40 0 lineto stroke 360 arms div rotate } repeat }
>> component

% A second instance, set up again with other parameters.
/blue /spinner << /Params << /Arms 3 /Color [0.3 0.7 1] >> >> instance

/scene {
  /spinner << /Box [40 100 200 200] >> place
  /blue << /Box [300 50 300 300] /At 1 /Speed 2 >> place
  /spinner << /Box [650 150 100 100] /At 2 /Before /hold >> place
} def
```

## A catalog entry with parameters: fire

`/fire` is the catalog's flame, in two looks (`/Style /realistic`, with
glow, embers and smoke; `/Style /stylized`, flat tongues like a sticker).
One entry gives a candle, a campfire and a torch by parameters alone: the
height and width of the flame, its intensity (flicker, glow, embers), the
wind, the amount of smoke and embers, and a palette. Each instance has its
own /Seed, so two flames never flicker together. It declares /Loop true
and a 4 second period, so a film whose length is a whole number of fours
joins without a seam. A flame stands at the bottom centre of its box;
leave room above it for the embers and smoke, which are drawn inside the
box and clipped to it (the glow, being light, may leave a faint tail past
its edge).

```postscript
%!PS
<< /Size [1280 940] /Duration 4 /Loop true /Background [0.03 0.035 0.06] >> animation

% One entry, six instances: three flames in each look. The parameters, not the code,
% take it from a candle to a campfire to a torch.
/c0 /fire << /Seed 3 /Params << /Style /realistic /Height 0.26 /Width 0.07 /Intensity 0.45 /Embers 0 /Smoke 0.08 >> >> instance
/k0 /fire << /Seed 11 /Params << /Style /realistic /Height 0.5 /Width 0.8 /Intensity 1.3 /Embers 1.6 /Smoke 0.5 >> >> instance
/t0 /fire << /Seed 5 /Params << /Style /realistic /Height 0.6 /Width 0.28 /Wind 0.5 /Embers 0.8 /Smoke 0.2 >> >> instance
/c1 /fire << /Seed 3 /Params << /Style /stylized /Height 0.26 /Width 0.07 /Intensity 0.45 /Embers 0 >> >> instance
/k1 /fire << /Seed 11 /Params << /Style /stylized /Height 0.5 /Width 0.8 /Intensity 1.3 /Embers 1.6 >> >> instance
/t1 /fire << /Seed 5 /Params << /Style /stylized /Height 0.6 /Width 0.28 /Wind 0.5 /Embers 0.8 >> >> instance

% Each flame stands 40 units above the bottom of its 400 x 400 box, at its middle.
/stage { % y: a strip of ground
  gsave 0.07 0.06 0.075 setrgbcolor 0 exch 40 sub 1280 45 rectfill grestore
} def
/candleprops { gsave translate
    0.93 0.88 0.76 setrgbcolor -22 -92 44 92 rectfill
    0.80 0.74 0.62 setrgbcolor 8 -92 14 92 rectfill
    0.15 0.12 0.1 setrgbcolor 1.5 setlinewidth 0 -2 moveto 0.5 9 lineto stroke
  grestore } def
/campprops { gsave translate
    0.34 0.20 0.11 setrgbcolor gsave 14 rotate -110 -10 220 20 rectfill grestore
    0.27 0.15 0.08 setrgbcolor gsave -14 rotate -110 -10 220 20 rectfill grestore
  grestore } def
/torchprops { gsave translate
    0.24 0.15 0.09 setrgbcolor -9 -120 18 120 rectfill
    0.10 0.08 0.07 setrgbcolor -22 -20 44 30 rectfill
  grestore } def

/row { % y c k t: draw one row of three
  4 dict begin /t exch def /k exch def /c exch def /y exch def
  y 40 add stage
  c << /Box [ 20 y 400 400 ] >> place
  k << /Box [ 440 y 400 400 ] >> place
  t << /Box [ 860 y 400 400 ] >> place
  220 y 40 add candleprops
  640 y 40 add campprops
  1060 y 40 add torchprops
  end } def

/scene {
  540 /c0 /k0 /t0 row
  140 /c1 /k1 /t1 row
} def
```

### Reacting to a catalog entry's events

The celebrations (`/fireworks`, `/confetti`) take their burst times as a
parameter, `/Bursts`. Pass your own array and the same array is an event
track your movie owns: `sincelast` says how long ago the latest one was,
on the clock of the scope it is called in, so read it inside the same
delay the instance is placed at.

```postscript
%!PS
<< /Size [640 360] /Duration 5 /Background [0.04 0.04 0.08] >> animation
/booms [0.8 1.6 2.3] def
/show /fireworks << /Params << /Bursts booms /Message (Shipped) /Detail () /Sky false >> >> instance
/scene {
  /show << /At 0.5 >> place
  0.5 {                                   % on the instance's clock
    booms sincelast -6 mul 2.718 exch exp  % 1 at each burst, falling away
    1 0.8 0.3 setrgbcolor 0.6 mul setalpha 0 0 width 12 rectfill
  } delayed
} def
```

## Placing

`/name opts place`, in the scene. All options are optional:

| Key | Meaning | Default |
|---|---|---|
| /At | movie time (in the current scope) it starts | 0 |
| /Speed | how fast its clock runs | 1 |
| /Box | [x y w h] its /Size is fitted into, centred | the whole view |
| /Before | /hide, or /hold its first frame before /At | /hide |
| /After | /hide, /hold its last frame, or /loop | /loop if it declared /Loop true, else /hide |

`place` is itself drawing: wrap it in `faded`, `blurred`, a motion from
pkmotion or a camera like any other drawing. It respects the time scope it
is called in, so `2 { /flame << >> place } delayed` starts the flame two
seconds later. Use `componentduration` to time the movie around one.

`place` keeps the component's shape, so a box of another shape leaves
bars on two sides. The catalog's panels take /Width and /Height
parameters so an instance can be made the shape of its box.

A component paints its /Background into its box only if it declares one;
otherwise it is transparent and draws over the movie.

## A worked example: The Builds Passed

`gallery/builds-passed/builds-passed.ps` is a whole film made of catalog
components, with a host of about a hundred lines: a terminal types
`cargo test`, test dots fill in, the pipeline lights up a stage at a time,
the screen clears and fireworks and confetti burst in around "Huzzah! The
builds passed". With `--param result=fail` the same host ends the film
differently (a red dot, a failed stage, no celebration). Its key lines, shortened,
are below. Read it for the
pattern:

1. **Make the script once, as data**, and read times off it. The terminal
   is given the plan (`/Plan session`), so the host can ask it when things
   happen with `planmark`, and everything else is timed from those marks.
2. **Make each part with `instance`** and its parameters, before the
   declaration if the film's length depends on one (`componentduration`).
3. **Place each in time and space** in `scene`: `/At`, `/Box`, and
   `/Before /hold /After /hold` to keep the first and last picture.
   Boxes are plain arrays, so `u pbox pend between` slides a part from
   one place to another.
4. **React to an event** by owning its track: the same `/Bursts` array goes
   to the entry and, shifted to the movie's clock, to `sincelast`.

```postscript partial
/result (pass) param cvn /result exch def  /ok result /pass eq def     % --param result=fail

/session [ [/type (cargo test)] [/mark /built] [/lines tests 0.13]    % a plan the host can ask
           [/mark /ran] [/out ok { (24 passed) } { (1 failed) } ifelse] [/mark /result]
         ] 5 << /Prompt ($ ) >> sessionplan def
/tb session /built planmark def   /tr session /ran planmark def   /tt session /result planmark def

/tty  /terminal << /Params << /Plan session >> >> instance
/ci   /pipeline << /Params << /Fail ok { -1 } { 1 } ifelse /Times [ 0 tb tt tt 1 add ] >> >> instance
/dots /testdots << /Params << /Fails ok { [] } { [12] } ifelse /Seconds tr tb sub >> >> instance
/cheer /fireworks << /Params << /Style /finale /Bursts bursts /Message (Huzzah!) >> >> instance
/fstart tt 2 add def                                                  % the finale, on the movie's clock
/hostbursts [ bursts { fstart add } forall ] def                      % the same event track, to react to
/finish ok { fstart /cheer componentduration add } { tt 5 add } ifelse def

/scene {
  /tty  << /Box [30 60 800 600] /After /hold >> place
  /ci   << /Box [860 520 390 78] /After /hold >> place
  /dots << /Box [880 150 350 190] /At tb /Before /hold /After /hold >> place
  ok { /cheer << /At fstart >> place } if
  hostbursts sincelast -9 mul 2.718281828 exch exp                    % 1 at each burst, falling away
  0.3 mul setalpha 1 1 1 setrgbcolor 0 0 width height rectfill        % a flash on every burst
} def
```

## Limits

- A component cannot `/Persist` or have a `/Step`: those run only as the
  movie itself.
- Component names are global: two `component` calls with one name replace
  the first.
- Its setup runs when `component` or `instance` is called, so call them in
  setup, not in the scene (where they would run every frame).
- `vignette` and `filmgrain` inside a component act only on its box, and
  blur and glow stay inside it too, as they stay inside any clip (a
  `within` box, a `rectclip`) in force when they begin.
- A component's /Size and /Duration may depend on its parameters: read them
  with `param` before `animation` (the instance's /Seed is already in force
  there). `/terminal` and `/pipeline` size and time themselves that way.
- A file component is read from the movie's own directory and must be
  inside the read roots (`--read-root`).
