# Signage: a screen that cycles through scenes

A playlist is a list of scenes, each held a few seconds, joined by the
reel's transitions, that plays for ever: the last scene hands over to the
first. It is a [component](components.md), so a playlist is placed like
one, scenes are components (or catalog entries, or procedures), and the
playlist can sit in a box of a larger movie. The list can live in a file
beside the movie, so the content changes without editing it.

```postscript
%!PS
/hello { 1 1 1 setrgbcolor /Helvetica-Bold findfont 60 scalefont setfont
         width 2 div height 2 div moveto (Welcome) showcenter } def
/hours { 0.9 0.6 0.2 setrgbcolor 0 0 width height 6 div rectfill
         1 1 1 setrgbcolor /Helvetica findfont 30 scalefont setfont
         40 height 12 div moveto (Open 9 to 5) show } def

/show [ [4 /hello]  [1 /dissolve]
        [4 /hours]  [1 /wipe-left]
        [4 /spinner << /Params << /Style /dots >> >>] [1 /iris] ] << >> playlist

<< /Size [800 450] /Duration /show playlistduration /Loop true >> animation
/scene { /show << >> place } def
```

`/name list opts playlist` goes in setup, before `animation` if the movie
takes its length from `playlistduration`, as here. `place` draws it
(`/Box`, `/At`, `/Speed` as for any component). With no schedule the loop
has a fixed length, the holds and the transitions added up, so a movie
with `/Duration /show playlistduration /Loop true` repeats without a seam:
`psanim check` measures the join, and `psanim sheet --seam 6` shows it.

## The list

```
[ [8 /welcome] [1 /dissolve] [10 (events.ps) << /Params << /Title (Today) >> >>] [1 /push-left] ]
```

- `[hold scene]` or `[hold scene << options >>]`: a scene, `hold` seconds
  fully on screen.
- `[seconds /transition]`: how the scene before it hands over to the next.
  Any name in `transitions`, the reel's own, and your own (put it there, or name it `t-...`, before the playlist plays). The
  transition is extra time, not part of the hold, so a round takes the
  sum of both for every scene. `[0 /cut]`, or nothing written and
  `/Transition [0 /cut]` in the options, is a plain cut. A transition
  written after the last scene is the one from the last scene back to the
  first; without one that hand-over uses `/Transition` (default
  `[1 /dissolve]`).
- A scene is one of: a **catalog entry** (`/spinner`), a **component**
  already defined, a **file** name (`(events.ps)`, read beside the movie
  like a component file), an inline **component dictionary**, or a
  **procedure** (a name, or `{ }`) that draws for `width` by `height`.
  A name that is a component is used as it is; a catalog entry, or any
  component given `/Params`, `/Seed` or `/Fill`, gets an instance of its
  own, set up once.

Scene options, all optional:

| Key | Meaning |
|---|---|
| /Params, /Seed | for the instance, as in `instance` |
| /Fill true | lay the component out for the whole box, instead of fitting its declared /Size into it. Read `width` and `height` in the scene, not in setup |
| /Background | this scene's ground, `[r g b]` or a procedure (default the playlist's) |
| /Days, /From, /To | when it is on the list: see Schedules |
| /Orientation | `/landscape` or `/portrait`: on the list only for a screen of that shape |

