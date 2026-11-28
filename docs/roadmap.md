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
- Text in a `face` font is UTF-8 with PSCat's fallback faces and colour
  emoji, but Korean, Japanese, Chinese, Thai, Arabic, the Indic scripts,
  Armenian, Georgian and Ethiopic have no face yet (each character draws
  an empty box, and `check` names it); there is no shaping or bidi, so
  Hebrew runs left to right; fallback faces are regular weight. Fonts
  chosen with `findfont` still read bytes (`check` warns on UTF-8 in one).
  The catalog entries (chat, splitflap, terminal ...) choose their own
  fonts and are still byte text, and `typingplan` types bytes.
- `showin` and `node` labels do not wrap or shrink a long string by
  themselves; `kinetic` has no tracking option.
- pkchart has no stacked or grouped bars, no log scales, no unit charts,
  and ticks re-pick abruptly while a range animates.
- pkdiagram has no automatic layout: boxes are placed by the author.
- A transition between storyboard beats (`[ /a 4 [0.6 /dissolve] /b 6 ]
  storyboard`) works as a reel's does, with the same conditions: each
  beat on either side paints its own ground, its `inbeat` calls come in
  the storyboard's order, and a transition of one's own draws the
  outgoing side and then the incoming one, neither depending on the
  other (every built-in one does). A beat body that fails inside a
  transition's layer leaves the layer open, as in a reel.
- The code tokenizer is a highlighter, not a parser: one keyword list
  per language, no nested or multi-line strings.

## Next: the vocabulary

What the second gallery's authors built by hand, in the order of how
much it cost them. Each is self-contained and can be given to another
agent with the report it came from (summarised in
[the evaluation](vocabulary-evaluation.md)).

1. **A stroke scheduler for painting over time**: done, as pkink's
   `paintplan` (#107): strokes /along a path, /sweep and /soak, kept once
   on the step that finishes them and regrown in `overlay` until then,
   exact at any `--fps`, with `painttip` for a brush. Chalk Lecture uses
   it (byte-identical to its hand-written scheduler); Evening Harbour was
   painted with it. Heron and Chalk Proof keep their own: Heron regrows its
   pen strokes on the kept canvas and Chalk Proof lays its chalk in
   clipped pieces, so either would change its look if moved.
2. **Equations as objects.** Run lists set a line and bring it in term
   by term, and `morphruns` (#104) makes one line become the next: terms
   are paired by content in order or by tag (`[/as /key run]`), slide,
   cross-fade when a tag joins different glyphs, and leave or arrive;
   a chain of lines is one progress number, and a tagged term can hold
   still (an aligned derivation). Still missing: mathkit's TeX layout
   exposed per term so displayed formulas (radicals, big fractions,
   matrices) can be matched the same way; colour per term inside a run
   list (today a term is marked by drawing over its `runbox`); and
   pairing a term with two (one 3 becoming two halves).
3. **Soft and ragged reveals**: a feathered or noisy edge for `spotlight`,
   wipes and wash fronts, which now cut hard.
4. **Typography**: tracking as an option of `kinetic` and as a motion
   ("tracks out"); text fitted by cap height with the box it ends up in
   returned; a flow layout (stack lines top-down with gaps) so fitted
   lines place their neighbours; vertical and rotated text in a box;
   outline type as a state.
5. **Charts**: unit charts (a waffle of cells), stacked and grouped
   bars, a ruler outside a plot frame. Done: names at line ends kept
   clear of each other (`serieslabels`) and a tracer with its readout
   (`tracer`, `lineat`), #105; cross-fading ticks (`plotaxes /Fade`).
6. **Diagrams**: a layered layout for graphs given as data; a section
   header / lower-third word; labels along curved edges.
7. **Motion**: vector motions when a pivot is given (no rasterization);
   motion along the text's own axis under rotation; per-contour
   `morphpath`.
8. **Diagnostics**: `check` warning on text drawn outside the canvas,
   and on type below a legible size; marking intended cuts so `check --all` can be fully clean;
   naming the procedure that left operands on the stack.
9. **A review loop.** Blind review found what authors did not (holds
   that are not holds, dead time, faint text) and authors score
   themselves 7 to 8 whatever the film. `check` could measure some of it
   (frames changing during a stated hold, a second with nothing
   changing mid-film, an ending in motion); the rest wants a second
   agent reviewing a sheet and one revision pass.
   The template experiment (issue #108, [evaluation, section
   5](vocabulary-evaluation.md#5-templates-for-everyday-explainers))
   adds two things. Reviewers' commonest complaint across eight films was
   reading time with nothing changing, and text swapped through an empty
   frame; `check` could flag both (a long run of identical frames
   mid-film; a region that goes blank and refills). And a template is a
   place where one review pays many times: both models made the same
   film from it, so fixing a template fixes every film made from it. It
   took three rounds: after blind review and revision, both templates
   rank first in their sets, and the template tests now catch the poem
   template's round-2 defects (two notes legible at once, a card left
   empty, unfinished sentences, the closing hold) mechanically.

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
