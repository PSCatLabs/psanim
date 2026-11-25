// <psanim-widget>: a psanim program that keeps running in a page.
//
//   <script type="module" src="widget.js"></script>
//   <psanim-widget entry="clock-digital" params='{"Signal": "clock"}'></psanim-widget>
//   <psanim-widget src="status.ps" fps="10"></psanim-widget>
//
// A widget is a movie plus live inputs. Where <psanim-player> plays a film
// with a scrubber, a widget has no controls and no end: it runs from the
// moment it opens, on the page's clock, until it is removed.
//
// Attributes
//   src      URL of a .ps program; or
//   entry    the name of a catalog entry (`psanim catalog` lists them)
//   params   JSON object of the program's `param` values, given at open:
//            '{"Theme": "amber", "Signal": "clock"}'. Values are read as the
//            command line reads --param (a number if it parses, else text).
//   time     how the page's clock maps to movie time:
//              loop  wrap at the duration (the default for a movie that
//                    declares /Loop true)
//              hold  run to the end and stay there (the default otherwise)
//              run   seconds since open, past the declared duration (for a
//                    movie written to be open-ended)
//   fps      the most frames a second to draw (default: the movie's /FPS)
//   scale    pixels per canvas unit (default: fit the element's width)
//   wasm     URL of psanim.wasm (default: beside this file)
//
// Inputs, set before every frame it draws
//   clock      local seconds since midnight, fractions included (what the
//              catalog clocks read with /Signal /clock)
//   mousex, mousey   pointer position in canvas units, y up
//   mousedown  1 while a button is down over the widget
//   hover      1 while the pointer is over the widget
//   key        the code of the last key pressed while it had focus
//   and any signal the page sets with widget.setSignal(name, value)
// Events (tracks for `/name sinceevent` and `/name eventcount`)
//   click      a press on the widget
//   keydown    a key press while it had focus
//   and any event the page adds with widget.event(name)
//
// It draws only when there is something new to draw: when the movie's
// frame number changes (at `fps`) or an input changed. It stops drawing
// while it is off screen or the tab is hidden. It dispatches psanim-load
// (detail: metadata) and psanim-error (detail: the structured error).

import { PsAnim, PsAnimError } from "./psanim.js";

// Compiled engines, one per wasm URL. A failed fetch or compile is
// forgotten, so a later widget (or a retry) can try again.
const modules = new Map();
function compiled(url) {
  let p = modules.get(url);
  if (!p) {
    p = fetch(url)
      .then((r) => {
        if (!r.ok) throw new Error(`cannot fetch ${url}: ${r.status}`);
        return r.arrayBuffer();
      })
      .then((bytes) => WebAssembly.compile(bytes));
    p.catch(() => modules.delete(url));
    modules.set(url, p);
  }
  return p;
}

/** Local seconds since midnight, with the fraction. */
export function secondsSinceMidnight(d = new Date()) {
  return d.getHours() * 3600 + d.getMinutes() * 60 + d.getSeconds() + d.getMilliseconds() / 1000;
}

const css = `
:host { display: inline-block; position: relative; line-height: 0; }
:host(:focus-visible) { outline: 2px solid #ffc94d; outline-offset: 2px; }
canvas { display: block; width: 100%; height: auto; }
.err { position: absolute; inset: 0; margin: 0; padding: 8px; overflow: auto; background: rgba(60, 12, 16, 0.94);
       color: #ffd9d4; white-space: pre-wrap; font: 11px/1.4 ui-monospace, Menlo, monospace; }
.err:empty { display: none; }
`;

export class PsAnimWidget extends HTMLElement {
  static observedAttributes = ["src", "entry", "params"];

  constructor() {
    super();
    this._request = 0;
    const root = this.attachShadow({ mode: "open" });
    root.innerHTML = `<style>${css}</style><canvas part="canvas"></canvas><pre class="err" role="alert"></pre>`;
    this.canvas = root.querySelector("canvas");
    this.ctx = this.canvas.getContext("2d");
    this.err = root.querySelector(".err");
    this.engine = null;
    this.meta = null;
    this.signals = new Map();       // the page's own signals, kept across reopens
    this.eventNames = new Set();    // event tracks used so far
    this.pointer = { x: 0, y: 0, down: 0, hover: 0, key: 0 };
    this.dirty = true;              // an input changed since the last frame
    this.lastFrame = -1;
    this.lastT = 0;
    this.visible = true;
    this.running = false;
    this.frameMs = 0;               // cost of the last frame drawn
    this.framesDrawn = 0;
    this.frameSkips = 0;            // animation frames with nothing new to draw
    this.#wire();
  }

