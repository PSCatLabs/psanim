# Letter

A 25 second film (1280x720, 30 fps): a letter typed on a manual
typewriter. The line being typed stays put and the paper climbs at each
carriage return; when the letter is finished the sheet is drawn back to
be read.

## PSCat library: `lib/typewriter.ps`

`tw-type` types a text the way a machine would: each key has fixed
faults (offset, tilt, weight) from `/Machine`, each strike its own
pressure from `/Seed`, and the ink is laid through a ribbon weave. The
same machine and seed always type the same thing, so the movie simply
types the first n characters on every frame: character n arrives and
the ones before it do not move. The text is broken into lines by hand so
that no word changes line while it is being typed. The paper's climb is
a sum of `tween`s, one per newline.

Cost: up to about 50 ms/frame with the whole letter on the sheet (the
letter is retyped on every frame: frames are independent).

## Render

```sh
psanim check  gallery/letter/letter.ps
psanim sheet  gallery/letter/letter.ps -o out/letter/sheet.png
psanim render gallery/letter/letter.ps -o out/letter/letter.mp4
```
