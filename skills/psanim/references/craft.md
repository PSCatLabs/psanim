# Making it good

A program that runs is the start. These are the habits that separate a
demo from something worth watching.

## Timing

- **Hold.** Things need time to be read after they stop moving: a title,
  1.5 to 2.5 s; a sentence, about 0.35 s per word plus a second. Most
  first drafts are too fast.
- **Vary durations.** A quick 0.3 s snap next to a slow 2 s drift has
  rhythm; everything at 1 s has none.
- **Overlap.** Start the next thing before the last one finishes
  (`staggered` with a gap shorter than each item's own duration). Motion
  that takes strict turns looks mechanical.
- **Beginnings and ends.** Open from the background (a fade or a draw-on)
  and close deliberately. A movie that just stops looks broken.
- A beat of stillness before a change makes the change land.

## Easing

- Things entering: `-out` curves (fast, then settle): `/cubic-out`,
  `/back-out` for a little life, `/spring` for more.
- Things leaving: `-in` curves (`/cubic-in`).
- Things travelling on screen: `-inout`.
- `/linear` is for rotation, scrolling and clocks: things that are
  mechanical on purpose.
- `/elastic-out` and `/bounce-out` are strong flavours: once per piece.

## Composition over time

- One thing moves at a time, or everything moves as one. Three unrelated
  motions split attention.
- Keep a margin: nothing important within 5% of the frame edge.
- Establish a layout and keep it; move the camera or the content, not
  both, unless the piece is about that.
- Depth is cheap and worth it: a slow far layer, a faster near layer,
  the far one dimmer and `blurred`.

## Colour and light

- Choose a palette of four or five colours before writing the scene, as
  named arrays in setup, and use only those. One accent.
- Dark grounds are not pure black: `[0.04 0.05 0.08]` leaves room for
  shadow and for `vignette`.
- `glowing` on the brightest elements only. If everything glows, nothing
  does.
- Thin light lines on dark read well at 2 units or more at 1280 wide.

## Type

- Two faces at most; one weight for emphasis.
- Sizes on a 1280x720 canvas: captions 24 to 30, body 36 to 48, titles 72
  and up. Smaller than 22 will not survive video compression.
- Letterspace capitals a little (draw with `eachglyph` and add a few
  units per letter).
- Never animate every property of text at once. Position *or* opacity
  *or* a draw-on.

## How to review motion without watching

You judge a movie from stills, so be systematic:

1. `psanim check`: read the motion column. Long runs of ~0 are dead air;
   a spike is a pop.
2. `psanim sheet`: is every frame a composed picture? Would any single
   frame work as a poster?
3. For each transition or entrance: `psanim sheet --from T0 --to T1
   --count 12`. Consecutive stills should differ by even steps that
   shrink toward the end of an `-out` ease. A big jump between two
   neighbours is a pop.
4. One full-size `psanim frame` at the busiest moment: check small type,
   line weights, edges of the frame.
5. First frame, last frame, and for a loop, last against first.

Then change one thing and look again. Three rounds of this beats one long
first draft.