Playlist options: `/Size [w h]` (the screen; default the movie's),
`/Safe 0.05`, `/Background` (a colour or a procedure; the ground every
scene is drawn over), `/Transition`, `/Guides true`, `/Day`, `/Time`,
`/Clock`, `/Duration`. `psanim api playlist` has the details.

Every scene is drawn over its own ground, full bleed, so wipes, pushes
and irises work as they do in a reel; a scene that declares its own
`/Background` (a component does) paints it only inside its box, so give
the playlist the ground and let scenes draw over it.

How the hand-over works: each frame the playlist builds the reel for the
round it is in and plays it with `reel`, so the transitions are the
reel's. The seam is arranged the way a loop of a reel would be: the
first scene's clock has been running since the last hand-over began, so
at the wrap both copies of it are at the same moment of their animation.
Each scene has its own clock from the moment its hand-over starts; a
component inside sees `now` run from there.

## A playlist in a file

A playlist file holds one PostScript array and nothing else. It is read
from the movie's own directory, inside the read roots, like a component
file, and the scenes it names are read the same way.

```postscript partial
/lobby (lobby.playlist) << /Safe 0.05 >> playlist
```

```
% lobby.playlist: what plays, in order. Edit this; the movie does not change.
[
  [8.8 (welcome.ps) << /Fill true /Params << /Place (Larkfield Works) >> >>]
  [1.2 /wipe-left]
  [8.8 (events.ps)  << /Fill true >>]
  [1.2 /dissolve]
  [8.8 (lunch.ps)   << /Fill true /Days /weekdays /From (11:30) /To (14:00) >>]
  [1.2 /dissolve]
]
```

PostScript data was chosen over a line-oriented list because it needs no
parser of its own: numbers, names, strings, arrays and dictionaries mean
what they mean everywhere else, a scene's `/Params` can hold any of them,
comments are `%`, and an error comes with the engine's usual message. It
is read with `run`, so it is as trusted as the movie, and the same read
roots apply (`psanim check --untrusted` on a movie that reads one fails on
the read with `undefinedfilename`, as for a component file). It must
leave exactly one array.

The name of the file can itself be a parameter:
`/List (lobby.playlist) param`, then `--param List=cafe.playlist` shows a
different screen with the same movie.

## Schedules: which scenes, when

A scene with `/Days`, `/From` or `/To` is on the list only then.

- `/Days`: `/mon /tue /wed /thu /fri /sat /sun`, `/weekdays`, `/weekend`,
  `/daily`, a number 0 (Monday) to 6, or an array of them. It is the day
  of the moment, so a window that crosses midnight belongs to both days.
- `/From`, `/To`: a time of day, as hours (`9.5`) or a string (`(9:30)`,
  `(9:30:15)`). `/From (22:00) /To (6:00)` crosses midnight.

The playlist plays in **rounds**, one pass through the scenes that are
on the list, and the set is chosen when a round starts, so a scene is
never cut off halfway and the hand-over from one round to the next goes
to the next round's first scene. The set of scenes may therefore change
between rounds, and a list of nothing at that time is just the ground
until something is.

What time is it? The playlist's own time 0 is `/Day` (default `/mon`)
and `/Time` (default midnight), which the movie takes from `param`:

```postscript
%!PS
/Day /tue param /day exch def
/Time (11:29:45) param /time exch def               % hours or (H:MM[:SS])

/morning { 0.9 0.6 0.2 setrgbcolor 0 0 width height rectfill
           0 setgray /Helvetica-Bold findfont 40 scalefont setfont
           40 height 2 div moveto (Good morning) show } def
/lunch   { 0.2 0.6 0.4 setrgbcolor 0 0 width height rectfill
           0 setgray /Helvetica-Bold findfont 40 scalefont setfont
           40 height 2 div moveto (Lunch is served) show } def

/show [ [3 /morning] [1 /dissolve]
        [3 /lunch << /Days /weekdays /From (11:30) /To (14:00) >>] [1 /dissolve] ]
      << /Day day /Time time >> playlist
<< /Size [640 360] /Duration 30 /FPS 10 >> animation
/scene { /show << >> place } def
```

`psanim render movie.ps --param Time=11:29:45 --param Day=tue` starts at
that moment, and the movie's own clock then moves the time on. It is
never the wall clock: the same movie renders the same frames tomorrow.

For the movie's length, `/name playlistduration` is the length of the
first round, which is the loop when there is no schedule. A scheduled
playlist's own loop is a week (`componentduration`; `/Duration` in the
options changes it), because the sets repeat weekly; at the week's wrap
the rounds start again from `/Day` and `/Time`, so there can be a cut
there. For a short film of a scheduled playlist, give the movie
`/Duration` of the round (or `--duration`) and start it at a moment when
the set does not change during the film.

## Live tiles inside a scene

A scene is a component, so it can read anything a movie can. For the time:
**`weektime`** is the time the schedule is at, in seconds since Monday
00:00. Inside a playlist it is `/Day` and `/Time` plus the playlist's
clock, so it moves on during the loop and a scene needs no clock of its
own. `weektime 86400 fmod` is the time of day; `weektime 3600 div`, hours.

```postscript
%!PS
/clock << /Size [400 200] /Duration 5
  /Scene {
    /Helvetica-Bold findfont 90 scalefont setfont 1 1 1 setrgbcolor
    weektime 86400 fmod cvi /s exch def
    20 70 moveto
    s 3600 idiv 2 zeropad show (:) show s 3600 mod 60 idiv 2 zeropad show (:) show
    s 60 mod 2 zeropad show
  } >> component

/show [ [5 /clock << /Fill true >>] [1 /dissolve] ] << /Time (9:41:50) /Background [0.1 0.1 0.3] >> playlist
<< /Size [640 360] /Duration /show playlistduration /Loop true >> animation
/scene { /show << >> place } def
```

