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

## Two ways to write one

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

A component paints its /Background into its box only if it declares one;
otherwise it is transparent and draws over the movie.

## Limits

- A component cannot `/Persist` or have a `/Step`: those run only as the
  movie itself.
- Component names are global: two `component` calls with one name replace
  the first.
- Its setup runs when `component` or `instance` is called, so call them in
  setup, not in the scene (where they would run every frame).
- A file component is read from the movie's own directory and must be
  inside the read roots (`--read-root`).
