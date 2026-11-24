# Halftone

A 16 second loop (1280x720, 30 fps): a day at sea, dawn to night,
printed in four inks. Nothing is drawn as a picture. Each ink is a screen
whose mark sizes are a function of position and time; the screens sit at
different angles and slightly out of register.

## PSCat library: `lib/halftonekit.ps`

`halftone` fills the clip with a dot or line screen and calls a `/Tone`
procedure at each cell. The movie has four tone procedures (yellow,
magenta, cyan, and a black line screen for the hills, the sea and the
night) that read the sun's position, which is a function of `progress`.
Each ink is drawn in a `blended` layer in `/multiply`, so the inks
overprint as on paper: yellow over cyan is green, and all four make the
dark of the hills. The night sky is a second pass of the black ink as
dots, because a dot can shrink to nothing as dusk begins and the
thinnest line of a line screen is still a line: with lines the sky
changed visibly in one frame at sunset.

Cost: about 220 ms/frame at full size on one core. It is the slowest pure piece
in the gallery: some 9,000 cells an ink, each calling an interpreted
tone procedure. It still renders on every core.

## Render

```sh
psanim check  gallery/halftone/halftone.ps
psanim sheet  gallery/halftone/halftone.ps -o out/halftone/sheet.png
psanim render gallery/halftone/halftone.ps -o out/halftone/halftone.mp4
```
