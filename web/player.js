// <psanim-player>: a live player for a psanim program.
//
//   <script type="module" src="player.js"></script>
//   <psanim-player src="hello.ps" autoplay loop></psanim-player>
//
// Attributes: src (URL of a .ps program), autoplay, loop, scale (pixels per
// canvas unit; default fits the element's width), wasm (URL of psanim.wasm;
// default beside this file). Or set the program text with `player.source =
// "..."`. The element renders with the real engine, compiled to
// WebAssembly: the frame on screen is computed in the page, at the time on
// the scrubber. It dispatches `psanim-load` (detail: metadata) and
// `psanim-error` (detail: the structured error).
//
// While the pointer is over the canvas the program can read it as the
// signals mousex, mousey (canvas units, y up) and mousedown (0 or 1).

import { PsAnim, PsAnimError } from "./psanim.js";

let modulePromise = null;
function compiled(url) {
  modulePromise ??= fetch(url).then((r) => {
    if (!r.ok) throw new Error(`cannot fetch ${url}: ${r.status}`);
    return r.arrayBuffer();
  }).then((bytes) => WebAssembly.compile(bytes));
  return modulePromise;
}

const css = `
:host { display: block; position: relative; background: #0b0d12; color: #e8e6df;
        font: 13px/1.4 ui-sans-serif, system-ui, sans-serif; border-radius: 6px; overflow: hidden; }
canvas { display: block; width: 100%; height: auto; background: #000; }
.bar { display: flex; align-items: center; gap: 10px; padding: 7px 10px; background: #12151c; }
button { all: unset; cursor: pointer; width: 26px; height: 26px; display: grid; place-items: center;
         border-radius: 4px; color: inherit; }
button:hover { background: #232836; }
button:focus-visible, input:focus-visible { outline: 2px solid #ffc94d; outline-offset: 1px; }
input[type=range] { flex: 1; accent-color: #ffc94d; min-width: 60px; }
.time { font-variant-numeric: tabular-nums; color: #9aa3b5; white-space: nowrap; }
.err { position: absolute; left: 0; right: 0; top: 0; margin: 0; padding: 10px 12px; max-height: 60%;
       overflow: auto; background: rgba(60, 12, 16, 0.94); color: #ffd9d4; white-space: pre-wrap;
       font: 12px/1.45 ui-monospace, Menlo, monospace; }
.err:empty { display: none; }
`;

export class PsAnimPlayer extends HTMLElement {
  static observedAttributes = ["src"];

  constructor() {
    super();
    const root = this.attachShadow({ mode: "open" });
    root.innerHTML = `<style>${css}</style>
      <canvas part="canvas"></canvas>
      <pre class="err" role="alert"></pre>
      <div class="bar" part="controls">
        <button class="play" aria-label="Play"></button>
        <input type="range" min="0" max="1" step="0.001" value="0" aria-label="Time">
        <span class="time">0.00 / 0.00 s</span>
      </div>`;
    this.canvas = root.querySelector("canvas");
    this.ctx = this.canvas.getContext("2d");
    this.err = root.querySelector(".err");
    this.playButton = root.querySelector(".play");
    this.slider = root.querySelector("input");
    this.label = root.querySelector(".time");
    this.engine = null;
    this.meta = null;
    this.time = 0;
    this.playing = false;
    this.lastTick = 0;
    this.pointer = { x: 0, y: 0, down: 0 };
    this.frameMs = 0;

    this.playButton.addEventListener("click", () => (this.playing ? this.pause() : this.play()));
    this.slider.addEventListener("input", () => {
      this.pause();
      this.seek(Number(this.slider.value) * (this.meta?.duration ?? 0));
    });
    const move = (e) => {
      if (!this.meta) return;
      const r = this.canvas.getBoundingClientRect();
      this.pointer.x = ((e.clientX - r.left) / r.width) * this.meta.width;
      this.pointer.y = (1 - (e.clientY - r.top) / r.height) * this.meta.height;
      if (!this.playing) this.draw();
    };
    this.canvas.addEventListener("pointermove", move);
    this.canvas.addEventListener("pointerdown", (e) => { this.pointer.down = 1; move(e); });
    this.canvas.addEventListener("pointerup", (e) => { this.pointer.down = 0; move(e); });
    this.#icon();
  }

