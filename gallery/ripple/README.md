# Ripple

A 10 second loop (1280x720, 30 fps): two wave sources interfering on a
square of water, drawn as 73 ridge lines, with the formula above it.

## PSCat libraries: `lib/graph.ps` and `lib/mathkit.ps`

`setview` and `surfacerow` from graph.ps project z = f(x, y) one row at
a time. graph.ps draws wireframes without hidden lines, so the movie
draws the rows from the far edge to the near one and fills under each
with the ground colour before stroking it: a near ridge hides what is
behind it, with no polygon sorting. The view sways through one period
and the wave phase turns three whole times, so the loop closes. The
formula is TeX laid out as pen strokes by mathkit and stroked plainly.

Cost: about 22 ms/frame at preview size for 8,000 evaluations of the
wave function and one `glowing` layer.

## Render

```sh
psanim check  gallery/ripple/ripple.ps
psanim sheet  gallery/ripple/ripple.ps -o out/ripple/sheet.png
psanim render gallery/ripple/ripple.ps -o out/ripple/ripple.mp4
```
