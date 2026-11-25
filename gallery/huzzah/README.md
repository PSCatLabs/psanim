# Huzzah

A 12 second celebration card (1280x720, 30 fps). A release pipeline turns
green one stage at a time, and each pass gets a bigger celebration. When
the build passes, one small shell goes up. When the tests pass, confetti
bursts out from behind them. When the deploy passes, the pipeline moves
aside for a fireworks finale with the message, confetti cannons fire from
both corners, and the pipeline kicks on every burst.

## What it is for

It shows the catalog's celebrations (`/fireworks` and `/confetti`, built
on `particles`) placed as components, at three sizes and in three styles,
inside a movie that is about something else. None of the fireworks or
confetti is drawn by the film itself. It places five instances: a spinner,
a `/single` shell, a `/burst` of confetti, the `/finale` and the
`/cannons`. Each one has its own seed, parameters and clock:

```postscript partial
/pop1 /fireworks << /Seed 4 /Params << /Style /single /Message () /Detail () /Sky false /Weight 1.8 >> >> instance
/burst2 /confetti << /Seed 2 /Params << /Style /burst /Message () /Detail () /Backdrop /none >> >> instance
```

The bursts are an **event track** the film owns. It passes the finale an
array of burst times, fires the cannons on two of the same times, and
reacts to all of them with `sincelast` on the movie's clock:

```postscript partial
/fbursts [0.6 1.1 1.5 1.85 2.15 2.4 2.6 2.8 2.95 3.1 3.25 3.4 3.5] def
/finale /fireworks << /Params << /Style /finale /Bursts fbursts ... >> >> instance
/hostbursts [ fbursts { fstart add } forall ] def
/kick hostbursts sincelast -9 mul 2.718281828 exch exp 0.05 mul def
```

## Techniques

- **Times are derived.** The stages' run lengths are one list (`/runs`).
  When each stage passes, when each celebration starts (`/At` is the
  pass time less the entry's own time to its first burst) and when the
  finale starts all follow from that list. Lengthen the tests and
  everything after them moves.
- **Placed small**: a component is fitted into its box, so the single
  shell is drawn at about a third of its size. Its `/Weight 1.8` keeps
  the stars from going thin at that scale.
- **Behind and in front**: the small shell and the test confetti are
  placed before the pipeline, so they rise and burst from behind it. The
  cannons are placed after everything else. Inside the confetti entry,
  a fifth of the pieces pass in front of its own message and the rest
  behind.
- The finale's night sky fades in (`faded` around `place`) while the
  pipeline eases down and shrinks, so the cut from a dashboard to a
  night sky reads as one move.

## Render

```sh
psanim check  gallery/huzzah/huzzah.ps
psanim sheet  gallery/huzzah/huzzah.ps --count 16 --cols 4 -o sheet.png
psanim render gallery/huzzah/huzzah.ps -o huzzah.mp4
```
