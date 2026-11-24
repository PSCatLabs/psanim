# Afterglow

The opening titles of an imaginary film, 19.9 s at 1280x720, 30 fps.

A sun goes down behind a flat horizon while four credit cards pass on the
left of the frame. A line of italic rides around the sun's limb and goes
out as the sun meets the horizon. When the last light has gone, the main
title draws itself on from its outlines, fills, takes one sweep of light
and holds; its O burns in the single accent colour of the piece.

Two faces (AvantGarde-Book capitals, Palatino-Italic), five colours, one
accent. The movie is pure: every frame is a function of its time.

## Techniques

| psanim word | What it does here |
|---|---|
| `sequence` | Places the four cards one after another, each on its own clock. |
| `span`, `tween`, `faded` | One `card` procedure enters from 0 and leaves before `span`, so it is reused four times unchanged. The exit fades the card as one group. |
| `eachglyph`, `staggered`, `delayed` | The letters of each name rise into place in turn, letterspaced. |
| `trimpath` | The accent rule beside each card and the horizon hairline draw on. |
| `charpath` + `trimeach` | The main title's outlines draw on, every contour of a letter at once. |
| `pathlength`, `pathpoint` | Sets the tagline along an arc over the sun, glyph by glyph. |
| `blurred` | The far layer: the title at 330 units, out of focus, drifting. |
| `glowing` | The sun, and later the O of the title; nothing else glows. |
| `clip` (plain PostScript) | The light sweep is a soft white band clipped to the letters. |
| `keyat` + `globaltime` | The sun's height, read the same inside every time scope. |
| `smoothstep`, `between` | The sky warms as the sun sinks; letters stay brighter after the sweep. |
| `camera` | A slow 7% push-in over the whole film. |
| `after` | Starts the main title on its own clock. |
| `vignette`, `filmgrain` | The filmic surface, applied last. |

## Render

```sh
psanim check  gallery/afterglow/afterglow.ps
psanim sheet  gallery/afterglow/afterglow.ps -o out/afterglow/sheet.png
psanim render gallery/afterglow/afterglow.ps -o out/afterglow/afterglow.mp4
```

`check` warns that frame 0 is blank: the film opens from black on purpose.
About 42 ms per frame at full size on one thread.
