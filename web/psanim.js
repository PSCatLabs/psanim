// psanim.js: load the psanim WebAssembly module and render movies with it.
//
//   import { PsAnim } from "./psanim.js";
//   const engine = await PsAnim.load("psanim.wasm");
//   const meta = engine.open(source, { scale: 0.5 });   // throws PsAnimError
//   const frame = engine.render(1.25);                  // { width, height, data }
//   ctx.putImageData(new ImageData(frame.data, frame.width, frame.height), 0, 0);
//
// The module is the same engine as the native `psanim` binary; there is no
// generated glue, only the C ABI in crates/psanim-wasm.

export class PsAnimError extends Error {
  /** @param {object} info the engine's structured error */
  constructor(info, text) {
    super(text || info.message);
    this.name = "PsAnimError";
    this.info = info;
  }
}

export class PsAnim {
  constructor(instance) {
    this.x = instance.exports;
    this.meta = null;
    this.encoder = new TextEncoder();
    this.decoder = new TextDecoder();
  }

  /** Fetch and instantiate the module. */
  static async load(url = "psanim.wasm") {
    const response = await fetch(url);
    if (!response.ok) throw new Error(`cannot fetch ${url}: ${response.status}`);
    const bytes = await response.arrayBuffer();
    return PsAnim.fromBytes(bytes);
  }

  /** Instantiate from the module's bytes (Node, or a page that has them). */
  static async fromBytes(bytes) {
    const { instance } = await WebAssembly.instantiate(bytes, {});
    return new PsAnim(instance);
  }

  #string(ptr, len) {
    return this.decoder.decode(new Uint8Array(this.x.memory.buffer, ptr, len));
  }

  #withString(text, f) {
    const bytes = this.encoder.encode(text);
    const ptr = this.x.pa_alloc(bytes.length);
    new Uint8Array(this.x.memory.buffer, ptr, bytes.length).set(bytes);
    try {
      return f(ptr, bytes.length);
    } finally {
      this.x.pa_free(ptr, bytes.length);
    }
  }

  #error() {
    const info = JSON.parse(this.#string(this.x.pa_error_ptr(), this.x.pa_error_len()));
    const text = this.#string(this.x.pa_error_text_ptr(), this.x.pa_error_text_len());
    return new PsAnimError(info, text);
  }

  /**
   * Load a program. Returns its metadata (the same object as
   * `psanim info --json`'s `movie`). Throws PsAnimError.
   * @param {string} source
   * @param {{scale?: number, maxSteps?: number}} [options]
   */
  open(source, { scale = 1, maxSteps = 50_000_000 } = {}) {
    const status = this.#withString(source, (p, n) => this.x.pa_load(p, n, scale, maxSteps));
    if (status !== 0) {
      this.meta = null;
      throw this.#error();
    }
    this.meta = JSON.parse(this.#string(this.x.pa_meta_ptr(), this.x.pa_meta_len()));
    return this.meta;
  }

  /**
   * Render the moment `time` seconds in. The returned `data` is a view of
   * the module's memory, valid until the next call: copy it or draw it now.
   */
  render(time) {
    if (this.x.pa_render(time) !== 0) throw this.#error();
    const width = this.x.pa_width();
    const height = this.x.pa_height();
    const data = new Uint8ClampedArray(this.x.memory.buffer, this.x.pa_pixels_ptr(), width * height * 4);
    return { width, height, data };
  }

  /** Set a signal the program reads with `/name signal`. */
  setSignal(name, value) {
    this.#withString(name, (p, n) => this.x.pa_set_signal(p, n, value));
  }

  /** Add a reading of a signal taken at movie time `t`, keeping the earlier
   *  ones, so the program can read its recent past (`signalago`,
   *  `signalseries`) and draw smoothly between readings. */
  recordSignal(name, t, value) {
    this.#withString(name, (p, n) => this.x.pa_record_signal(p, n, t, value));
  }

  /** Interpreter steps the last frame took. */
  get lastSteps() {
    return this.x.pa_last_steps();
  }

  close() {
    this.x.pa_unload();
    this.meta = null;
  }
}
