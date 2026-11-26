// widget-core.js: what <psanim-widget> does with an engine, wherever the
// engine runs (in the page, or in the widget's Worker: widget-worker.js).

/**
 * Open a widget's movie on `engine` and size `canvas` to it.
 * `o`: { text, entry, params, scale, want }: the program text or a catalog
 * entry's name; its params; pixels per canvas unit, or none to fit `want`
 * device pixels across. Returns { meta, scale }.
 */
export function openMovie(engine, canvas, o) {
  let text = o.text;
  if (o.entry) {
    text = engine.catalog(o.entry);
    if (text == null) throw new Error(`no catalog entry named ${o.entry}`);
  }
  if (!text) throw new Error("the widget has no program");
  const params = o.params ?? {};
  let scale = o.scale;
  if (!(scale > 0)) {
    const probe = engine.open(text, { scale: 0.05, params });
    const want = o.want || probe.width;
    scale = Math.max(0.05, Math.min(2, want / probe.width));
  }
  const meta = engine.open(text, { scale, params });
  canvas.width = meta.pixel_width;
  canvas.height = meta.pixel_height;
  return { meta, scale };
}

/**
 * Set `signals` ([[name, value], ...]), render the moment `at` and draw it
 * on `ctx`. Returns { ms, steps }.
 */
export function drawFrame(engine, ctx, at, signals) {
  for (const [name, value] of signals) engine.setSignal(name, value);
  const started = performance.now();
  const frame = engine.render(at);
  const ms = performance.now() - started;
  ctx.putImageData(new ImageData(frame.data, frame.width, frame.height), 0, 0);
  return { ms, steps: engine.lastSteps };
}
