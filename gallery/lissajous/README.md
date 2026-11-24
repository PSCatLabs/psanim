# Lissajous

A seamless 16 second loop (1080x1080, 30 fps): the Lissajous table.
Six wheels along the top turn at 1 to 6 times the base rate, and six
more down the side. Each cell takes its x from the wheel above it and
its y from the wheel beside it, and that point draws the curve for the
pair: circles on the diagonal, and elsewhere the figure of the ratio.
2:1 and 4:2 are the same curve, traced twice as fast.

## How it works

- **No fade, no cut.** Every frequency is a whole number, so after one
  turn of the slowest wheel all 36 points are back where they started.
  Nothing is "drawn and then cleared": each point is a comet whose tail
  is the last third of a turn of base phase, brightest at the head.
- **A tail is 110 short strokes** whose colour and width fall off along
  it. The whole closed curve sits faintly underneath as one path.
- **Guides** carry each wheel's point down its column or along its row,
  so the eye can see where a cell's x and y come from.
- One `glowing` layer over everything that is light.

Cost: about 4,600 strokes and 68 ms a frame at full size on one core.

## Render

```sh
psanim check  gallery/lissajous/lissajous.ps
psanim sheet  gallery/lissajous/lissajous.ps -o out/lissajous/sheet.png
psanim render gallery/lissajous/lissajous.ps -o out/lissajous/lissajous.mp4
```
