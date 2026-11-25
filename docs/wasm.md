# psanim in the browser

Status: **working and exercised**. The engine compiles to
`wasm32-unknown-unknown` and renders movies in a page, with the same code
as the native build. Tested in Node 24 and in Chromium (through
Playwright); not yet tested in Firefox or Safari.

## Build

```sh
rustup target add wasm32-unknown-unknown     # once
scripts/build_wasm.sh                        # writes web/psanim.wasm
```

No wasm-bindgen, wasm-pack or bundler: `crates/psanim-wasm` exports a
small C ABI and the module has **no imports**, so it instantiates with
`WebAssembly.instantiate(bytes, {})`. Size: about 13 MB, 5.8 MB gzipped;
most of that is PSCat's embedded fonts and libraries.

## Use

```html
<script type="module" src="player.js"></script>
<psanim-player src="movie.ps" autoplay loop></psanim-player>
```

`<psanim-player>` (`web/player.js`) gives play, pause, a scrubber and an
error panel showing the engine's own message. Or drive the engine
directly (`web/psanim.js`):

```js
import { PsAnim } from "./psanim.js";
const engine = await PsAnim.load("psanim.wasm");
const meta = engine.open(source, { scale: 0.5, maxSteps: 50_000_000 });
const frame = engine.render(1.25);           // { width, height, data: Uint8ClampedArray }
ctx.putImageData(new ImageData(frame.data, frame.width, frame.height), 0, 0);
engine.setSignal("mousex", 320);             // read in the program with /mousex signal
engine.recordSignal("cpu", 12.5, 73.0);      // a reading at movie time 12.5, keeping the earlier ones
```

`setSignal` replaces a signal with a constant; `recordSignal` adds a
reading and keeps the earlier ones, so a live dashboard can draw the
signal's recent past (`signalseries`) and read it a little late to glide
between readings (`signalago`). `scripts/wasm_smoke.mjs` exercises it in
Node; it has not yet been run in a browser page.

Errors throw `PsAnimError` with `.info`, the same structured error the
CLI prints with `--json`.

The player sets three signals from the pointer before every frame:
`mousex`, `mousey` (canvas units, y up) and `mousedown`. A program that
reads them is interactive in the page and still renders offline (with the
signals from a file, or zero).

## Measured

On an Apple M-series laptop, Chromium, single thread:

| Movie | Size | Per frame |
|---|---|---|
| examples/hello.ps | 640x360 | 0.9 ms |
| gallery/epicycles | 640x360 | 14 ms (one glow layer) |

Native is about 1.5 to 2 times faster per thread and uses every core.

## What the browser build does not do

- **Threads.** One movie renders on the page's main thread. Run it in a
  Worker for heavy pieces (the module is self-contained and works there;
  not packaged yet).
- **Time limits.** There is no clock in this target, so only the step
  limit bounds a frame.
- **Files.** A program can `run` PSCat's built-in libraries and nothing
  else: no images from disk, no font packs.
- **Video export.** The page plays; it does not encode. (WebCodecs could;
  not built.)
- **Stateful and persistent movies** play, but seeking backward replays
  from a checkpoint and can stall on a long one.