  #icon() {
    this.playButton.innerHTML = this.playing
      ? `<svg width="14" height="14" viewBox="0 0 14 14"><rect x="2" y="1" width="4" height="12" fill="currentColor"/><rect x="8" y="1" width="4" height="12" fill="currentColor"/></svg>`
      : `<svg width="14" height="14" viewBox="0 0 14 14"><path d="M3 1l10 6-10 6z" fill="currentColor"/></svg>`;
    this.playButton.setAttribute("aria-label", this.playing ? "Pause" : "Play");
  }

  async attributeChangedCallback(name, _old, value) {
    if (name === "src" && value) {
      try {
        const r = await fetch(value);
        if (!r.ok) throw new Error(`cannot fetch ${value}: ${r.status}`);
        this.source = await r.text();
      } catch (e) {
        this.#fail(e);
      }
    }
  }

  /** The program text. Setting it loads the movie. */
  get source() {
    return this._source ?? "";
  }

  set source(text) {
    this._source = text;
    this.#open();
  }

  async #open() {
    const text = this._source;
    try {
      if (!this.engine) {
        const url = this.getAttribute("wasm") ?? new URL("psanim.wasm", import.meta.url).href;
        const module = await compiled(url);
        const { exports } = await WebAssembly.instantiate(module, {});
        this.engine = new PsAnim({ exports });
      }
      if (text !== this._source) return; // a newer program arrived meanwhile
      // Fit the element: at most one device pixel per CSS pixel times dpr.
      let scale = Number(this.getAttribute("scale"));
      if (!(scale > 0)) {
        const probe = this.engine.open(text, { scale: 0.05 });
        const want = Math.min(this.clientWidth || probe.width, probe.width) * Math.min(devicePixelRatio || 1, 2);
        scale = Math.max(0.1, Math.min(2, want / probe.width));
      }
      this.meta = this.engine.open(text, { scale });
      this.canvas.width = this.meta.pixel_width;
      this.canvas.height = this.meta.pixel_height;
      this.err.textContent = "";
      this.time = Math.min(this.time, this.meta.duration);
      this.dispatchEvent(new CustomEvent("psanim-load", { detail: this.meta }));
      this.draw();
      if (this.hasAttribute("autoplay")) this.play();
    } catch (e) {
      this.#fail(e);
    }
  }

  #fail(e) {
    this.pause();
    this.err.textContent = e.message;
    this.dispatchEvent(new CustomEvent("psanim-error", { detail: e instanceof PsAnimError ? e.info : { message: e.message } }));
  }

  /** Draw the current time. Returns false if the program failed. */
  draw() {
    if (!this.meta) return false;
    // A movie with state exists only at its own ticks; a pure one at any time.
    const t = this.meta.mode === "pure" ? this.time : Math.floor(this.time * this.meta.fps + 1e-9) / this.meta.fps;
    try {
      this.engine.setSignal("mousex", this.pointer.x);
      this.engine.setSignal("mousey", this.pointer.y);
      this.engine.setSignal("mousedown", this.pointer.down);
      const started = performance.now();
      const frame = this.engine.render(Math.min(t, this.meta.duration - 1e-6));
      this.frameMs = performance.now() - started;
      this.ctx.putImageData(new ImageData(frame.data, frame.width, frame.height), 0, 0);
      this.err.textContent = "";
    } catch (e) {
      this.#fail(e);
      return false;
    }
    this.slider.value = String(this.time / this.meta.duration);
    this.label.textContent = `${this.time.toFixed(2)} / ${this.meta.duration.toFixed(2)} s`;
    return true;
  }

  seek(time) {
    this.time = Math.max(0, Math.min(time, this.meta?.duration ?? 0));
    this.draw();
  }

  play() {
    if (!this.meta || this.playing) return;
    if (this.time >= this.meta.duration - 1e-3) this.time = 0;
    this.playing = true;
    this.#icon();
    this.lastTick = performance.now();
    const tick = (now) => {
      if (!this.playing) return;
      this.time += (now - this.lastTick) / 1000;
      this.lastTick = now;
      if (this.time >= this.meta.duration) {
        if (this.hasAttribute("loop") || this.meta.loop) {
          this.time %= this.meta.duration;
        } else {
          this.time = this.meta.duration;
          this.draw();
          this.pause();
          return;
        }
      }
      if (this.draw()) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }

  pause() {
    this.playing = false;
    this.#icon();
  }

  disconnectedCallback() {
    this.pause();
  }
}

customElements.define("psanim-player", PsAnimPlayer);
