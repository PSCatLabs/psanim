# Beyond pure frames: state, persistence, inputs, blur

By default a movie is **pure**: frame = f(time). Prefer it. It renders in
parallel, seeks instantly, and cannot drift. Reach for the modes below
only when the piece needs what they give.

## Stateful: a simulation with `step`

Define `step` and the movie becomes stateful. The engine calls `step` at
a fixed rate (`/StepRate`, default the FPS) with the tick length on the
stack; what it `def`s **persists**. `scene` then draws the state, and
whatever *it* changes is rolled back as usual.

```postscript
%!PS
<< /Size [640 360] /Duration 6 /FPS 30 /StepRate 120 >> animation
/N 40 def
/px N array def /py N array def /vx N array def /vy N array def
0 1 N 1 sub { /i exch def
  px i 320 put  py i 300 put
  vx i [i 1] hash01 0.5 sub 400 mul put
  vy i [i 2] hash01 200 mul put } for
/step {                       % h: seconds per tick
  /h exch def
  0 1 N 1 sub { /i exch def
    vy i vy i get 500 h mul sub put                        % gravity
    px i px i get vx i get h mul add put
    py i py i get vy i get h mul add put
    py i get 20 lt { py i 20 put vy i vy i get -0.8 mul put } if      % floor
    px i get dup 20 lt exch 620 gt or { vx i vx i get neg put } if    % walls
  } for
} def
/scene {
  0 1 N 1 sub { /i exch def
    i N div 0.6 1 sethsbcolor
    px i get py i get 6 0 360 arc fill } for
} def
```

- Keep state in arrays or names made in setup. Strings are exempt from
  rollback in PostScript, so do not keep state in strings.
- The result does not depend on the output frame rate: ticks are fixed.
- `rand` inside `step` is seeded per tick, so a replay repeats exactly.
- Seeking backward replays ticks from the nearest checkpoint (the engine
  takes one every two seconds of movie). Frames render on one thread.
- No motion blur.

## Persistent: paint over the last frame

`/Persist true` stops the engine erasing between frames. Each call of
`scene` adds to the picture: trails, a painting built stroke by stroke,
feedback. Draw only what is new this frame.

```postscript
%!PS
<< /Size [640 360] /Duration 5 /FPS 30 /Persist true /Background [0.96 0.94 0.88] >> animation
/scene {
  % veil the past a little, then add this frame's mark
  0.96 0.94 0.88 setrgbcolor 0.04 setalpha 0 0 640 360 rectfill 1 setalpha
  0.1 0.2 0.5 setrgbcolor
  320 now 150 mul cos 240 mul add
  180 now 210 mul sin 120 mul add  6 0 360 arc fill
} def
```

- `scene` runs once per frame of the *declared* FPS, in order. Rendering
  at another `--fps` resamples those frames.
- To paint a list of strokes over time, index by `frame`: stroke
  `frame` is this frame's.
- Seeking restores a canvas checkpoint and repaints from there.

## Parameters

`/name default param` reads a host value: `psanim render m.ps --param
title=Hello --param count=12`. Call it in setup. `psanim info` lists the
parameters a program asks for.

```postscript partial
/title /title (Untitled) param def
/count /count 8 param cvi def
```

## Signals and events

A signal is a number that varies with time and comes from outside: audio
level, a data series, a pointer. An event track is a list of moments.
`psanim render m.ps --signals beats.csv` loads them from CSV: first column
time, other columns signals; a header ending in `!` is an event track.

```
t,level,beat!
0.00,0.10,
0.50,0.90,x
1.00,0.30,x
```

```postscript partial
/level signal 200 mul 40 add             % radius follows the level
/beat sinceevent 4 mul neg 2.718 exch exp   % 1 at each beat, decaying
```

In the browser player the host sets `mousex`, `mousey` (canvas units) and
`mousedown` before each frame; the same program runs offline with those
signals read from a file, or 0.

## Motion blur

`/MotionBlur 8 /Shutter 0.5` in the declaration (or `--motion-blur 8`)
renders 8 moments per frame and averages them. It costs 8x. Use it for
fast, large motion, on the final render only; `--motion-blur 0` turns it
off for previews. Pure movies only.
