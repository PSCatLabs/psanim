# Build Farm

A 64 second film (1280x720, 30 fps): one day on a twenty-machine build
farm, replayed in a minute as a wall dashboard. The nightly builds light
every row at two in the morning; developers arrive at nine; `lin07` stops
answering at 11:10 and its row goes dark and its lamp red; a runaway test
pins `win02` from 13:20; a release candidate at 16:30 sends the queue,
the build times and the cache misses up together; the evening drains it.
At midnight everything but the day's totals steps back.

## The data

The film reads no file. Its numbers come from the host as signals, the
way a live dashboard's would:

    psanim render gallery/buildfarm/buildfarm.ps --signals gallery/buildfarm/day.csv -o buildfarm.mp4

`day.csv` is **synthetic**: written by [`gen_day.py`](gen_day.py) from a
fixed seed (1118) and a story of the day in a few lines of Python. Nothing
in it was measured. Its first column is movie time, the day at 1 s = 24
minutes, a row every two minutes of the day; the other columns are CPU
per builder, the queue, the day's builds and failures, the p95 build
time, the cache hit rate, the mean load, builds in the last hour, and an
event track (`incident!`). Run `python3 gallery/buildfarm/gen_day.py` to
write it again.

The `% Signals: day.csv` line in the source tells the gallery scripts
and the documentation test to pass the recording. Without it the film
says that it has no data, instead of drawing a farm of zeros
(`hassignal`).

## Techniques

- **Catalog panels, each the shape of its box.** Every panel but two is a
  catalog entry placed as a component: `/horizon` (twenty rows, a cursor,
  a readout with a lamp per row), `/stat` four times, `/gauge`,
  `/counter`, `/timeseries`. The entries take /Width and /Height, so
  `sized` makes each instance the shape of the box it goes in and
  `place` fits it without bars.
- **One clock for the data.** Signals are read on the movie's clock even
  inside a component (`signaltime`), so the panels can enter one after
  another under `cascade` and `enters` while their data stays in step.
- **A replayed day does not shimmer.** The horizon rows read the whole
  recording on a fixed grid (`signalbetween`, 360 readings) and reveal it
  behind the cursor with `/Draw`; the queue chart scrolls a five second
  window read on a grid fixed in time, one step past now, and clipped.
  Two consecutive frames compared were pixel-identical left of the cursor.
- **Values that glide.** A recording is read straight-line between its
  rows, so the numbers, the gauge and the counter move smoothly at 1440x.
  A live host gets the same by recording readings (`record_signal`) and
  giving the panels a /Lag of one update interval.
- **Ticks that do not jump.** The queue chart's y range follows the data;
  `plotaxes /Fade` cross-fades its labels between round steps.
- **Thresholds are roles.** Lamps, numbers and the cache tile's card take
  `/good`, `/warn` and `/bad` from the theme through `thresholdrole`.
- **Computed once from the host's data.** The heatmap's hourly means per
  pool are worked out in setup with `signalbetween`, then revealed column
  by column.
- **Incidents are events.** The banner and the log read the `incident!`
  track with `sinceevent` and `eventcount`.

## Cost

Measured on an Apple silicon Mac (`psanim bench`, 48 frames): a median
of about 11 ms a frame at 1280x720 on one thread; the whole film renders
in about 10 s on 10 threads.