For a value from outside, read a signal: `/outdoor signal` in a weather
scene is the number the host last gave for `outdoor`, and the film
renders from a `--signals` file when there is no live host. A scene
should have a default for a signal that is 0 because nobody sent it (the
gallery's weather card shows 14).

A playlist inside a playlist, a scene placing other components
(`/busy << /Box b >> place` for a spinner in a status row), a scene with
its own `/Setup`: all of it is components, so all of it works.

## Portrait, landscape, safe areas

- `/Size [720 1280]` makes a portrait screen. Write scenes against
  `width`, `height` and `vmin`, and give them `/Fill true`, so they lay
  out for the screen they are on: `aspect 1 lt` is portrait. For a list
  shared by screens of both shapes, `/Orientation /portrait` or
  `/landscape` on a scene keeps it to its shape.
- `/Safe 0.05` keeps a fraction of every edge clear, for a TV that
  crops (overscan). Scenes are placed inside it; the ground and the
  transitions still run to the edge, so the picture is full bleed and
  only the content keeps clear. 5% each side is the usual title-safe
  margin. `/Guides true` draws the safe area as a dashed rectangle to
  check a layout, and is left out of the final render.

### Text for a room

A rule of thumb from signage practice: one inch of letter height for every
ten feet of viewing distance. In a size that does not depend on the
screen: `vmin` is a percentage of the screen's shorter side, and the
type size in `vmin` is about

    size  =  14 x (viewing distance in feet) / (shorter side in inches)

so a 55 inch screen (27 inches on its short side) seen from 10 feet
wants type of about 5 vmin, which is 36 canvas units on a 720-high
movie. That makes, for a screen across a room of about 10 feet:

- body copy, rows, times, the words people read in one glance: **5 vmin
  or more**; headings 8 vmin or more;
- anything under 3 vmin (the gallery's small labels) is fine print:
  captions people walk up to read.
- a line longer than about 25 characters at 5 vmin does not fit a
  portrait screen's width, so keep portrait lines short.
- `vmin` inside a playlist with `/Safe s` is of the safe area, not the
  screen: the screen-relative size is (1 - 2s) of the number written, so
  with `/Safe 0.05` write 5.5 where 5 is wanted.

`psanim check` does not warn about type size; look at a `psanim frame`
and squint, or view it from a distance: the rule is a starting point, not a
measurement, and room, contrast and font weight move it.

## Running for ever

- **A long render.** A playlist with no schedule is a seamless loop of
  `playlistduration` seconds: render one loop (`psanim render lobby.ps -o
  lobby.mp4`), and let the player or the TV loop the file. For a long
  stretch of a scheduled playlist, render with `--duration` and start it
  where you want with `--param Time=...`. The frames are pure functions of
  time, so a render can be split with `--from` and `--to` and joined.
- **The browser player.** `<psanim-player src="lobby.ps" autoplay loop>`
  loops a `/Loop true` movie for ever: the player wraps its time at the
  movie's duration (read in `web/player.js`; not run on a playlist in a
  browser for this chapter). The browser build has no files and no
  `--param`: write the list inline (`[ ... ] playlist`, component
  dictionaries, procedures), and have the page put the start time into
  the program text it hands to the engine. Two cases:
  - *No schedule.* `/Duration /name playlistduration /Loop true` repeats
    every loop; nothing more is needed. A clock tile then shows the
    start time again after each wrap.
  - *A schedule, or a clock that must be right.* The wrap must not restart
    the rounds, so make the movie a week long: `/Day /mon /Time 0` and
    `/Duration /name componentduration /Loop true`, and have the page seek
    to the present when it starts (`player.seek(secondsSinceMondayMidnight)`:
    the element has a `seek(time)` method). The rounds are then found from
    Monday 00:00 whenever the page opened, and the loop wraps at the week's
    end, once, with a cut. For the tiles give the playlist `/Clock
    /signal` and have a page that drives the engine itself set the
    `weektime` signal before each frame (`engine.setSignal("weektime",
    secondsSinceMonday)`); the `<psanim-player>` element sets only the
    mouse signals.
- **What does not loop.** Anything that reads the real time of day (a
  clock) shows the same minute each time the file loops: a rendered
  loop is a recording of one moment. Only a live host moves the time on.

## Limits

- At most 60 scenes in a playlist.
- A scene's setup runs when `playlist` is called, so define everything it
  needs first: procedures it names, components.
- The clock restarts for a scene each time its hand-over begins, so a
  scene that is on the list twice in a round starts twice.
- A scheduled playlist is a week long and its first round is what
  `playlistduration` measures: `check --all` of a movie of that length
  renders every frame of the week.
