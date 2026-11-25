# How psanim works

A 69.5 second architecture explainer of psanim, written in psanim
(1280x720, 30 fps, one file: `how-it-works.ps`). On-screen text only.

1. **A movie is one PostScript program**: a seven-line program is typed
   in close-up, the camera pulls back, the declaration and then the
   `scene` procedure are focused while a small screen shows what they
   describe.
2. **A frame is a function of its time**: clock -> scene -> frame, one
   message per frame, a filmstrip filling up; then the setup snapshot,
   restore and roll back.
3. **Time scopes re-base the clock**: `translate` on a ruler of space
   above `during` on a ruler of time, with the movie clock and the clock
   inside the scope read out side by side.
4. **So any frame renders alone, in any order**: one frame by itself,
   then all of them fanning out to four cores in scattered order and into
   the encoder.
5. **The stack**: program -> psanim -> PSCat -> frames -> ffmpeg / browser,
   with a camera move into the two middle boxes for their parts.
6. A closing summary.

## Techniques

- `storyboard` / `inbeat` for the sections; every time in a section is
  local to its beat.
- `pkdiagram` `node`, `edge` (with `pulsetrain`), `connect` + `pulse` for
  single messages timed inside a `looped` scope, `arrow` on a hand-built
  path, `circled`.
- `pkcode` `codetokens` / `codeblock` with `/Reveal` and `/Focus`.
- `pkmotion` `enters`, `onstage`, `staged`, `leaving`, `cascade`.
- `pktype` `captions`, `kinetic`, `typewrite`, `showin`.
- `lookat` over `keyframes` / `between` of boxes for the two camera moves.
- The little ball movie shown throughout is a real procedure placed with
  `within`, `attime`, `looped`, `during` and `timescaled`: the film uses
  the ideas it explains.

## Commands

```sh
psanim check  gallery/how-it-works/how-it-works.ps --all
psanim sheet  gallery/how-it-works/how-it-works.ps --count 24 -o out/how-it-works/sheet.png
psanim render gallery/how-it-works/how-it-works.ps -o out/how-it-works/how-it-works.mp4
```
