# Orrery

A seamless 12 second loop (1080x1080, 30 fps, 360 frames): an astronomical
instrument made of light. A planetary gear train turns at the centre with
a breathing guilloche rosette in its hub; four graduated orbit rings tilt
and precess around it like the hoops of an armillary sphere, carrying
bodies with comet trails and moons; behind them two families of string-art
chords weave an envelope bent by looping noise, over out-of-focus stars
and nebula. About 3,300 marks a frame (teeth, ticks, chords, stars, trail
segments) plus some 5,000 polyline segments in the rosette and ropes.

## How the loop closes

The movie is pure: every frame is a function of `progress` alone, with
`T = progress * 360`. Three rules make frame 359 lead into frame 0:

- **Whole turns.** Anything unique (a moon, a noise phase, the rosette's
  breathing) is a function of `sin`/`cos` of an integer multiple of `T`.
- **Symmetry lets things move slowly.** A part with n-fold symmetry only
  has to turn a multiple of 360/n. The carrier (8 planets) turns 90
  degrees, the sun gear (24 teeth) 270, each planet -270 (so its
  decoration is 4- and 12-fold, not 6-fold), the vernier ring 30.
- **Looping noise.** `gnoise3` is sampled on a circle in noise space
  (`T cos r mul`, `T sin r mul` added to two coordinates), so the chord
  envelope and the star shimmer wander and still come home.

One trap worth recording: a tilted ring turned half a turn looks the same
but has swapped its near and far sides, so the orbit planes precess by
whole turns, and the bodies' in-plane motion is chosen so the sum is a
multiple of 360/bodies.

## Techniques

- Planetary gear kinematics (ring fixed: sun = 3 x carrier, planet spin =
  -3 x carrier) with tooth phases solved so all eight planets mesh.
- Gear outlines precomputed in setup as point arrays; per-item constants
  from `hash01`.
- Tick scales drawn as one dashed circle each (`setdash`, line width =
  tick length); tilted rings by `rotate` + a squashed `scale`.
- String art: 2 x 240 translucent chords, separately stroked so the
  crossings accumulate into an envelope.
- One `blurred` back layer (nebula, stars), one `glowing` layer (rosette,
  bodies, moons), `vignette` last.

Cost: about 83 ms/frame at full size on one core (max 88). The two
full-canvas layers (`blurred`, `glowing`) are about two thirds of it;
the 480 chords are about 10 ms.

## Render

```sh
psanim check  gallery/orrery/orrery.ps
psanim sheet  gallery/orrery/orrery.ps -o out/orrery/sheet.png
psanim sheet  gallery/orrery/orrery.ps --frame 354,355,356,357,358,359,0,1,2,3,4,5 \
              --cols 6 -o out/orrery/seam.png
psanim render gallery/orrery/orrery.ps -o out/orrery/orrery.mp4
psanim render gallery/orrery/orrery.ps -o out/orrery/orrery.gif --width 480 --fps 20
```
