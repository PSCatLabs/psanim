# Logo

psanim's own mark, and the film that makes every version of it: a 4.4
second loop (1140x400, 30 fps, 132 frames). An amber tile has an
ease-in-out curve cut through it, with the current point at its end. The
point is scrubbed back to the start, then plays forward again and draws
the curve as it goes, leaving its last few positions behind as onion
skins. It comes to rest where the loop began.

## How it works

- **One idea, from both sides of the project.** The curve is a
  `curveto` whose handles are flat, the shape of an ease-in-out; the
  point rides it with `pathpoint`, at a position that `/cubic-inout
  ease` gives from the time. The groove behind it is the same path, cut
  with `0 u trimpath stroke`.
- **Onion skins are the same function, earlier.** Each skin is the
  point at `now` minus a few hundredths of a second. Equal steps in time
  land close together where the ease is slow and far apart where it is
  fast, so the skins draw the easing. At rest they all sit under the
  point.
- **The still is frame 0.** The loop starts and ends on the resting
  mark, so the PNGs, the favicons and the social preview are frames of
  the film, not a separate drawing.
- **One file, every form.** `--param form=` chooses `lockup` (the
  default: mark and word), `mark` (512 square), `icon` (the tile edge to
  edge, for favicons) or `social` (1280x640, with a line under the word);
  `--param ground=` chooses `dark`, `light` or `transparent`. Both are
  read before `animation`, so they set the canvas and the background.
- **Made to survive 32 pixels.** The groove is a seventh of the tile's
  width and the point nearly a third, so at favicon size the tile still
  shows an S and a dot.

The word is set in AvantGarde-Demi, whose round letters answer the
tile's corners and the point.

## Render

```sh
psanim check  gallery/logo/logo.ps
psanim sheet  gallery/logo/logo.ps --param form=mark --from 1.6 --to 4.4 --count 16 -o sheet.png
psanim render gallery/logo/logo.ps -o logo.mp4
scripts/build_logo.sh        # the GIFs, PNGs and favicons in docs/img/
```
