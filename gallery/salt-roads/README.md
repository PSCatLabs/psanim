# The Salt Roads

A 50 second opening crawl (1280x720, 30 fps) for a film that does not
exist. A title and three paragraphs of an invented story recede over a
starfield on a tilted plane, justified, shrinking towards a vanishing
point and fading as they go. As the last paragraph thins out, the camera
tilts down past it to the night side of a planet, and a sun comes up over
the rim.

## What it is for

It shows `onplane`, psanim's perspective word, doing what the CTM cannot:
making the far end of a drawing smaller than the near end. `onplane` runs
any procedure with its fills, strokes, text and clips projected onto a
plane in 3D, and the catalog's `/crawl` entry is built on it. The piece
places that entry with its own words, type and colours:

```postscript partial
/story /crawl << /Params <<
  /Title [ (CHAPTER SEVEN) (THE SALT ROADS) ]
  /Text [ (For three hundred years ...) (When the light at Corvane ...) (Now Ilse Marrow ...) ]
  /Font /AvantGarde-Demi /Size 44 /Width 860
  /Color [0.80 0.90 0.98] /TitleColor [1.0 0.86 0.62]
  /Tilt 60 /Stars 0 /Sky /none
>> >> instance
```

## Techniques

- **The crawl** lays its text out once, in setup: title lines centred,
  paragraphs justified (the last line of each set ragged). Each frame it
  draws every line that is in view as one path (a `charpath` per word,
  one `fill`) inside `onplane`. One path per line matters: the plane's
  fade gives each painted path one opacity, from its depth, and a line on
  a tilted plane is all at one depth.
- **Pacing.** The entry fits its text into its own 40 second clock; the
  piece places it at `/Speed 0.6`, so the words are readable for about
  half a minute. Constant speed along the plane looks like slowing down
  on the screen, as it should.
- **The camera tilt** is a `translate`: stars, crawl and planet are drawn
  in one space that slides up as the camera looks down, and the stars are
  scattered over a field taller than the frame so there are stars to tilt
  down to. The crawl's own starfield is off (`/Stars 0`) for that reason.
- **The planet** is a dark disc under a rim of light: concentric strokes,
  fainter outward, inside `glowing`. The sun is a disc in a halo of faint
  discs, `blurred`, and clipped to the sky with `eoclip` (the planet's
  disc inside a large rectangle), so it rises from behind the rim.

About 5 ms a frame at full size during the crawl and 6 to 7 ms over the whole film, with the planet frames (`glowing`, `blurred`) the dearest at about 20 to 30 ms (`psanim bench`, 1280x720, on a busy machine).

## Render

```sh
psanim check  gallery/salt-roads/salt-roads.ps
psanim sheet  gallery/salt-roads/salt-roads.ps -o sheet.png
psanim render gallery/salt-roads/salt-roads.ps -o salt-roads.mp4
```
