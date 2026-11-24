# Neon

A 12 second loop (1280x720, 30 fps): a neon sign on a brick wall at
night. The letters strike one at a time, stuttering as they catch; one
tube in the second line never quite holds; then the sign is switched
off, which is where the loop starts again.

## PSCat library: `lib/fonts/neon.ps`

`/Neon` is a Type 3 font whose glyphs are tube skeletons overdrawn five
times, wide and dim to thin and bright. It reads its tube colour from a
dictionary when a glyph is drawn, so the movie shows the sign a letter at
a time with `eachglyph` and sets the colour (lit or dead) before each.
Whether a letter is lit is a function of `now` and `hash01`, so the
flicker is identical on every render and any frame can be drawn alone.
`glowing` adds the light on the wall; the wet pavement is the sign drawn
again upside down inside `blurred` and `faded`.

Cost: about 25 ms/frame at preview size, nearly all in the two layers;
the font itself is about 1.5 ms.

## Render

```sh
psanim check  gallery/neon/neon.ps
psanim sheet  gallery/neon/neon.ps -o out/neon/sheet.png
psanim render gallery/neon/neon.ps -o out/neon/neon.mp4
```
