# Benchmarks

Measured on 2026-11-23 on one machine: Apple M4 (10 cores), 16 GB,
macOS, rustc 1.96.1, release build, ffmpeg 9.0. Wall-clock, one run
each; treat differences under 10% as noise. Repeat them with the commands
shown.

## The gallery, at full size

`scripts/bench_gallery.sh` (36 frames per piece, spread over a pure
movie; the first 36 in order for a stateful or persistent one):

| Piece | Mode | Size | Frames | Load ms | Median ms/frame | p95 | 1 thread fps | Threads | All-core fps |
|---|---|---|---|---|---|---|---|---|---|
| one-frame | pure | 1280x720 | 1215 | 1.0 | 3.4 | 16.7 | 205.5 | 10 | 629.4 |
| epicycles | pure | 1280x720 | 660 | 0.9 | 11.9 | 22.5 | 78.2 | 10 | 368.1 |
| afterglow | pure | 1280x720 | 597 | 1.6 | 19.3 | 22.2 | 57.1 | 10 | 280.6 |
| paper-boat | pure | 1280x720 | 999 | 1.4 | 29.2 | 59.6 | 32.0 | 10 | 146.9 |
| orrery | pure | 1080x1080 | 360 | 1.5 | 76.3 | 77.9 | 13.1 | 10 | 57.5 |
| taiji | pure | 1080x1080 | 720 | 0.8 | 55.5 | 57.3 | 18.0 | 10 | 78.4 |
| tesseract | pure | 1080x1080 | 840 | 1.0 | 20.0 | 24.9 | 48.7 | 10 | 224.7 |
| lissajous | pure | 1080x1080 | 480 | 1.0 | 68.0 | 70.2 | 14.7 | 10 | 70.1 |
| neon | pure | 1280x720 | 360 | 1.1 | 45.0 | 45.8 | 23.9 | 10 | 119.1 |
| ripple | pure | 1280x720 | 300 | 9.1 | 53.4 | 55.2 | 18.7 | 10 | 82.7 |
| letter | pure | 1280x720 | 750 | 12.7 | 97.1 | 117.4 | 12.2 | 10 | 56.7 |
| halftone | pure | 1280x720 | 480 | 1.1 | 255.7 | 268.7 | 4.2 | 10 | 21.9 |
| tidewater | persistent | 1280x720 | 576 | 6.7 | 36.6 | 73.4 | 37.9 | 1 | 37.7 |
| murmuration | stateful | 1280x720 | 780 | 8.2 | 92.4 | 95.5 | 10.9 | 1 | 11.1 |

Reading it:

- **Pure movies scale with cores**: 3 to 5 times on this 10-core machine
  (4 performance cores, 6 efficiency cores, and the figure includes each
  worker loading the program). Every pure piece renders faster than it
  plays.
- **Stateful and persistent movies use one thread.** Murmuration spends
  its time in the simulation (a 1,200-bird flock in interpreted
  PostScript, two ticks a frame, about 30 ms a tick); Tidewater in paintkit's watercolour washes.
- **What a frame costs** is mostly layers. One-frame has almost none.
  Orrery has two full-canvas ones (a blurred star field and a glow),
  about two thirds of its 76 ms.

## Whole films, encoded

`scripts/build_gallery.sh`: render every frame and encode H.264 through
ffmpeg, time from first frame to finished file.

| Piece | Frames | Wall time | Threads | Frames/s | File |
|---|---|---|---|---|---|
| one-frame | 1215 | 3.7 s | 10 | 331 | 1.1 MB |
| epicycles | 660 | 3.0 s | 10 | 225 | 3.6 MB |
| afterglow | 597 | 5.6 s | 10 | 114 | 4.6 MB |
| paper-boat | 999 | 13.4 s | 10 | 77 | 14.6 MB |
| orrery | 360 | 9.4 s | 10 | 40 | 40.9 MB |
| taiji | 720 | 10.9 s | 10 | 67 | 3.5 MB |
| tesseract | 840 | 6.5 s | 10 | 131 | 8.8 MB |
| lissajous | 480 | 9.0 s | 10 | 55 | 8.5 MB |
| neon | 360 | 3.3 s | 10 | 111 | 0.3 MB |
| ripple | 300 | 4.8 s | 10 | 65 | 9.3 MB |
| letter | 750 | 13.3 s | 10 | 57 | 1.2 MB |
| halftone | 480 | 23.5 s | 10 | 21 | 39.8 MB |
| tidewater | 576 | 21.7 s | 1 | 27 | 0.8 MB |
| chalk-proof | 1110 | 57.9 s | 1 | 19 | 0.9 MB |
| murmuration | 780 | 91.9 s | 1 | 8.5 | 12.8 MB |

A 40-second 720p film in 13 seconds; a 12-second 1080-square loop of
3,300 marks a frame in under 10.

## The cost of the model

Measured while choosing the design (a small scene, 1280x720, one thread):

| What | Cost |
|---|---|
| Creating an interpreter | 1.4 ms |
| Loading artkit / paintkit into it | 4 ms / 10 ms |
| A frame with no rollback (erase, draw) | 1.2 ms |
| The same frame with `restore` / `save` around it | 0.9 ms |
| Copying a 1280x720 canvas | 0.26 ms |
| PNG-encoding it | 7 ms |

