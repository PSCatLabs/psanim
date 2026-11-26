# Campfire

An 8 second seamless loop (1280x720, 30 fps): a camp at night. A tent
and a ring of stones sit under a sky of twinkling stars, a crescent moon
and a faint galaxy; one shooting star crosses once a loop; and in the
middle a fire burns, its glow flickering on the ground, the stones and
the tent, its sparks climbing into the stars.

## What it is for

It is the first piece built on the **catalog** rather than beside it. The
fire is not drawn in this file. It is the `/fire` entry, set up once and
placed in a box:

```postscript partial
/camp /fire << /Seed 11 /Params <<
  /Height 0.74 /Width 0.5 /Intensity 1.25 /Embers 2.2 /Smoke 0.35
>> >> instance

/scene { ... /camp << /Box campbox >> place ... } def
```

The entry's flames, glow, rising embers and smoke are its own. This film
decides how tall, how wide and how fierce, and paints everything the
fire lights: that is the part a ready-made element cannot know.

## Techniques

- **A whole number of loops.** The entry's period is 4 seconds and this
  film's is 8, so `place` wraps it with no jump at the seam. Every star
  twinkles a whole number of times a loop (`k 360 mul progress mul`), and
  the shooting star is a function of the clock; `check` measures the
  join against the steps beside it.
- **Stars from `hash01`.** 180 stars and 240 specks of the galaxy are
  hashed places, sizes and tints, so a frame does not depend on the one
  before it. A dim band of wide strokes under the specks is the galaxy.
- **Firelight without a layer.** The pools of warm light on the ground
  and the lit faces of the tent, stones and logs are stacked translucent
  ellipses and alpha fills, scaled by one `flare` number made of two
  `loopnoise` keys, so the whole camp breathes together and closes at the
  seam. Layers are the expensive thing in psanim (the entry's own glow is
  the only one in the scene).
- **A crescent by clipping.** The moon is a pale disc, clipped, with a
  sky-coloured disc offset inside it.
- **The land** is two skylines of summed sines and a few dozen
  three-tier pines from `hash01`, all flat dark shapes: what carries the
  depth is the light falling on them.

## Cost

About 40 ms a frame at 1280x720 on one core (measured with `psanim
bench`; noisy, as other work shared the machine): roughly 19 ms of that
is the fire (its glow layer covers a large box), about 7 ms the sky and
stars, the rest the land and light. 240 frames render in 2.7 s on 10
threads.

## Render

```sh
psanim check  gallery/campfire/campfire.ps
psanim sheet  gallery/campfire/campfire.ps -o out/campfire/sheet.png
psanim render gallery/campfire/campfire.ps -o out/campfire/campfire.mp4
```