  #wire() {
    const at = (e) => {
      if (!this.meta) return;
      const r = this.canvas.getBoundingClientRect();
      this.pointer.x = ((e.clientX - r.left) / r.width) * this.meta.width;
      this.pointer.y = (1 - (e.clientY - r.top) / r.height) * this.meta.height;
      this.dirty = true;
    };
    this.canvas.addEventListener("pointermove", at);
    this.canvas.addEventListener("pointerenter", (e) => { this.pointer.hover = 1; at(e); });
    this.canvas.addEventListener("pointerleave", (e) => { this.pointer.hover = 0; this.pointer.down = 0; at(e); });
    this.canvas.addEventListener("pointerdown", (e) => {
      this.pointer.down = 1;
      at(e);
      this.event("click");
    });
    this.canvas.addEventListener("pointerup", (e) => { this.pointer.down = 0; at(e); });
    this.addEventListener("keydown", (e) => {
      this.pointer.key = e.keyCode || 0;
      this.event("keydown");
    });
  }

  connectedCallback() {
    // Focusable, for keys; set here because a constructor may not add attributes.
    if (!this.hasAttribute("tabindex")) this.tabIndex = 0;
    this.io = new IntersectionObserver((entries) => {
      this.visible = entries.some((e) => e.isIntersecting);
      this.#wake();
    });
    this.io.observe(this);
    this.onVisibility = () => this.#wake();
    document.addEventListener("visibilitychange", this.onVisibility);
    this.ro = new ResizeObserver(() => this.#resized());
    this.ro.observe(this);
    // Moved in the document: disconnect closed the engine, so open again
    // (the widget's clock is kept).
    if (!this.engine && (this._source || this._entry)) this.#open();
  }

  disconnectedCallback() {
    // An open still compiling or instantiating must not finish on a
    // detached element, and a pending resize must not reopen it.
    this.generation = (this.generation ?? 0) + 1;
    clearTimeout(this.resizeTimer);
    cancelAnimationFrame(this.frameRequest);
    this.loop = (this.loop ?? 0) + 1;
    this.running = false;
    this.io?.disconnect();
    this.ro?.disconnect();
    document.removeEventListener("visibilitychange", this.onVisibility);
    this.engine?.close();
    this.engine = null;
    this.meta = null;
  }

  async attributeChangedCallback(name, old, value) {
    if (old === value) return;
    if (name === "src" && value) {
      // A later src or entry supersedes this fetch: only the newest request
      // may open its movie, whatever order the responses arrive in.
      const request = ++this._request;
      try {
        const r = await fetch(value);
        if (!r.ok) throw new Error(`cannot fetch ${value}: ${r.status}`);
        const text = await r.text();
        if (request === this._request) this.source = text;  // (the setter moves _request on)
      } catch (e) {
        if (request === this._request) this.#fail(e);
      }
    } else if (name === "entry" && value) {
      ++this._request;
      this._entry = value;
      this._source = null;
      this.#open();
    } else if (name === "params" && (this._source || this._entry)) {
      this.#open();
    }
  }

  /** The program text. Setting it opens the movie. */
  get source() {
    return this._source ?? "";
  }

  set source(text) {
    ++this._request; // supersedes any src fetch still in flight
    this._source = text;
    this._entry = null;
    this.#open();
  }

  /** The params given at open, from the `params` attribute. */
  get params() {
    const text = this.getAttribute("params");
    if (!text) return {};
    try {
      return JSON.parse(text);
    } catch {
      throw new Error(`params is not JSON: ${text}`);
    }
  }

  /** Set a signal the program reads with `/name signal`; it holds until set again. */
  setSignal(name, value) {
    if (this.signals.get(name) === value) return;
    this.signals.set(name, value);
    this.dirty = true;
  }

  /** Add an event now to the track the program reads with `/name sinceevent`. */
  event(name) {
    if (!this.meta) return;
    this.eventNames.add(name);
    // Wrap first, so clearing the last time round cannot take this event.
    const elapsed = this.#elapsed();
    const t = this.#movieTime(elapsed);
    this.#rollover(elapsed, t);
    this.engine.event(name, t);
    this.dirty = true;
  }

  /** The loop wrapped since the clock was last read, once or many times
   * (a hidden widget can sleep through several): events from earlier time
   * rounds are in this one's future, so forget them. Counted on the
   * unwrapped clock. Call before adding an event. */
  #rollover(elapsed, t) {
    const round = this.#timeMode() === "loop" ? Math.floor(elapsed / this.meta.duration) : 0;
    if (this.lastRound != null && round !== this.lastRound) {
      for (const name of this.eventNames) this.engine.clearEvents(name);
    }
    this.lastRound = round;
    this.lastT = t;
  }

  #elapsed() {
    return (performance.now() - this.t0) / 1000;
  }

  #timeMode() {
    const mode = this.getAttribute("time");
    if (mode === "loop" || mode === "hold" || mode === "run") return mode;
    return this.meta?.loop ? "loop" : "hold";
  }

  #movieTime(elapsed) {
    const d = this.meta.duration;
    switch (this.#timeMode()) {
      case "loop": return elapsed % d;
      case "run": return elapsed;
      default: return Math.min(elapsed, d - 1e-6);
    }
  }

  async #open() {
    const generation = (this.generation = (this.generation ?? 0) + 1);
    try {
      if (!this.engine) {
        const url = this.getAttribute("wasm") ?? new URL("psanim.wasm", import.meta.url).href;
        const module = await compiled(url);
        const { exports } = await WebAssembly.instantiate(module, {});
        if (generation !== this.generation || !this.isConnected) return;
        this.engine = new PsAnim({ exports });
      }
      if (generation !== this.generation || !this.isConnected) return;
      let text = this._source;
      if (this._entry) {
        text = this.engine.catalog(this._entry);
        if (text == null) throw new Error(`no catalog entry named ${this._entry}`);
      }
      if (!text) return;
      const params = this.params;
      let scale = Number(this.getAttribute("scale"));
      if (!(scale > 0)) {
        const probe = this.engine.open(text, { scale: 0.05, params });
        const want = (this.clientWidth || probe.width) * Math.min(devicePixelRatio || 1, 2);
        scale = Math.max(0.05, Math.min(2, want / probe.width));
      }
      this.scale = scale;
      this.meta = this.engine.open(text, { scale, params });
      this.canvas.width = this.meta.pixel_width;
      this.canvas.height = this.meta.pixel_height;
      this.err.textContent = "";
      this.t0 ??= performance.now();   // a reopen keeps the widget's clock
      this.lastFrame = -1;
      this.lastT = 0;
      this.lastRound = null;   // a new movie has no events from earlier rounds
      this.dirty = true;
      this.dispatchEvent(new CustomEvent("psanim-load", { detail: this.meta }));
      this.#wake();
    } catch (e) {
      this.#fail(e);
    }
  }

  #resized() {
    if (!this.meta || this.hasAttribute("scale")) return;
    const want = this.clientWidth * Math.min(devicePixelRatio || 1, 2);
    if (!(want > 0) || Math.abs(want - this.meta.pixel_width) < this.meta.pixel_width * 0.15) return;
    clearTimeout(this.resizeTimer);
    this.resizeTimer = setTimeout(() => this.#open(), 200);
  }

  #fail(e) {
    this.running = false;
    this.err.textContent = e.message;
    this.dispatchEvent(new CustomEvent("psanim-error", { detail: e instanceof PsAnimError ? e.info : { message: e.message } }));
  }

  #awake() {
    return this.meta && this.visible && document.visibilityState !== "hidden" && this.isConnected;
  }

  #wake() {
    if (this.running || !this.#awake()) return;
    this.running = true;
    // One loop at a time: a callback from an earlier loop (one queued
    // before a disconnect, say) finds its token stale and stops.
    const loop = (this.loop = (this.loop ?? 0) + 1);
    const tick = () => {
      if (!this.running || loop !== this.loop) return;
      if (!this.#awake()) {
        this.running = false;   // asleep until the observer or the tab wakes it
        return;
      }
      if (this.#step()) this.frameRequest = requestAnimationFrame(tick);
    };
    this.frameRequest = requestAnimationFrame(tick);
  }

  /** One animation frame: draw if there is something new. False on error. */
  #step() {
    const meta = this.meta;
    const fps = Math.min(Number(this.getAttribute("fps")) || meta.fps, meta.fps);
    const elapsed = this.#elapsed();
    const t = this.#movieTime(elapsed);
    this.#rollover(elapsed, t);
    // The frame number on the movie's clock: it stops changing when a held
    // movie reaches its end, and then only an input wakes it.
    const index = Math.floor(t * fps + 1e-9);
    if (index === this.lastFrame && !this.dirty) {
      this.frameSkips++;
      return true;
    }
    this.lastFrame = index;
    this.dirty = false;
    // A movie with state exists only at its own ticks; a pure one at any time.
    const at = meta.mode === "pure" ? t : Math.floor(t * meta.fps + 1e-9) / meta.fps;
    try {
      const e = this.engine;
      e.setSignal("clock", secondsSinceMidnight());
      e.setSignal("mousex", this.pointer.x);
      e.setSignal("mousey", this.pointer.y);
      e.setSignal("mousedown", this.pointer.down);
      e.setSignal("hover", this.pointer.hover);
      e.setSignal("key", this.pointer.key);
      for (const [name, value] of this.signals) e.setSignal(name, value);
      const started = performance.now();
      const frame = e.render(at);
      this.frameMs = performance.now() - started;
      this.framesDrawn++;
      this.ctx.putImageData(new ImageData(frame.data, frame.width, frame.height), 0, 0);
      this.err.textContent = "";
    } catch (e) {
      this.#fail(e);
      return false;
    }
    return true;
  }
}

customElements.define("psanim-widget", PsAnimWidget);
