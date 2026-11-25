# Banner

A 41 second film (1280x720, 30 fps): a dot-matrix printer in a dark office
prints a job header on green-bar paper and tears the page off. The header
ends "TURN YOUR HEAD LEFT", and then a banner starts, HELLO in letters made of
characters, printed sideways down the paper. The picture rolls a quarter turn,
as the header said, and the letters stand up as they print, one to a sheet;
each sheet is torn off as its letter is finished. At the end the printer
slides away and the five sheets arrive side by side along their perforations,
spelling the word. The job is invented.

## What it is for

The printer is the catalog's `/dotmatrix` and the five sheets are the same
entry with `/Sheet true`, one finished page lying flat:

```postscript partial
/job /dotmatrix << /Seed 4471 /Params << /Window [720 1000] /Text report /Banner (HELLO)
      /PageLines 18 /Tear /each /Speed 70 /Paper /greenbar /Backdrop false >> >> instance
/s0 /dotmatrix << /Seed 4471 /Params << /Sheet true /Top 0 /Text () /Banner (HELLO)
      /PageLines 18 /Page 0 >> >> instance      % ... /s4, /Page 4
```

The numbers fit together on purpose. The report is 15 lines and the printer
feeds 3 blank ones first, so the first page is 18 lines. A banner letter is
a 5 by 7 bitmap of the entry's own font, each lit pixel a block of 5
characters and 3 lines (a character cell is 6 dots across and a line is 10 dot widths
down, so the block is 30 by 30 dots, square), and a column of the letter is one printed line
of 7 blocks: a letter and the gap after it are 6 columns of 3 lines, 18 lines.
So `/PageLines 18` makes every letter a page of its own, and `/Tear /each` tears
a page off as it fills. The tops of the letters are at the paper's right edge,
so the page is read turned a quarter turn to the left, which is what the camera
does. The sheets repeat the text and settings with `/Page k`, which is the page
the printer tore off.

## The camera

The film is a rolled placement of one component: `gsave ... translate rotate
/job << /Box [...] >> place grestore`. The window is 720 by 1000, taller than the
film, so the first shot sees only its lower part with the printer at the
bottom, and after the roll the whole length of the paper runs across the film.
The roll turns about the print line, which travels from the lower centre to the
right, so the head and the paper coming out stay in view all through the turn.
When the picture has rolled, the paper streams leftward out of the printer on
the right.

The camera's times (when the roll starts, when the printer leaves) were read
from the job's plan, which is fixed by its parameters: the first page tears
off at 9.8 s, the banner starts at 11.2 s, the five letters' pages tear at
14.2, 19.3, 23.4, 27.6 and 32.5 s. Change the report or the speed and they move.

## Techniques

- Dots as zero-length round-capped strokes, stroked four times a line (one
  path a tone); the head's position and each dot's strike time are pure
  functions of the plan.
- Sheets sliding from the right one after another, each with an ease-out, in
  an order that cannot collide: the first to leave goes the furthest.
- `/Backdrop false`, so the printer sits in the room instead of its own dark
  cavity; the room is a radial shading and a vignette.

Cost, from `psanim bench` (48 frames sampled across the film at full size): a
median of about 16 ms a frame and 41 ms for the slowest; the load was about
0.6 s before the flat sheets learned to build dots only for their own page, now
about 0.24 s.

## Render

```sh
psanim check  gallery/banner/banner.ps
psanim sheet  gallery/banner/banner.ps -o out/banner/sheet.png
psanim render gallery/banner/banner.ps -o out/banner/banner.mp4
```
