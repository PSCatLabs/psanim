# The Builds Passed

A 16.9 second film (1280x720, 30 fps) in one continuous story: someone types
`cargo test` at an old green terminal, a grid of test dots fills in beside it
while the pipeline lights up a stage at a time, the screen clears, the tube
shuts to a line, and the dashboard drops to the bottom of a night sky where
"Huzzah! The builds passed" bursts in with fireworks and confetti.

`--param result=fail` is the same film with a different ending: test
`net::retry_limit` fails (a red dot, a red test stage, a red flash), the
deploy stage is skipped, there is no celebration, the screen fades, and the
same dashboard settles to the bottom of an empty frame under "The builds did
not pass".

```sh
psanim render gallery/builds-passed/builds-passed.ps -o builds-passed.mp4
psanim render gallery/builds-passed/builds-passed.ps --param result=fail -o builds-failed.mp4
```

## What it is for

It is the worked example of the component API: the host is about a hundred
lines, most of them the terminal's script and the picture, and it draws
nothing but a red flash and the failure text.
Everything else is a catalog entry (`/terminal`, `/testdots`, `/pipeline`,
`/fireworks`, `/confetti`), made once with `instance` and placed with
`place`. See the section "A worked example" in
`skills/psanim/references/components.md`.

- **Instance**: each part is `/name /entry << /Params << ... >> >> instance`,
  with its own seed and parameters (`/Fails`, `/Fail`, `/Times`, `/Bursts`,
  `/Message`).
- **Place**: `/name << /At t /Box [x y w h] /Before /hold /After /hold >> place`
  puts it in time and space. The terminal is placed at its 800 x 600 box,
  the dots and the pipeline in boxes that the host animates from the
  dashboard to the stage (`u pbox pend between`: boxes are plain arrays).
- **Time comes from one place.** The terminal's session is a plan the host
  makes (`sessionplan`) and gives to the terminal as `/Plan`, so the host can
  ask it for marks (`session /built planmark`). The dots start at the mark
  where the tests begin and run as long as the test lines do
  (`/Seconds ta tr sub`); the pipeline's stage boundaries are marks
  (`/Times [ ty tb tt ... ]`). Slow the typing or add a line and everything
  after it moves.
- **`componentduration`** sets the film's length: the finale starts half a
  second after the screen clears, and the movie lasts as long as the finale
  does (`fstart /cheer componentduration add`). Instances are made before the
  declaration so that it can read this.
- **Reacting to events**: the finale's burst times are an array the host
  passes as `/Bursts` and also keeps (`hostbursts`, shifted to the movie's
  clock). `hostbursts sincelast` is how long ago the last burst was, so the
  dashboard kicks a little bigger on every burst and settles. In the failing
  film the one event is the moment the result prints (`[tt] sincelast`), and
  it makes the red flash.
- **One host, two endings.** The whole difference is `result`, read once
  with `param`; it picks the session text, which dot fails, which stage
  fails, and whether the finale is placed.

## Cost

Measured with `psanim bench` at full size: median 33.6 ms a frame for the
passing film (p95 58.6 ms in the fireworks), 34.8 ms for the failing one,
mostly the terminal's glow and the particles.

## Render

```sh
psanim check  gallery/builds-passed/builds-passed.ps
psanim sheet  gallery/builds-passed/builds-passed.ps --count 16 --cols 4 -o sheet.png
psanim sheet  gallery/builds-passed/builds-passed.ps --param result=fail --count 16 --cols 4 -o fail.png
```
