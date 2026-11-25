// Smoke test of the browser build in Node: load, render, signals, errors, limits.
//   scripts/build_wasm.sh && node scripts/wasm_smoke.mjs
import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { PsAnim } from "../web/psanim.js";

const bytes = readFileSync(new URL("../web/psanim.wasm", import.meta.url));
const module = await WebAssembly.compile(bytes);
assert.deepEqual(WebAssembly.Module.imports(module), [], "the module must have no imports");
const engine = await PsAnim.fromBytes(bytes);

const hello = readFileSync(new URL("../examples/hello.ps", import.meta.url), "utf8");
const meta = engine.open(hello);
assert.equal(meta.pixel_width, 640);
assert.equal(meta.mode, "pure");
let started = performance.now();
const first = Uint8Array.from(engine.render(2).data);
for (let k = 0; k < 60; k++) engine.render(k / 15);
const perFrame = (performance.now() - started) / 61;
assert.deepEqual(Uint8Array.from(engine.render(2).data), first, "a frame is a function of its time");
assert.ok(first.some((b) => b > 200), "the frame has something bright in it");

engine.open("<< /Size [16 8] >> animation /scene { /mousex signal 16 div 0 0 setrgbcolor 0 0 16 8 rectfill } def");
engine.setSignal("mousex", 8);
assert.equal(engine.render(0).data[0], 128, "signals reach the program");
engine.open("<< /Size [16 8] >> animation /scene { /cpu 0.5 signalago 0 0 setrgbcolor 0 0 16 8 rectfill } def");
engine.recordSignal("cpu", 0, 0);
engine.recordSignal("cpu", 1, 1);
assert.ok(Math.abs(engine.render(1).data[0] - 128) <= 1, "recorded signals keep their past");

engine.open("<< /Size [8 8] >> animation /scene { 1 (a) add } def");
assert.throws(() => engine.render(0), (e) => e.name === "PsAnimError" && e.info.kind === "typecheck" && e.info.backtrace[0].procedure === "scene");
engine.open("<< /Size [8 8] >> animation /scene { { } loop } def", { maxSteps: 100000 });
assert.throws(() => engine.render(0), (e) => e.info.kind === "limitexceeded" && e.info.limit === "steps");
assert.throws(() => engine.open("<< /Size [8 8] >> animation (/etc/hosts) (r) file /scene { } def"), (e) => e.info.kind === "invalidfileaccess" || e.info.kind === "undefinedfilename");

console.log(`wasm ok: ${(bytes.length / 1e6).toFixed(1)} MB, hello.ps at ${perFrame.toFixed(2)} ms/frame`);
