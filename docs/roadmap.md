# Limitations and roadmap

What psanim does not do today, stated plainly, and what would come next.

## Known limitations

**Rendering**

- CPU only. Frames are rasterized by PSCat (tiny-skia) on the CPU;
  throughput comes from rendering frames on all cores. A full-canvas
  blur or glow costs about 10 to 30 ms at 1080p.
- 8 bits per channel, sRGB-encoded, composited in gamma space. Layer
  blends and motion-blur averaging are not linear-light, so very
  saturated additive glows can band or shift slightly.
- Layers are raster. `faded`, `blurred` and `glowing` work on pixels at
  output resolution, which is right for video and means there is no
  vector (SVG or PDF) export of a frame that uses them.
- Video is 4:2:0 through ffmpeg. No alpha video, no audio track, no
  ProRes or image-sequence formats other than PNG.
- The canvas is at most 8000 pixels a side (PSCat's limit).

**Model**

- No audio. Signals can carry an audio analysis made elsewhere (levels,
  beats as an event track), but psanim neither reads nor writes sound.
- Stateful and persistent movies render on one thread, in order, and
  seeking back replays from a checkpoint.
- `trimpath` and friends flatten curves to polylines (about 1.5 pixels
  per chord), invisible at normal sizes, visible if the result is scaled
  up a great deal afterwards.
- `morphpath` handles one closed outline per shape; it does not match
  the separate contours of two glyphs.
- Errors inside a scene are located by procedure and element, not by
  source line: PSCat keeps no source positions for procedure bodies.
- Strings are exempt from PostScript's `restore`, so a scene that writes
  into a string made in setup leaks that change to later frames.

**Platform**

- Developed and tested on macOS (Apple silicon). Linux and Windows
  should work (nothing is platform-specific outside the ffmpeg call) but
  have not been run.
- The browser build is tested in Node and Chromium only, runs on the
  main thread, and has no time limit (step limit only).
- Not published to crates.io; depends on a PSCat branch that is in review and not merged
  (PSCatLabs/pscat#1410).

**Safety**

- Resource limits are checked between interpreter steps; a single heavy
  operator can overrun them. Not an OS sandbox. See
  [security](security.md).

**Vocabulary** (the pk libraries)

- A thing in motion is rasterized: `enters` and its relatives draw the
  body on a layer and place that, so a large scale-up is soft while it
  moves, and each concurrent motion costs a canvas-sized layer (measured:
  about 1 to 1.5 ms per moving thing at 1280x720; 24 at once, 23 ms a
  frame).
- A motion measures what was painted. Applied to something that covers
  the frame (a ground) it moves the frame. Motions move in screen
  directions whatever the transformation.
- Text is ASCII in the text fonts. `sym` and run lists reach the Symbol
  font; there is no Unicode fallback and a missing glyph is silent.
- `showin` and `node` labels do not wrap or shrink a long string by
  themselves; `kinetic` has no tracking option.
- pkchart has no stacked or grouped bars, no log scales, no unit charts,
  and ticks re-pick abruptly while a range animates.
- pkdiagram has no automatic layout: boxes are placed by the author.
- Storyboard beats cut; there is no transition between beats as there is
  between a reel's clips.
- The code tokenizer is a highlighter, not a parser: one keyword list
  per language, no nested or multi-line strings.

## Next: the vocabulary

What the second gallery's authors built by hand, in the order of how
much it cost them. Each is self-contained and can be given to another
agent with the report it came from (summarised in
[the evaluation](vocabulary-evaluation.md)).

1. **A stroke scheduler for painting over time** (`paintplan`): a list of
   strokes with a start, a duration, a seed and a kind (along a path, a
   sweep, a soak), played onto a persistent canvas, with the active
   stroke's tip available to `overlay` for a brush cursor. Chalk Proof
   and Heron each wrote this from scratch (about 60 lines each).
   `revealband` is its first brick.
2. **Equations as objects.** Run lists set a line and bring it in term
   by term. Missing: matching terms between two lines so one equation
   *becomes* the next (terms slide, cancel, appear), and mathkit's TeX
   layout exposed per term so it can be used the same way.
3. **Soft and ragged reveals**: a feathered or noisy edge for `spotlight`,
   wipes and wash fronts, which now cut hard.
4. **Transitions between storyboard beats**, reusing the reel's.
5. **Typography**: tracking as an option of `kinetic` and as a motion
   ("tracks out"); text fitted by cap height with the box it ends up in
   returned; a flow layout (stack lines top-down with gaps) so fitted
   lines place their neighbours; vertical and rotated text in a box;
   outline type as a state.
6. **Charts**: a tracer (a point riding a series with its readout), unit
   charts (a waffle of cells), cross-fading ticks, stacked and grouped
   bars, a ruler outside a plot frame.
7. **Diagrams**: a layered layout for graphs given as data; a section
   header / lower-third word; labels along curved edges.
8. **Motion**: vector motions when a pivot is given (no rasterization);
   motion along the text's own axis under rotation; per-contour
   `morphpath`.
9. **Diagnostics**: `check` warning on non-ASCII bytes shown in a text
   font, on text drawn outside the canvas, and on type below a legible
   size; marking intended cuts so `check --all` can be fully clean;
   naming the procedure that left operands on the stack.

## Next: the engine

Roughly in order of value:

1. **Merge the PSCat branch** and depend on a released PSCat. (The
   vocabulary work needed no further PSCat changes.)
2. **A Worker-based browser player** with a time budget, and export from
   the page through WebCodecs.
3. **An MCP server** (`psanim-mcp`): `check`, `frame`, `sheet` and
   `render` as tools returning images and structured diagnostics, so an
   agent without a shell can author movies. The CLI's `--json` output is
   already the shape those tools would return.
4. **Audio**: read a WAV into level and onset signals natively (the
   `tempo` words then follow a real track); mux an audio file into the
   render.
5. **Linear-light compositing** and 16-bit intermediates for layers.
6. **Incremental rendering**: cache frames by a hash of program,
   options and time so an edit that changes one clip re-renders only its
   frames; cache static layers within a render; checkpoints on disk for
   persistent paintings.
7. **Group opacity and placed layers in PSCat itself**, so layered and
   moving frames can export as vector.
8. **Source positions in PSCat procedures**, for line numbers in
   backtraces.
