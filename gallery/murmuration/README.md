# Murmuration

Twelve hundred starlings over a fen at dusk: the flock flies in, folds and
breathes, is split twice by a falcon, re-forms, and funnels down to roost as
the light goes. 26 s, 1280x720, 30 fps. A **stateful** movie: `step` runs a
flocking simulation at 60 ticks a second and `scene` only draws it.

## The simulation

State is four arrays (`px py vx vy`) made in setup. Each tick every bird
sums these accelerations, integrates, and has its speed held to 140..340
units/s (birds can neither hover nor sprint):

| Rule | What it does | Parameters |
|---|---|---|
| Separation | pushed away from each bird of its own sheet inside a radius | `SEP-R` 12 (x0.5..1.3), `K-SEP` 70 |
| Alignment | turns toward the mean velocity of its neighbourhood | `K-ALI` 2.4 /s |
| Cohesion | accelerates toward the neighbourhood's centre | `K-COH` 2.2 /s^2 |
| Attractor | steers to fly at 250 u/s toward a wandering point (sum of sines; it descends to the roost from t = 20.5) | `K-LEAD` 1.3 /s, `LEAD-R` 220 |
| Wind | two samples of `gnoise3(x, y, t)`: neighbours feel the same gust | `K-WIND` 300 |
| Falcon | flee it inside a radius; it makes two straight stoops (t = 6.5, 14.5), homing for 1.5 s then flying on | `FEAR-R` 160, `K-FEAR` 2200 |
| Walls | soft push back into the sky | `K-WALL` 14 |

Two things give the density its folds:

- **Depth sheets.** Bird `i` lives in sheet `i mod 3`. Birds keep their
  distance only within a sheet but align and cohere with everyone, and each
  sheet follows the attractor 0.3 s later and reads the wind a little further
  on. The sheets slide over each other; where they overlap the flock is dark.
- **A breathing radius.** The separation radius follows a slow noise field in
  space and time, so bands of tight packing travel through the flock.

## Keeping the neighbour search cheap

No all-pairs loop. Both grids are rebuilt every tick in O(n):

- a **coarse grid** (44-unit cells) of per-cell sums: count, position,
  velocity. Alignment and cohesion read the 3x3 block around the bird: 9
  lookups, whatever the density;
- a **fine grid** (16-unit cells, one per sheet) of linked lists
  (`f-head`, `f-next`). Separation walks the 3x3 block, which holds every
  bird that can be inside the radius; only used cells are cleared afterwards.

Cost on one core: about 35 ms per tick on average for 1200 birds (25 when the
flock is loose, more when it packs), about 45 ms to draw a frame.
`--param birds=400` runs a lighter flock; `--param debug=1` shows the attractor.

## Drawing

Each bird is a body along its velocity, a beating swept wing, and a faint
streak back along `position - velocity * 0.055`. Marks are translucent and
stroked one by one, far sheet first, so overlaps darken. The sky is an
`axialsh` wash with a screen-blended `radialsh` afterglow, one blurred layer
of cloud puffs lit from below, a glowing star, and a slow blue veil as the
light drains. Trees (recursive branching), hedge line (`fbm2`), reeds and
cloud are generated once in setup with `hash01` and replayed from arrays.

## Render

```sh
psanim check  gallery/murmuration/murmuration.ps
psanim render gallery/murmuration/murmuration.ps -o out/murmuration/murmuration.mp4
```

Stateful movies render on one thread: about two and a half minutes.
