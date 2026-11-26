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
//   thread   worker  render in a Worker of its own, off the page's main
//                    thread (the default where the browser has
//                    OffscreenCanvas and module Workers)
//            main    render on the page's main thread
//   timeout  the longest, in milliseconds, opening the movie or drawing one
//            frame may take in a Worker before the widget stops it and
//            shows an error (default 5000). On the main thread only the
//            engine's step budget stops a frame.
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
// frame number changes (at `fps`) or an input changed, and one frame at a
// time. It stops drawing while it is off screen or the tab is hidden. It
// dispatches psanim-load (detail: metadata) and psanim-error (detail: the
// structured error).

import { PsAnim, PsAnimError } from "./psanim.js";
import { openMovie, drawFrame } from "./widget-core.js";

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

/** The error a widget's timeout gives. */
function timedOut(e) {
  return e?.timedOut === true;
}

/** Whether this browser can render a widget in a Worker. */
function workersWork() {
  return typeof Worker === "function" && typeof OffscreenCanvas === "function" &&
    typeof HTMLCanvasElement.prototype.transferControlToOffscreen === "function";
}

/** A widget's engine on the page's main thread. */
class PageDriver {
  constructor(module, canvas) {
    this.thread = "main";
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d");
    this.ready = WebAssembly.instantiate(module, {}).then((instance) => {
      this.engine = new PsAnim(instance);
    });
  }
  async open(o) {
    await this.ready;
    return openMovie(this.engine, this.canvas, o);
  }
  async frame(at, signals) {
    return drawFrame(this.engine, this.ctx, at, signals);
  }
  event(name, t) { this.engine?.event(name, t); }
  clear(name) { this.engine?.clearEvents(name); }
  close() { this.engine?.close(); this.engine = null; }
}

/** A widget's engine in a Worker of its own, drawing into the widget's
 * canvas through an OffscreenCanvas. */
