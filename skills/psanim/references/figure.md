# Characters that dance: pkfigure and Twiddle

A character is two procedures: a **pose**, numbers computed from a beat,
and a **drawing** that reads them. `pkfigure` gives you the parts every
cartoon dancer needs and the dance step itself, as numbers. It reads no
clock and keeps no state, so the character works under any time scope,
in any box, at any moment.

## Twiddle, ready-made

The catalog's `/twiddle` is the twiddle project's mascot: a round
clay-orange staticling with a peach belly, pink cheeks, big shiny eyes and
one antenna with a pink bobble. He keeps his own rules:

- **He dances only to a real beat.** By default that is his `/Tempo` (120
  bpm, counted from his own clock). With `/Beat /beat` it is an event
  track given with `--signals`, and then `/Level` can name a signal for
  the loudness. He hops higher on a loud beat, grins `^ ^` on a big one
  (loudness 0.75 or more), winks now and then, and sings a beat late: to
  a tempo his mouth opens to the beat before; to a track it follows the
  level from his second beat on.
- **With no music he sleeps**, with a z, and the picture is still. This
  covers `/Tempo 0`, an event track with no events, and the time before
  `/From` or two beats after `/Until`. When the music stops he settles,
  his eyes droop, and he falls asleep where he stands.
- **On a splash** (`/State /wave`) he waves, blinks and sways his bobble.
  He never hops or sings there, because nothing is playing.
- **One Twiddle at a time.** Placing two in one frame is an error.

```postscript
%!PS
<< /Size [640 360] /Duration 6 /FPS 30 /Background [0.11 0.07 0.06] >> animation
/tw /twiddle << /Params << /Until 4 /Accents [1 0.4 0.7 0.4] >> >> instance
/scene { /tw << /Box [170 0 300 300] /After /hold >> place } def
```

With `/Until`, place him with `/After /hold`: his last frame is asleep
and still, while the default `/loop` would wrap his clock and start the
music again. `psanim catalog twiddle` lists his parameters. Read his source the same
way to see the drawing.

Signals and events are read on the movie's clock, even inside a
component. His tempo counts from his own clock, which starts at `/At`.
His film is 32 seconds long and loops cleanly at tempos that are
multiples of 15 bpm. Give him a box about as wide as it is tall. He still
reads as a face at about 100 pixels.

## Your own: the rig

| word | gives |
|---|---|
| `beat dancepose` | the step: `/Hop /Squash /Lean`, `/ArmL /ArmR` (degrees raised), `/LiftL /LiftR /Spread`, `/Eyes` (`/open /grin /wink`), `/Look`, `/Mouth`, `/BobX /BobY` |
| `restpose`, `a b u blendpose` | standing asleep; easing from one pose to another (settling, waking) |
| `hop squash lean { } squashed` | the bounce, anchored to the feet: draw the figure standing on y = 0 |
| `x y angle len r limb` | an arm or a leg; leaves its end for the hand or foot |
| `x y r << /State /open >> eye`, `x y w << /Open u >> mouth` | faces: open, blink, `^` grin, sleeping lid; smile or singing D with a tongue |
| `t freq damp spring` | how far a bobble, an ear or a tail still lags, t seconds after a jolt |

`beat` is a dictionary you fill from whatever the music is: `/Beat`, the
index of the current beat; `/Since`, the seconds since it; `/Length`, the
seconds a beat lasts; `/Loud` and `/Sing`, both 0..1. Below, a backing
dancer for Twiddle in about twenty-five lines, on the movie's `tempo`:

```postscript
%!PS
<< /Size [640 360] /Duration 4 /FPS 30 /Background [0.08 0.1 0.14] >> animation
/pkfigure use
120 tempo                                   % the beat: countnow, from the clock

/frog { % pose
  begin
  Hop Squash Lean {
    0.35 0.75 0.3 setrgbcolor
    [ -1 1 ] { /sg exch def                 % legs and flat feet
      sg 0.4 mul 0.55 0  0.45 sg 0 lt { LiftL } { LiftR } ifelse sub  0.15 limb
      /fy exch def /fx exch def
      gsave fx sg 0.1 mul add fy translate 1 0.45 scale 0 0 0.3 0 360 arc fill grestore
    } forall
    -0.8 1.2 ArmL neg 0.6 0.12 limb 0.16 0 360 arc fill
     0.8 1.2 ArmR      0.6 0.12 limb 0.16 0 360 arc fill
    gsave 0 1.25 translate 1 0.8 scale 0 0 0.95 0 360 arc fill grestore  % body
    -0.45 1.9 0.32 0 360 arc fill  0.45 1.9 0.32 0 360 arc fill         % eye bumps
    [ -1 1 ] { 0.45 mul 1.92 0.17 << /State Eyes dup /wink eq { pop /grin } if /Look Look >> eye } forall
    0 1.3 0.35 << /Open Mouth /Tongue [1 0.5 0.55] >> mouth
  } squashed
  end
} def

/scene {
  320 60 translate 70 dup scale
  << /Beat countnow floor cvi /Since countnow dup floor sub 0.5 mul
     /Length 0.5 /Loud countnow floor cvi 4 mod 0 eq { 1 } { 0.5 } ifelse >>
  dancepose frog
} def
```

Lengths in a pose are in units of the body's half-width, so draw the
character about two units wide, standing on y = 0, and scale it into
place. Keep to the rule Twiddle keeps: let a character react to music
only when there is music. Feed `dancepose` from a real beat, and draw
`restpose` when there is none.
