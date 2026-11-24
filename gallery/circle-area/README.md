# Circle Area

Why the area of a circle is πr², shown twice in 63 seconds (1280x720, 30 fps).

1. **Question.** A disc draws itself, its radius is labelled, the claim `Area = πr²` is stated.
2. **Rings.** The disc is cut into ten concentric rings. Each ring straightens into a strip
   (held at its lowest point, so the strips land already stacked); the stack is a staircase.
   The rings roll back up, the cut is redone with fifty rings, and the staircase becomes a
   triangle of base 2πr and height r: `Area = ½ · 2πr · r = πr²`.
3. **Wedges.** The disc is cut into twelve wedges. Each half opens like a comb and the two
   combs mesh; with sixty-four wedges the bumpy block becomes a rectangle πr by r:
   `Area = πr · r = πr²`.
4. **Close.** Disc, triangle and rectangle at one scale (equal areas), over the formula.

## Geometry

Every intermediate frame is exact, not an in-between of two drawings.

- A ring of mid-radius ρ and thickness w is drawn as an arc of constant length 2πρ stroked
  at width w, with curvature (1 − u)/ρ. Its area is length × thickness = 2πρw for every u,
  which is the ring's true area π((ρ + w/2)² − (ρ − w/2)²). The strips therefore sum to πr²
  at every ring count; only the outline is a staircase.
- The wedges are rigid sectors. A half's crusts ride a curve of length πr whose curvature
  falls to zero; while it opens, the halves part by exactly the amount the outermost tips
  swing past the centre line. Meshing moves the combs by R cos(θ/2) and R sin(θ/2) so the
  straight edges coincide.
- The circumference is kept in the accent colour throughout, and the brace, the label and
  the term of the equation that measure it share that colour; the radius has its own.

## Techniques

- `storyboard` / `inbeat` for the four beats; all times inside a beat are local.
- pkmotion: `staged`, `onstage`, `enters`, `entering` for headings, labels and equation
  terms; `emphasized` with `/pulse` to tie a label to the term that names it.
- pkdiagram `bracepath` for the braces, `trimpath` for everything that draws on.
- pkstage themes: `ink`, `face`, `mixink`, `ground`, `disc`.
- `crossfade` to change the number of pieces while the disc is whole (nothing visible moves).
- A small maths typesetter in the file (`mshow`, `mwidth`, `eqbuild`): runs of roman, italic,
  Symbol, superscript and a stacked ½, and an equation whose terms rise into fixed places.

## Commands

```sh
psanim check  gallery/circle-area/circle-area.ps --all
psanim sheet  gallery/circle-area/circle-area.ps -o out/circle-area/sheet.png
psanim frame  gallery/circle-area/circle-area.ps --at 60 -o out/circle-area/poster.png
psanim render gallery/circle-area/circle-area.ps -o out/circle-area/circle-area.mp4
```