class WorkerDriver {
  constructor(module, canvas) {
    this.thread = "worker";
    this.worker = new Worker(new URL("widget-worker.js", import.meta.url), { type: "module" });
    this.calls = new Map();
    this.next = 0;
    this.worker.onmessage = (m) => {
      const r = m.data;
      const call = this.calls.get(r.id);
      if (!call) return;
      if (r.begun) {
        call.begin();
        return;
      }
      this.calls.delete(r.id);
      if (r.ok) call.resolve(r.value);
      else call.reject(r.ps ? new PsAnimError(r.info, r.text) : new Error(r.text));
    };
    this.worker.onerror = (e) => {
      e.preventDefault?.();
      this.#failAll(new Error(`the widget's worker failed: ${e.message || "it could not start"}`));
    };
    const off = canvas.transferControlToOffscreen();
    // Settles once the Worker has run its script and started the engine;
    // rejects if the browser could not run it (a module Worker it does not
    // support fails here, after the constructor returned). Timed from now,
    // not from when it begins: a script that never loads never begins.
    this.ready = this.#call({ type: "init", module, canvas: off }, [off]);
    delete this.ready.begun;
  }
  /** Post `msg`; a promise of its result, with `begun`, a promise that
   * settles when the Worker starts on it (after anything queued before). */
  #call(msg, transfer = []) {
    const id = ++this.next;
    let begin;
    const begun = new Promise((r) => { begin = r; });
    const done = new Promise((resolve, reject) => {
      this.calls.set(id, { resolve, reject, begin });
      this.worker.postMessage({ ...msg, id }, transfer);
    });
    done.begun = begun;
    return done;
  }
  #failAll(err) {
    for (const call of this.calls.values()) call.reject(err);
    this.calls.clear();
  }
  open(o) { return this.#call({ type: "open", ...o }); }
  frame(at, signals) { return this.#call({ type: "frame", at, signals }); }
  event(name, t) { this.worker.postMessage({ type: "event", name, t }); }
  clear(name) { this.worker.postMessage({ type: "clear", name }); }
  close() {
    this.worker.terminate();
    // What was waiting on it was cut off by the widget itself (a timeout,
    // a disconnect): not an error of its own to show.
    const e = new Error("the widget was closed");
    e.closed = true;
    this.#failAll(e);
  }
}

/** Local seconds since midnight, with the fraction. */
export function secondsSinceMidnight(d = new Date()) {
  return d.getHours() * 3600 + d.getMinutes() * 60 + d.getSeconds() + d.getMilliseconds() / 1000;
}

const css = `
:host { display: inline-block; position: relative; line-height: 0; width: 300px; }
/* The host's width comes from CSS (300px unless the page says otherwise),
   never from the canvas: the canvas's pixel buffer is sized from the host's
   width times the device pixel ratio, so letting the canvas size the host
   would grow it on every resize. */
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
    this.err = root.querySelector(".err");
    this.driver = null;             // the engine, in the page or in a Worker
    this.meta = null;
    this.signals = new Map();       // the page's own signals, kept across reopens
    this.eventNames = new Set();    // event tracks used so far
    this.pointer = { x: 0, y: 0, down: 0, hover: 0, key: 0 };
    this.dirty = true;              // an input changed since the last frame
    this.inflight = false;          // a frame is being drawn
    this.lastFrame = -1;
    this.lastT = 0;
    this.visible = true;
    this.running = false;
    this.frameMs = 0;               // cost of the last frame drawn
    this.framesDrawn = 0;
    this.frameSkips = 0;            // animation frames with nothing new to draw
    this.#wire();
  }

  /** Where the engine runs: "worker", "main", or null before it opens. */
  get thread() {
    return this.driver?.thread ?? null;
  }

  // On the host, not the canvas: the canvas is replaced whenever a new
  // engine is made (an OffscreenCanvas can be taken from a canvas once).
  #wire() {
    const at = (e) => {
      if (!this.meta) return;
      const r = this.canvas.getBoundingClientRect();
      this.pointer.x = ((e.clientX - r.left) / r.width) * this.meta.width;
      this.pointer.y = (1 - (e.clientY - r.top) / r.height) * this.meta.height;
      this.dirty = true;
    };
    this.addEventListener("pointermove", at);
    this.addEventListener("pointerenter", (e) => { this.pointer.hover = 1; at(e); });
    this.addEventListener("pointerleave", (e) => { this.pointer.hover = 0; this.pointer.down = 0; at(e); });
    this.addEventListener("pointerdown", (e) => {
      this.pointer.down = 1;
      at(e);
      this.event("click");
    });
    this.addEventListener("pointerup", (e) => { this.pointer.down = 0; at(e); });
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
    if (!this.driver && (this._source || this._entry)) this.#open();
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
    this.#dropDriver();
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
    if (!this.meta || !this.driver) return;
    this.eventNames.add(name);
    // Wrap first, so clearing the last time round cannot take this event.
    const elapsed = this.#elapsed();
    const t = this.#movieTime(elapsed);
    this.#rollover(elapsed, t);
    this.driver.event(name, t);
    this.dirty = true;
  }

  /** The loop wrapped since the clock was last read, once or many times
   * (a hidden widget can sleep through several): events from earlier time
   * rounds are in this one's future, so forget them. Counted on the
   * unwrapped clock. Call before adding an event. */
  #rollover(elapsed, t) {
    const round = this.#timeMode() === "loop" ? Math.floor(elapsed / this.meta.duration) : 0;
    if (this.lastRound != null && round !== this.lastRound) {
      for (const name of this.eventNames) this.driver.clear(name);
      // A new round is a new picture even when the frame number is the
      // same (a one-frame loop): the clock signal and the events changed.
      this.dirty = true;
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

  #timeout() {
    const ms = Number(this.getAttribute("timeout"));
    return ms > 0 ? ms : 5000;
  }

  /** `work`, or an error once it has taken longer than the timeout. Only a
   * Worker can be stopped: the page's own thread cannot be interrupted.
   * The time counts from when the Worker starts on it (`work.begun`), not
   * while it waits behind an earlier call: that call has its own timeout. */
  #guard(work, what, driver = this.driver) {
    if (driver?.thread !== "worker") return work;
    const ms = this.#timeout();
    let timer;
    const late = new Promise((_, reject) => {
      const arm = () => {
        timer = setTimeout(() => {
          const e = new Error(
            `${what} took longer than ${ms} ms, so the widget was stopped (its timeout attribute sets the limit)`);
          e.timedOut = true;
          reject(e);
        }, ms);
      };
      if (work.begun) work.begun.then(() => { if (!settled) arm(); });
      else arm();
    });
    let settled = false;
    return Promise.race([work, late]).finally(() => {
      settled = true;
      clearTimeout(timer);
    });
  }

  /** Close the engine and put a fresh canvas in place of the one it had. */
  #dropDriver() {
    this.driver?.close();
    this.driver = null;
    this.inflight = false;
  }

  /** Put a fresh canvas in place of the current one: a canvas handed to
   * an OffscreenCanvas cannot be drawn on from the page again. */
  #freshCanvas() {
    const fresh = document.createElement("canvas");
    fresh.setAttribute("part", "canvas");
    this.canvas.replaceWith(fresh);
    this.canvas = fresh;
    return fresh;
  }

  /** An engine in a Worker if this browser can run one, else in the page.
   * A Worker that fails to start (synchronously, or once it runs its
   * script) is dropped for the page. Null when a newer open (or a
   * disconnect) superseded `generation` while the Worker started: the
   * canvas is that open's by then, and is left alone. */
  async #newDriver(module, generation) {
    if (this.getAttribute("thread") !== "main" && workersWork()) {
      let driver = null;
      try {
        driver = new WorkerDriver(module, this.#freshCanvas());
        await this.#guard(driver.ready, "starting the widget's worker", driver);
      } catch {
        driver?.close();
        driver = null;
      }
      if (generation !== this.generation || !this.isConnected) {
        driver?.close();
        return null;
      }
      if (driver) return driver;
    }
    return new PageDriver(module, this.#freshCanvas());
  }

  async #open() {
    const generation = (this.generation = (this.generation ?? 0) + 1);
    let openedOn = null; // the engine this open was sent to
    try {
      if (!this.driver) {
        const url = this.getAttribute("wasm") ?? new URL("psanim.wasm", import.meta.url).href;
        const module = await compiled(url);
        if (generation !== this.generation || !this.isConnected) return;
        // Another open may have made the engine while this one waited.
        if (!this.driver) {
          const driver = await this.#newDriver(module, generation);
          // A newer open (or a disconnect) superseded this one while the
          // Worker started: that one makes its own engine and canvas.
          if (!driver) return;
          this.driver = driver;
        }
      }
      const text = this._entry ? null : this._source;
      if (!this._entry && !text) return;
      let scale = Number(this.getAttribute("scale"));
      const want = (this.clientWidth || 0) * Math.min(devicePixelRatio || 1, 2);
      // No frames while the movie opens: one sent now would be drawn after
      // the open, from the old movie's clock, and its timeout would count
      // the open's time as its own.
      this.opening = generation;
      openedOn = this.driver;
      const opened = await this.#guard(this.driver.open({
        text, entry: this._entry, params: this.params, scale: scale > 0 ? scale : null, want,
      }), "opening the movie");
      if (generation !== this.generation || !this.isConnected) return;
      this.scale = opened.scale;
      this.meta = opened.meta;
      // The canvas's pixel size is set where it is drawn; its shape here.
      this.canvas.style.aspectRatio = `${this.meta.pixel_width} / ${this.meta.pixel_height}`;
      this.err.textContent = "";
      this.t0 ??= performance.now();   // a reopen keeps the widget's clock
      this.lastFrame = -1;
      this.lastT = 0;
      this.lastRound = null;   // a new movie has no events from earlier rounds
      this.dirty = true;
      this.dispatchEvent(new CustomEvent("psanim-load", { detail: this.meta }));
      this.#wake();
    } catch (e) {
      if (e?.closed) return;
      // A timeout stops the Worker that was stuck, even when a newer open
      // has come since: that open waits behind this one and never begins.
      if (timedOut(e) && openedOn && this.driver === openedOn) {
        this.#fail(e);
        return;
      }
      if (generation !== this.generation) return;
      this.#fail(e);
    } finally {
      if (this.opening === generation) this.opening = null;
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
    // A Worker that ran over its time is stopped for good; the next open
    // (a new src, entry or params) starts another.
    if (timedOut(e)) {
      this.#dropDriver();
      this.meta = null;
    }
    this.err.textContent = e.message;
    this.dispatchEvent(new CustomEvent("psanim-error", { detail: e instanceof PsAnimError ? e.info : { message: e.message } }));
  }

  #awake() {
    return this.meta && this.driver && this.visible && document.visibilityState !== "hidden" && this.isConnected;
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
      this.#step();
      if (this.running && loop === this.loop) this.frameRequest = requestAnimationFrame(tick);
    };
    this.frameRequest = requestAnimationFrame(tick);
  }

  /** One animation frame: start drawing if there is something new and no
   * frame is being drawn already. */
  #step() {
    // A frame still being drawn, or the movie being opened: wait, leaving
    // what changed meanwhile for the next frame.
    if (this.inflight || this.opening != null) return;
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
      return;
    }
    this.lastFrame = index;
    this.dirty = false;
    // A movie with state exists only at its own ticks; a pure one at any time.
    const at = meta.mode === "pure" ? t : Math.floor(t * meta.fps + 1e-9) / meta.fps;
    const signals = [
      ["clock", secondsSinceMidnight()],
      ["mousex", this.pointer.x],
      ["mousey", this.pointer.y],
      ["mousedown", this.pointer.down],
      ["hover", this.pointer.hover],
      ["key", this.pointer.key],
      ...this.signals,
    ];
    const generation = this.generation;
    this.inflight = true;
    const driver = this.driver;
    this.#guard(driver.frame(at, signals), "a frame")
      .then((r) => {
        if (generation !== this.generation) return;
        this.frameMs = r.ms;
        this.framesDrawn++;
        this.err.textContent = "";
      })
      .catch((e) => {
        // Decided by the engine, not the animation loop: a widget that went
        // off screen and back has a new loop, and its Worker is still stuck
        // on this frame. A timeout stops that Worker whatever happened
        // since; another error matters only for the movie still open.
        if (this.driver !== driver) return;
        if (timedOut(e) || generation === this.generation) this.#fail(e);
      })
      .finally(() => {
        if (this.driver === driver) this.inflight = false;
      });
  }
}

customElements.define("psanim-widget", PsAnimWidget);
