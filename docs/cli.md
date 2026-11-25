# The `psanim` command

```
psanim <command> <program.ps> [options]
```

| Command | Does |
|---|---|
| `info` | Prints what the program declares: canvas, output size, duration, frames, mode, seed, parameters. |
| `check` | Loads the program and renders a sample of frames small. Reports errors, blank frames, no motion, abrupt changes, a loop that does not close, strings written by a frame, and timings. |
| `frame` | Renders one frame to a PNG. |
| `sheet` | Renders several frames onto one labelled contact-sheet PNG. |
| `render` | Renders the movie to `.mp4`, `.webm`, `.gif`, or a directory of PNGs. |
| `bench` | Times loading and rendering, serial and on all cores. |
| `api` | Prints the vocabulary; `psanim api tween` explains one word; `--markdown`, `--json`. |

Every command takes `--json` for a machine-readable result on stdout.
Errors are JSON too in that mode, with `"ok": false`.

## Selecting frames

| Option | Meaning | Commands |
|---|---|---|
| `--at T[,T...]` | times in seconds | frame, sheet |
| `--frame K[,K...]` | frame indices | frame, sheet |
| `--count N` | N frames spread evenly (12 for sheet, 16 for check) | sheet, check |
| `--from T --to T` | a time range | sheet, render, check |
| `--seam N` | the last N frames, then the first N | sheet |
| `--all` | every frame | check |

## Size and timing

`--scale F` (pixels per canvas unit), `--width PX`, `--height PX`,
`--fps N`, `--duration S`, `--seed N`, `--motion-blur N` (0 turns
declared blur off), `--shutter F`.

`sheet` and `check` default to small frames (320 px wide) because they
are previews; `frame`, `render` and `bench` default to scale 1.

## Inputs

`--param NAME=VALUE` (repeatable) for the program's `param`;
`--signals FILE.csv` (repeatable) for `signal`, `sinceevent`,
`eventcount`. The CSV's first column is time; other columns are signals;
a header ending in `!` is an event track.

## Output

`-o PATH`; the format follows the extension. `--crf N` sets video
quality (lower is better; default 16 for MP4, 28 for WebM). `-j N`
threads (default one per core; stateful and persistent movies use one).
`-q` quiet. `--cols N`, `--thumb PX` for sheets.

Video needs `ffmpeg` on PATH. Formats exercised: MP4 (H.264, yuv420p),
WebM (VP9, yuv420p), GIF, PNG sequence. PNG frames keep transparency;
video is flattened onto the background colour (black when the background
is transparent).

## Safety

`--untrusted` bounds steps, time and memory and confines reads to the
built-in libraries; `--max-steps`, `--max-seconds`, `--max-memory-mb`
set the bounds one by one; `--read-root DIR` allows reads under DIR
(default: the program's own directory). Programs never write files. See
[security](security.md).

## Exit status

0 success; 1 the program has an error; 2 bad usage; 3 output failed
(cannot write, ffmpeg missing or failed).

## `check`, in detail

```
$ psanim check movie.ps
Epicycles: 1280x720 units, 22s at 30 fps = 660 frames, pure
  loaded in 12.1 ms; 16 frames sampled at 320x180
  frame    time    ms   luma  cover  motion   (motion: difference from the next frame)
      0   0.00s  11.8  0.047  0.134  0.0006
     73   2.43s   1.7  0.059  0.206  0.0046
  ...
  ok: no problems found
```

- **luma**: mean brightness, 0 to 1.
- **cover**: the fraction of pixels that differ from the frame's most
  common colour. Near 0 is a blank or flat frame (a frame filled with one
  colour has cover 0, whatever the colour).
- **motion**: the mean difference between this frame and the next, 0 to
  1. Runs of 0 are stillness; a value many times its neighbours is a cut
  or a pop.

Warnings: every frame blank; blank frames in the middle; a slow start or
early end; nothing moves; an abrupt change; a write into a string made in
setup; for a `/Loop true` movie, a join whose step is larger than the
steps beside it. Notes (not problems): opening from or closing to the
background; frequent small changes taken as an effect.

**Abrupt change.** Two kinds. A change that fills the frame (a cut) is a
whole-frame difference many times the typical one, seen among the sampled
frames. A local pop is found tile by tile: the preview is cut into tiles
about a twentieth of its width (16 pixels at the default 320), and a pair
of adjacent frames is flagged where one tile's change is at least 0.04 and
more than four times the same tile's change in the pairs just before and
after, by at least 0.04 more. (At the default size, a white dot of 4 pixels,
about 16 canvas units, on black just reaches it.) A pop is a spike in time, so steady fast
motion (the same change every pair) and a fade (growing) are not flagged,
and a small element appearing in a busy frame is: its own tile was calm.
The warning gives the frame, the number of tiles and the area they cover
in canvas units, and the `psanim sheet` command that shows it.

Three more rules keep intended motion out of the warnings. A spike with
another spike within three tiles in the pair before or after is motion
(a wipe, or a fast edge crossing the frame, changes each tile for one pair
only, tile after tile). Small changes (at most 8 tiles) that recur within a
tile of each other (a counter, typed letters, lights coming on one by one)
are one note naming the area and the frames, not a warning each. And more
than one pair in twenty popping is a style (sparkle, particles): one note.
A persistent movie is a painting that accumulates marks, so local changes
are its substance: one note says how many, and only cuts are warnings.
A cut, or any change over 8 tiles, is always a warning: "fine if intended".
There is no way yet to mark a cut as intended.

What is scanned. Judging a pair needs the pairs either side, so frames are
rendered in runs. In a pure movie, besides the sampled frames and the one
after each, `check` scans 8 runs of 24 consecutive frames (about 230
frames) spread evenly over the movie, in an order that is a fixed function
of the movie: the same movie gives the same findings on every machine. The
summary line says how many of the frame pairs were covered; `--all` covers
every one. If frames are so slow that the scan reaches its 8 second limit,
a note says its coverage now depends on the machine. A movie with state
(stateful or persistent) is scanned only in a run of six frames around each
sampled frame, because reaching a frame costs every tick before it.

**A string written in a frame.** After setup, `check` fingerprints every
string reachable from `userdict` (strings in the dictionaries and arrays
it holds too), and again after the sampled frames, once the interpreter
has been rolled back. PostScript's `restore` does not roll back the
contents of a string, so a string that differs was written by a scene and
leaks into every later frame. The warning names it (`buf`, `table/name`,
`rows[3]`) and gives the fix, copying the string inside the scene
(`/buf buf dup length string copy def`). Only pure movies are checked:
stateful and persistent ones keep state on purpose. A literal string in a
procedure body is not looked at.
