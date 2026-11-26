// widget-worker.js: one <psanim-widget>'s engine, off the page's main thread.
//
// The widget posts, in order:
//   { type: "init", module, canvas }   a compiled psanim.wasm and the widget's
//                                      OffscreenCanvas (transferred)
//   { type: "open", id, text | entry, params, scale, want }
//   { type: "frame", id, at, signals: [[name, value], ...] }
//   { type: "event", name, t }  { type: "clear", name }
// and gets back, for each message with an id,
//   { id, begun: true }   when it starts on it (the page times it from here)
//   { id, ok: true, value }  or  { id, ok: false, info, text }
// Messages are handled one at a time, in the order they came: an event
// posted after a clear is added after it.
//
// A frame that never finishes is the page's to stop: it terminates this
// Worker (the widget's `timeout`), which is why each widget has its own.

import { PsAnim, PsAnimError } from "./psanim.js";
import { openMovie, drawFrame } from "./widget-core.js";

let engine = null;
let canvas = null;
let ctx = null;
let queue = Promise.resolve();
let failedToStart = null; // why the engine could not start, for every later call

async function handle(d) {
  if (d.type !== "init" && !engine) throw failedToStart ?? new Error("the engine has not started");
  switch (d.type) {
    case "init": {
      canvas = d.canvas;
      ctx = canvas.getContext("2d");
      try {
        engine = new PsAnim(await WebAssembly.instantiate(d.module, {}));
      } catch (e) {
        failedToStart = new Error(`the engine could not start: ${e?.message ?? e}`);
      }
      return undefined;
    }
    case "open":
      return openMovie(engine, canvas, d);
    case "frame":
      return drawFrame(engine, ctx, d.at, d.signals);
    case "event":
      engine.event(d.name, d.t);
      return undefined;
    case "clear":
      engine.clearEvents(d.name);
      return undefined;
    default:
      throw new Error(`widget worker: unknown message ${d.type}`);
  }
}

self.onmessage = (m) => {
  const d = m.data;
  queue = queue.then(async () => {
    if (d.id != null) self.postMessage({ id: d.id, begun: true });
    try {
      const value = await handle(d);
      if (d.id != null) self.postMessage({ id: d.id, ok: true, value });
    } catch (e) {
      if (d.id == null) return; // an event or clear: nothing waits on it
      self.postMessage({
        id: d.id,
        ok: false,
        info: e instanceof PsAnimError ? e.info : { message: String(e?.message ?? e) },
        text: String(e?.message ?? e),
        ps: e instanceof PsAnimError,
      });
    }
  });
};
