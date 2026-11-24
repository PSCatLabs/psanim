# Taiji

A seamless 24 second loop (1080x1080, 30 fps, 720 frames): the yin-yang
symbol, turning. Each of its two eyes is a whole yin-yang turning the
other way, and each of that one's eyes is another, seven levels deep,
until an eye's radius is under a third of a canvas unit and it is drawn
as the plain dot it has become. 127 symbols and 128 dots a frame.

## How it works

- **One recursive procedure.** `taiji` draws in a unit circle: a disc,
  then one S-shaped half over it. For each eye it translates to the
  centre of a lobe, scales by 0.32 and calls itself. The radius in
  canvas units travels on the operand stack and stops the recursion.
- **The loop closes by whole turns.** Level k makes k+1 turns per loop
  relative to its parent, alternately one way and the other, so against
  the canvas the levels make +1, -1, +2, -2, +3 ... turns and every
  one is home at 24 s.
- **Half a turn is the negative.** A yin-yang turned 180 degrees is
  itself with the colours exchanged. So the eye in the dark lobe is
  drawn as the eye in the light lobe with its two colours swapped and a
  half turn added: the same picture, but each disc is laid on a rim of
  its own colour, which leaves no hairline of the wrong colour at an
  antialiased edge.

Cost: about 58 ms/frame at full size on one core; nearly all of it is
the two `blurred` layers (a warm light and a shadow behind the symbol).

## Render

```sh
psanim check  gallery/taiji/taiji.ps
psanim sheet  gallery/taiji/taiji.ps -o out/taiji/sheet.png
psanim render gallery/taiji/taiji.ps -o out/taiji/taiji.mp4
```
