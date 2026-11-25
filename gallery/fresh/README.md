# Fresh

A 34 second kinetic typography short for a vertical feed (1080x1920, 30 fps,
no audio). The copy is a small argument about stateless animation, with a
turn at the end:

> A frame is a function of time. Nothing is remembered. No yesterday. No
> momentum. No drift. Every moment is ~~stored~~ computed, fresh. Thirty
> times a second: what time is it? So where does the motion live? Not here.
> In you. You remember. So it moves.

Everything lands on a 120 bpm grid. The beat is defined once (`/B`), all
times are written in beats (`2 b`), and the storyboard is nine named shots
whose lengths are whole bars or half bars.

## Techniques

- **Beat grid**: `/b { B mul } def`, a `storyboard` in beats, `inbeat` per shot.
  A four-square metronome and bar counter in the footer, a live `t =` readout
  in the header, and a faint ground flash on every bar line make the rhythm
  visible without sound.
- **Scale contrast**: `capset`, a local helper that sizes the display face so
  a word fills a box by width or cap height. TIME runs up the full height of
  the frame beside a small stacked lead-in; NO and a two-digit frame counter
  fill the width.
- **Motions with meaning** (pkmotion): YESTERDAY slides in and greys out,
  MOMENTUM overshoots and stops, DRIFT never settles, NO and FRESH are
  re-popped on the beat by `looped`, REMEMBERED blurs away glyph by glyph in
  random order (`rank` + `/focus`) while NOTHING drains to an outline that
  then undraws (`trimpath`).
- **Struck out and replaced**: pkdiagram's `struck` drawn on with `trimpath`,
  then `swapped` with `/mask-up` for STORED -> COMPUTED.
- **Counter and typed line**: the big digits are the real frame number within
  the second; thirty ticks fill beneath it; `typewrite` asks the question and
  the answer is `globaltime`.
- **Type on a path**: `eachonpath` around a slowly turning ring, per-glyph
  fade-in, with `?` flipping to NOT HERE. in the middle.
- **Colour as punctuation**: one accent. It floods the frame for IN YOU (the
  hud is redrawn in the ground colour, clipped to the flood) and drains off
  upward for the last shot.
- **Trail**: MOVES. sways under `echoed`.
- **A line that tracks out**: the closing tag widens its letterspacing with
  `eachspaced` / `spacedwidth`.
- **Aspect independence**: all places are boxes cut from `canvasbox` with a
  7 vmin margin; `/tall` switches two shots between a stacked and a
  side-by-side arrangement. Size is a parameter.

## Commands

```sh
psanim check  fresh.ps --all
psanim sheet  fresh.ps --count 24 --cols 8 -o sheet.png
psanim render fresh.ps -o fresh.mp4                              # 1080x1920
psanim render fresh.ps --param w=1080 --param h=1080 -o square.mp4
psanim render fresh.ps --param w=1280 --param h=720  -o wide.mp4
```

`check --all` reports five abrupt changes; they are the intended hard cuts
on bar lines and the two fast flood wipes.
