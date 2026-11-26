# Lobby Signage

A 50 second loop (1280x720, 30 fps) of a lobby screen cycling through five
scenes: a welcome card, today's events, a big clock with a countdown to the
next one, a weather card and a status board. Invented content, a made-up
building (Larkfield Works). It loops without a seam and is meant to run
for ever; a sixth scene, a lunch menu, is on the list only on weekdays from
11:30 to 14:00, so it is missing from this render (which starts at 10:02 on
a Tuesday) and appears in the one that starts at lunchtime.

## What it is for

It is the demonstration of the playlist words (`playlist`,
`playlistduration`, `weektime`; chapter `skills/psanim/references/signage.md`).
The movie file is eighteen lines: it places a playlist. What plays is in
`lobby.playlist`, an array of `[hold scene << options >>]` and
`[seconds /transition]` that can be edited without touching the movie. The
scenes are movies of their own (`welcome.ps`, `events.ps`, `clock.ps`,
`weather.ps`, `status.ps`, `lunch.ps`), each of which also runs alone
(`psanim sheet gallery/signage/clock.ps`), and the status board places
the catalog's `/spinner` in a row, so a component sits inside a component
inside a playlist.

## Techniques

- **A playlist is a component.** `/lobby (lobby.playlist) << ... >> playlist`
  in setup, `/lobby << >> place` in the scene, and `/Duration /lobby
  playlistduration /Loop true` for the movie.
- **A seamless loop.** Each scene's clock has been running since its
  hand-over began, so the wrap from the status board back to the welcome
  card is as smooth as any other (`psanim check` reports the join).
- **Live tiles.** The clock, the digits, the countdown to 11:30, the
  greeting ("Good morning" until noon), the NOW and NEXT marks on the
  events and the weekday all read `weektime`, which is `/Day` and `/Time`
  plus the playlist's clock: no scene has a clock of its own. The weather
  card's temperature is `/outdoor signal` (14 when there is none).
- **A schedule.** The lunch scene says `/Days /weekdays /From (11:30) /To
  (14:00)`. The set of scenes is chosen as each round starts.
- **Portrait and landscape.** Scenes are written against `width`, `height`
  and `vmin` with `/Fill true`, so `--param Portrait=1` gives the same loop
  on a 720x1280 screen.
- **A safe area.** `/Safe 0.05` keeps 5% of each edge clear for a TV that
  crops; the grounds and the transitions run to the edge. Add `/Guides
  true` to the playlist's options to draw it.
- **Text for a room.** Sizes are `vmin` of the safe area (0.9 of the screen's).
  Rows and times are 4.6 to 5.4 vmin and headings 8 vmin and more, which the
  chapter's rule reads as right for a viewing distance of about 9 feet on a
  55 inch screen; the labels (card captions, rooms, NOW and NEXT, 3.3 to 3.8
  vmin) are fine print, for people who walk up to read them.

The film is a recording of one moment (10:02, Tuesday): a real display
would pass the real time, and a live page would set a `weektime` signal
(`/Clock /signal`) so the clock keeps going after the loop wraps.

`psanim bench` on a laptop, 1280x720: median 3 to 4 ms a frame, 95th
percentile 11 to 23 ms and slowest sampled frame 20 to 31 ms over two runs
(the machine was not quiet; the slow frames were not matched to the
transitions).

## Render

```sh
psanim check  gallery/signage/signage.ps
psanim sheet  gallery/signage/signage.ps --seam 6 -o seam.png
psanim render gallery/signage/signage.ps -o signage.mp4
psanim render gallery/signage/signage.ps --param Portrait=1 -o portrait.mp4
psanim render gallery/signage/signage.ps --param Day=wed --param Time=11:59:30 -o lunch.mp4
psanim render gallery/signage/signage.ps --signals outdoor.csv -o live.mp4   # outdoor: a column of temperatures
```
