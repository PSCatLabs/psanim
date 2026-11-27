# Boot Sequence

A 22.9 second film (1280x720, 30 fps) set in PSCat 0.8.0's pixel faces. A
video terminal warms up and runs its power-on test in Phosphor; the
operator logs in and sends a status report to the printer, and the camera
follows it to the paper, where it prints in Pinpoint on green-bar; the
camera pulls back as the operator turns the status board on, which wipes
"SYSTEM ONLINE" and then "ALL SYSTEMS NOMINAL" across in Diode7; each
device gets its name in Diode5 and the room holds still for three seconds.

## What it is for

It shows the faces through psanim's own devices, so one film checks all
three integrations:

```postscript partial
/crtface   /Phosphor def
/printface /Pinpoint-Draft def
/boardface /Diode7 def
/labelface /Diode5 def

/pr    /dotmatrix << /Params << /Text report /Face printface ... >> >> instance
/tty   /terminal  << /Params << /Plan session /Font crtface ... >> >> instance
/board /ledmatrix << /Params << /Font boardface /Grid [112 8] ... >> >> instance
/labelfont labelface pixelface def
```

- The terminal `show`s its session in `/Phosphor` itself, glow, bowed
  glass and scanlines included.
- The printer reads Pinpoint's bitmaps as column masks (`/Pinpoint-Draft
  pixelfont`, in pkpixel) and strikes them dot by dot as the head passes,
  so the letters are Pinpoint's while the ink, the sweeps and the paper
  feed are the entry's own.
- The board lights Diode7's lamps the same way, on a grid 8 rows high.
- The labels are plain `show` in `/Diode5`, loaded once in setup by
  `pixelface`, because `findfont` does not load these faces by name.

The film names the faces once, at the top; psanim keeps their files in one
table at the top of `crates/psanim/lib/pkpixel.ps`.

**Time comes from one session plan.** The printer starts at the mark where
`status > lpt1` is entered; the session then waits as long as the printer
takes (`/pr componentduration`), so the operator types `board on` as the
last line is struck; the board starts at that mark, and the film ends three
seconds after the board's last message has fully wiped in. The camera is
`lookat` on a box that `keyframes` moves: the top of the screen, then the
paper at the print head, then the whole room.

`check --all` warns of about ten abrupt changes: they are lines appearing
on the terminal and the printer's head striking a line on its way back,
which is how those devices behave.

Cost: median 48 ms a frame at full size (`psanim bench`, p95 127 ms in
the wide shot with all three devices), measured on an Apple M4
while other builds were running, so expect less on a quiet machine.

## Render

```sh
psanim check  gallery/boot-sequence/boot-sequence.ps
psanim sheet  gallery/boot-sequence/boot-sequence.ps --count 16 --cols 4 -o out/boot-sequence/sheet.png
psanim render gallery/boot-sequence/boot-sequence.ps -o out/boot-sequence/boot-sequence.mp4
```
