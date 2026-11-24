# Beyond pure frames: state, persistence, inputs, blur

By default a movie is **pure**: frame = f(time). Prefer it. It renders in
parallel, seeks instantly, and cannot drift. Reach for the modes below
only when the piece needs what they give.

## Stateful: a simulation with `step`

Define `step` and the movie becomes stateful. The engine calls `step` at
a fixed rate (`/StepRate`, default the FPS); inside it `dt` is the tick
length, and what it `def`s **persists**. `scene` then draws the state, and
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
/step {
  /h dt def                   % seconds per tick
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

- `step` and `scene` are called on an empty stack. Inside `step`, `dt`
  is the tick length, `now` the tick's start time, `frame` the tick
  index; `hash01`, `wiggle`, `signal` and `rand` all work.
- **Frame 0 is drawn before any tick has run**: the scene sees the state
  exactly as setup left it. Frame *k* sees the state after
  `floor(k / fps × StepRate)` ticks. Initialise everything the scene
  reads in setup, not in the first `step`.
- Keep state in arrays or names made in setup. Strings are exempt from
  rollback in PostScript, so do not keep state in strings.
- The result does not depend on the output frame rate: ticks are fixed.
- `rand` inside `step` is seeded per tick, so a replay repeats exactly.
- Seeking backward replays ticks from the nearest checkpoint (the engine
  takes one every two seconds of movie). Frames render on one thread.
- No motion blur.
- **Iteration cost.** Every CLI invocation starts from tick 0, so `psanim
  frame --at 20` simulates twenty seconds before it draws, and a sheet of
  the last few seconds simulates the whole movie. While tuning, shorten
  the movie (`--duration`), lower the population with a `param`
  (`/birds /birds 1200 param def`, then `--param birds=200`), and look at
  late moments only when the early ones are right. `psanim bench` reports
  the cost of the first frames in order: ticks and drawing together.
- An interpreted simulation is slow: budget roughly 30 to 45 microseconds
  per particle per tick for a flocking rule set, and use a grid for
  neighbour search rather than all pairs.

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
- `/Background` is painted once, before the first frame. For a textured
  ground (paper), paint it yourself under `frame 0 eq { ... } if`.
- `rand` is reseeded the same way every frame, so call `srand` yourself
  when a mark's texture must not depend on which frame draws it.
- Inside the scene, `now dt sub` is the previous frame's time: an
  incremental painter needs both ends of the frame's interval.
- **Seeking is not free.** A frame is reached by painting every frame
  before it since the last checkpoint, and checkpoints live only as long
  as the process: `psanim frame --at 20` paints from the start, and so
  does every `sheet`. Iterate on late details with `--from`/`--to` on a
  shortened copy, or with the earlier strokes switched off.

### A stroke that travels across frames

One whole stroke per frame is easy (stroke number `frame`). A wash that
takes two seconds to cross the page is the real problem, because a
textured or translucent stroke cannot simply be drawn "a bit longer"
each frame: its texture would change and its alpha would stack. The
recipe that works:

1. Describe each stroke by a procedure that paints *all* of it, plus a
   start time, a duration and a seed.
2. Each frame, for every stroke in progress, clip to the sliver its
   brush front crossed during this frame (from the front's position at
   `now dt sub` to its position at `now`), `srand` the stroke's own seed,
   and run the whole procedure.

The slivers of successive frames tile the stroke exactly, so nothing is
painted twice and the texture is the same whichever frame reveals it.

```postscript
%!PS
<< /Size [640 360] /Duration 4 /FPS 30 /Persist true /Background [0.95 0.93 0.87] >> animation
% [ start dur seed proc ]: each proc paints one whole stroke
/plan [
  [ 0.2 1.6 11 { 0.20 0.35 0.60 setrgbcolor 0.5 setalpha
                 40 { rand 560 mod 40 add rand 60 mod 230 add rand 40 mod 10 add 0 360 arc fill } repeat } ]
  [ 1.4 1.8 23 { 0.75 0.40 0.25 setrgbcolor 0.5 setalpha
                 40 { rand 560 mod 40 add rand 60 mod 110 add rand 40 mod 10 add 0 360 arc fill } repeat } ]
] def
/scene {
  plan {
    aload pop /body exch def /seed exch def /dur exch def /t0 exch def
    /u1 now t0 sub dur div 0 1 clamp def            % how far the brush has got now
    /u0 now dt sub t0 sub dur div 0 1 clamp def     % ...and where it was a frame ago
    u1 u0 gt {
      gsave
        u0 640 mul 0  u1 u0 sub 640 mul 360 rectclip     % the sliver crossed this frame
        seed srand body
      grestore
    } if
  } forall
} def
/overlay {      % the brush itself: shown, never kept
  plan { aload pop pop pop /dur exch def /t0 exch def
    now t0 ge now t0 dur add le and {
      0.1 setgray now t0 sub dur div 640 mul 60 3 240 rectfill } if
  } forall
} def
```

A stroke pays its full cost on every frame it spans (the whole procedure
runs under the clip), so long travelling washes are the expensive part of
a painting.

### An overlay that is not kept

If a persistent movie defines `overlay`, it is drawn over each frame as
shown and then discarded: a brush or cursor, a caption, a wet sheen that
fades, a vignette. It runs like a pure scene (it reads `now`, may use
time scopes and layers) and cannot change the painting.

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
