# Ship It

A 37.6 second night shift (1280x720, 30 fps): someone types at an old
green terminal, builds a release, watches one test go red, fixes it with a
one-line `sed`, runs the tests again and ships. A pipeline, a progress bar
and a grid of test dots keep score beside the screen, and when the deploy
finishes the banner prints, the release spinner closes into a check and a
ring of green light leaves the monitor.

## What it is for

It is the first piece made almost entirely of catalog entries. The
terminal, the scoreboard and the typing are not drawn here: this file says
what is typed and what comes back, and what happens when.

- **`/terminal`** shows the session (phosphor glow, scanlines, a bowed
  screen, grain, persistence, a warm-up from a line of light).
- **`/pipeline`**, **`/progress`**, **`/testdots`** and **`/spinner`** are
  placed beside it, each started by a *mark* in the session.

The session is one array of items, made into a plan with `sessionplan`:

```postscript partial
/session [
  [/type (./run-tests) << /Think 0.9 >>]
  [/mark /tests]
  [/lines tests1 0.11]                  % 24 lines, one every ~0.11 s
  [/mark /failed]
  [/out (1 failed, 23 passed\n  net::retry_limit: expected 3, got 4)]
  [/type (sed -i 's/< 3/<= 3/' net/retry.c) << /Speed 1.5 /Slips [1] >>]
] 5 << /Prompt (vault# ) >> sessionplan def
/tt session /tests planmark def         % when the tests started
/tf session /failed planmark def        % and when the last line was out
```

The scoreboard is timed from those marks (`/Seconds tf tt sub`,
`/Times [ tb tt tf ... ]`), so changing a typing speed or adding a line
to the output moves the dots and the pipeline with it. The terminal is
given the plan (`/Plan session`) rather than the script, so the movie and
the screen agree on every time.

## Techniques

- `typingplan` / `sessionplan` / `planmark`: the human rhythm (bursts,
  word pauses, a thinking pause, a slip to a neighbouring key that is
  noticed, backspaced and retyped: `/Slips [4]` forces one) comes from
  the seed.
- Phosphor persistence by asking for the screen at `t - lag`: `screenat`
  is a pure function of time, so the ghost of a scrolled line is a second
  call, not stored state. A pure movie throughout.
- `during` to swap the first pipeline and test grid (failed) for a second
  pair (green) when the tests are run again, and to change the progress
  bar's caption from BUILD to DEPLOY.
- `/After /hold` on every placed entry, so each keeps its last picture.

Cost, measured with `psanim bench` while the machine was busy (load
average 18 to 31, so these are upper bounds): about 42 ms a frame for the
whole piece at 1280x720; the terminal alone about 28 ms at 960x720, of
which the glow is a quarter or so (`/Glow 0` measured 20 ms); each of the
pipeline, progress bar, test dots and spinner under 1 ms.

## Render

```sh
psanim check  gallery/ship-it/ship-it.ps
psanim sheet  gallery/ship-it/ship-it.ps -o sheet.png
psanim render gallery/ship-it/ship-it.ps -o ship-it.mp4
```
