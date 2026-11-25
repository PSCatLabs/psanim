# News Sign

A 39.9 second film (1280x720, 30 fps): a scrolling LED news sign on a brick
wall at night, after the signs of the 1980s that carried headlines, scores,
the time and the temperature. Red LEDs sit behind smoked glass in a metal
housing. Each message arrives its own way: the time drops in with a bounce,
the headlines scroll in from the right, a score is wiped on by a bright bar,
the temperature sparkles on, a weather alert flashes, and the film ends on the time. A smaller green sign
below runs a scores ticker round and round. The headlines and scores are
invented.

## What it is for

Both signs are the catalog's `/ledmatrix` with `/Look /sign`; the film says
what they show:

```postscript partial
/headlines [
  [ /time /drop 2.2 /amber ]
  [ (TRAM LINE 4 REOPENS MONDAY) /scroll 0 /red ]
  [ (MARSH FC 3  ROVERS 1) /wipe 2.4 /green ]
  [ /temp /sparkle 2.2 /amber ]
  ...
] def
/main /ledmatrix << /Params << /Items headlines /Grid [96 7] /Look /sign /Repeat false >> >> instance
/ticker /ledmatrix << /Params << /Text scores /Effect /marquee /Grid [65 5] /Colour /green >> >> instance
```

Each item is `[ text effect hold colour ]`; the text may be `/time` or `/temp`,
which the sign computes from `/Clock` (the time it starts), `/ClockRate`
(seconds a displayed minute takes) and `/Temp`: no wall clock is read, so
every frame is still a function of the time. A scrolling item lasts as
long as its text takes to cross; the others hold for the seconds given. The
7 row signs use the entry's 7 row pixel font, the ticker its 5 row font.
The ticker is a `/marquee`: it repeats with the exact period of its text,
so it would loop seamlessly on its own.

LEDs are tested against column bitmaps of the font (no anti-aliasing, no
rendering to a small layer and reading it back), scroll in whole columns
as a real sign does (30 LEDs a second is one column a frame at 30 fps), and
are drawn in a few passes: all the unlit LEDs as one path, a soft halo, the
bodies, the glass, and the hot centres on top of the glass.

The wall is drawn in the movie: bricks with a hash of their place for tone,
the red and green light of the signs spilling on it (`blurred`), and a soft
shadow under each.

Cost: about 56 ms a frame at full size (`psanim bench`), most of it the
three blurred layers behind the signs and the wall; the big sign
alone is about 12 ms.

## Render

```sh
psanim check  gallery/news-sign/news-sign.ps
psanim sheet  gallery/news-sign/news-sign.ps -o out/news-sign/sheet.png
psanim render gallery/news-sign/news-sign.ps -o out/news-sign/news-sign.mp4
```