Rolling the interpreter back before every frame is free at this
resolution of measurement, which is what made "purity by rollback" the
design rather than an option. Workers that each load the program pay
milliseconds.

## PSCat's interpreter, before and after the host API

`cargo bench --bench perf` in the PSCat branch (best of three; four runs
each side):

| Workload | Before | After |
|---|---|---|
| fib 27 (procedure calls) | 88 to 90 ms | 89 to 90 ms |
| defloop 200k (dictionary writes) | 13 ms | 12 to 13 ms |
| fern (48k small fills) | 154 to 156 ms | 153 to 155 ms |

The only change on the interpreter's hot path is keeping the
last-popped procedure body for the error backtrace instead of dropping
it.

## The browser

Chromium, one thread, `scripts/wasm_smoke.mjs` and the playground:

| Movie | Size | Per frame |
|---|---|---|
| examples/hello.ps | 640x360 | 0.9 to 1.1 ms |
| the playground's pointer example (300 strokes) | 640x360 | 2.5 ms |
| gallery/epicycles | 640x360 | 14 ms |

## Particles

`particles` at 1280x720 on one core, `psanim bench` (median ms a frame,
best of three runs), measured on 2026-11-25 on the same M4 while other
jobs were running (load average about 15), so treat these as upper
bounds. A continuous emitter keeps about n particles alive (`/Rate n/4
/Life 4 /From -4`) with gravity and drag; "gust" adds a `/Gust`, which
integrates each particle through 1/60 s cells from its birth.

| Particles | Procedure | Median ms/frame | All-core fps |
|---|---|---|---|
| 1,000 | empty `{ }` (the physics and the loop) | 0.4 | 1466 |
| 1,000 | fade and a 2-unit disc (`setalpha`, `arc fill`) | 2.0 | 1247 |
| 1,000 | the same, with `/Gust` | 2.7 | 1196 |
| 10,000 | empty `{ }` | 2.3 | 1498 |
| 10,000 | fade and a 2-unit disc | 17.1 | 271 |
| 10,000 | the same, with `/Gust` | 19.8 | 243 |

Reading it: the native half (birth, flight, the gust cells) and the
loop together cost about 0.2 microseconds a particle; what costs is the PostScript procedure run
for each one, about 1.5 to 2 microseconds for a fade and a disc. Ten
thousand particles a frame render faster than real time on one core.
A `glowing` around the whole call adds one layer (a few ms), not one per
particle.

## What these numbers are not

- Not a comparison with other animation systems: none was run.
- Not GPU numbers: there is no GPU path.
- One machine, one run. p95 above the median in a reel (paper-boat,
  one-frame) is real: scenes differ in cost.

## An optimisation the gallery paid for

Blur and glow first processed the whole canvas whatever was drawn. An
agent's report measured a mostly empty blurred layer at 22 to 34 ms.
They now process only the box around what was drawn, grown by the
kernel's reach (bit-identical output, checked by a test). Afterglow went
from 41 ms to 19 ms a frame.

## PSCat's libraries, per frame

What one typical use of each library costs when it is redrawn on every
frame of a pure movie, at 1280x720 on one core (`psanim bench` on a
one-line probe movie for each; the same M4):

| Library | The probe | Median ms/frame |
|---|---|---|
| dataviz | an 8-bar chart | 0.3 |
| fonts/neon | a 9-letter sign at 150 units | 1.5 |
| mathkit | two formulas at 70 units, stroked | 1.8 |
| artkit | a depth-6 L-system plant, stroked | 2.3 |
| hatchkit | a hatched disc, radius 300 | 3.5 |
| artkit | a 32x18 truchet `grid` | 6.3 |
| graph | a 40x40 `plotsurface` wireframe | 7.5 |
| qr | encode and draw a short URL | 12.7 |
| typewriter | 170 characters with the ink model | 25 |
| halftonekit | one full-canvas dot screen with a tone procedure | 69 |
| stipplekit | the full canvas stippled to a density gradient | 539 |
| drychalk | a whole board (Chalk Proof) | about 60,000 in all |

Reading it:

- **Vector libraries are cheap.** Fonts, formulas, plots, charts,
  L-systems, tilings and hatching cost a few milliseconds; a pure movie
  can redraw them every frame and still render faster than it plays.
- **Per-cell and per-character libraries are affordable.** Halftone
  screens and the typewriter's ink cost tens of milliseconds per layer
  or paragraph: fine on all cores (Halftone, five screens a frame,
  renders at 21 frames a second), not real time on one.
- **The texture libraries are too slow to redraw.** Stippling at this
  density is half a second a frame, and the dry media (chalk, crayon,
  charcoal) and paintkit are thousands of small fills a stroke. Use
  them the way Chalk Proof and Tidewater do: a persistent canvas, each
  stroke drawn once, so the cost of the picture is paid once over the
  whole film instead of once a frame. Anything computed that does not
  change (a QR code, a scatter, an L-system string) belongs in setup.

