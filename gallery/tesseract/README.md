# Tesseract

A seamless 28 second loop (1080x1080, 30 fps): a hypercube turned in
each of the six planes that four dimensions have, then in two at once.

In three dimensions a rotation has an axis. In four it has a plane, and
there are six: xy, xz, yz, xw, yw, zw. The film goes through them in
order, with a dial for each:

1. **xy, xz, yz**: the spins we know. The figure (a cube inside a cube,
   which is the hypercube's shadow in our space) turns rigidly.
2. **xw, yw, zw**: turns involving w, the direction we cannot see. The
   inner cube swells, passes through the outer one and becomes it.
3. **yz and xw together**: two planes that share no axis turning at
   once, which only four dimensions allow.

## How it works

- Sixteen corners (+-1, +-1, +-1, +-1) are rotated in 4D plane by
  plane, then projected twice in perspective: 4D to 3D from a point on
  the w axis, 3D to the page from a point on the z axis.
- Colour is w: cyan far along w, magenta near. Each of the 32 edges is
  drawn in eight pieces so colour and width (the perspective scale) run
  along it; the 24 faces are faint panes.
- Every turn is a half turn, which maps the hypercube onto itself, so
  the last frame leads into the first.

The corners, edges and faces are computed in setup from bit patterns.
Cost: about 20 ms/frame at full size on one core, most of it the `glowing` layer.

## Render

```sh
psanim check  gallery/tesseract/tesseract.ps
psanim sheet  gallery/tesseract/tesseract.ps -o out/tesseract/sheet.png
psanim render gallery/tesseract/tesseract.ps -o out/tesseract/tesseract.mp4
```
