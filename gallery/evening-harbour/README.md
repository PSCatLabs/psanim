# Evening Harbour

A plein-air sketch in gouache, 42 seconds, 1280x720, 30 fps.

A toned board is painted as the light goes: the sky and the sea blocked
in thin, then the sky laid in loaded broad bands, rose and gold toward
the horizon; a low sun soaks in pass over pass with its haze; the far
shore is washed in by a front moving along the horizon; the water is
brushed across in broken strokes, darker toward us, and the sun's path is
broken on it in quick liner dabs. Then a moored boat (hull, mast, boom, a
sail washed up from the boom, its reflection dragged down), the posts of
a jetty, and two gulls. The painter steps back and the title comes up.

## Painting over time with `paintplan`

The film is pkink's `paintplan` and nothing else for its schedule: 56
strokes, each a dictionary with a start, a length, a seed and the
procedure that paints it whole. The scene calls `plan paintkept` and the
overlay `plan paintlive`; the plan keeps each stroke on the canvas once,
whole, on the step that finishes it, and shows the strokes in progress
in the overlay until then. No stroke is ever laid down twice, so the
translucent washes and glazes come out as they would painted in one go.

- **Three kinds of stroke.**
  - `/along` with `/Reveal`: the broad brush (`pkbroad`, loaded with two
    or three colours), the oil (`pkoil`), the liner (`pkliner`) and the
    dry brush (`pkdry`) cannot grow the way chalk does, so each is shown
    through a band that follows its path, as wide as the brush.
  - `/sweep`: the underpainting (a front moving down the board), the far
    shore (along the horizon) and the sail (up from the boom).
  - `/soak`: the sun, 16 faint washes from the centre out, and its haze,
    10 wider, fainter ones, overlapping it in time.
- **Seeds.** Each stroke's `/Seed` goes to `srand` just before its path
  and its paint, which pins the paintkit presets; the soaks' washes take
  their own `pkwash /Seed` from `rand` after that, so every pass differs
  and every frame agrees.
- **The schedule is written as a hand would work**: a cursor `t` moves on
  by each stroke's length and a small gap, so the strokes can be
  reordered or retimed without touching the others.
- **Overlay**: a ring at `painttip`, where the brush is, and the title
  (pktype `face`, pkmotion `enters`).

## Measured

On the authoring machine (macOS, arm64; load average 2 to 4 while it
ran):

- **No accumulation.** The last frame equals the same board painted in
  one go (`--param oneshot=1`, which lays every stroke with `paintall`
  on frame 0): 0 of 921,600 pixels differ at 30 fps and at `--fps` 24,
  25 and 60.
- **Nothing vanishes.** `psanim check --all` at `--fps 24` and `--fps
  60` finds no pop in any pair of frames.
- **Cost** (`psanim bench --count 1260`): median 2.6 ms a frame, mean
  13.3, p95 84, max 112: a frame pays for the strokes in progress, each
  drawn whole under its band, and for those kept on its step.
- **Render** (`NOGIF=1 scripts/build_gallery.sh evening-harbour`): 1260
  frames in 19.4 s on one thread (persistent movies render in order),
  1.4 MB MP4.

To rerun the first check (after a PSCat pin moves, say):

```sh
psanim frame gallery/evening-harbour/evening-harbour.ps --param oneshot=1 --frame 1259 -o one.png
for f in 30 24 25 60; do
  psanim frame gallery/evening-harbour/evening-harbour.ps --fps $f --at 41.999 -o last-$f.png
done
# each last-*.png must equal one.png pixel for pixel (premultiplied RGBA)
cargo test --release -p psanim --test paintplan -- --ignored   # the same at 30, 24 and 60, about 2 minutes
```

## Render

```sh
psanim check  gallery/evening-harbour/evening-harbour.ps
psanim sheet  gallery/evening-harbour/evening-harbour.ps -o sheet.png
psanim frame  gallery/evening-harbour/evening-harbour.ps --param oneshot=1 --frame 0 -o oneshot.png
psanim render gallery/evening-harbour/evening-harbour.ps -o evening-harbour.mp4
```
