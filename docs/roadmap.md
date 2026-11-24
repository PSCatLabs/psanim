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

## Next

Roughly in order of value:

1. **Merge the PSCat branch** and depend on a released PSCat.
2. **A Worker-based browser player** with a time budget, and export from
   the page through WebCodecs.
3. **An MCP server** (`psanim-mcp`): `check`, `frame`, `sheet` and
   `render` as tools returning images and structured diagnostics, so an
   agent without a shell can author movies. The CLI's `--json` output is
   already the shape those tools would return.
4. **Audio**: read a WAV into level and onset signals natively; mux an
   audio file into the render.
5. **Linear-light compositing** and 16-bit intermediates for layers.
6. **Incremental rendering**: cache frames by a hash of program,
   options and time so an edit that changes one clip re-renders only its
   frames; cache static layers within a render.
7. **Group opacity in PSCat itself**, so layered frames can export as
   vector.
8. **More vocabulary, led by the gallery**: tapered strokes along a
   path, radial repetition, a gradient shorthand, per-contour morphing.
9. **Source positions in PSCat procedures**, for line numbers in
   backtraces.
