# Using psanim from Rust

The `psanim` crate is the engine: load a program, ask for frames. It has
no platform dependencies beyond PSCat's library and does no I/O of its
own, so it embeds in a server, a GUI, a test, or a browser (through
`psanim-wasm`).

```toml
[dependencies]
psanim = { path = "crates/psanim" }      # see docs/pscat-integration.md for the PSCat side
```

## Load and render

```rust
use psanim::{Movie, Options, Scale};

let source = std::fs::read_to_string("movie.ps")?;
let mut movie = Movie::load(&source, &Options {
    scale: Scale::Width(1280),            // output width in pixels; the canvas stays in its own units
    ..Options::default()
})?;

let meta = movie.meta();                  // size, fps, duration, frames, mode, ...
println!("{} frames, {:?}", meta.frames, meta.mode);

let frame = movie.render_frame(120)?;     // any frame, any order
std::fs::write("f120.png", frame.to_png())?;

let frame = movie.render_at(2.5)?;        // or any time, not only frame times
let rgba: &[u8] = frame.premultiplied;    // borrowed until the next render
```

`Movie` owns one PSCat interpreter and is not `Send`: one per thread.

## Options

| Field | What it does |
|---|---|
| `scale` | `Factor(f)`, `Width(px)` or `Height(px)`: output pixels per canvas unit |
| `fps`, `duration`, `seed` | override what the program declared |
| `params` | values for the program's `param` (`Param::Number`, `Param::Text`) |
| `signals` | a `Signals` set: sampled tracks and event times (`Signals::from_csv`) |
| `budget` | `Budget { setup, frame }` of PSCat `Limits`; `Budget::untrusted()` is a preset |
| `motion_blur` | `Some(Some(MotionBlur { samples, shutter }))` to force, `Some(None)` to disable |
| `read_roots` | directories the program may read; `Some(vec![])` for built-in libraries only |
| `interrupt` | an `Arc<AtomicBool>` another thread raises to stop a render |

## Many frames, many cores

```rust
use psanim::render::render_frames;

let frames: Vec<u64> = (0..meta.frames).collect();
render_frames(&source, &options, &frames, 0 /* one thread per core */, |f| {
    // called in order, on this thread
    encoder.write(&f.premultiplied).map_err(|e| e.to_string())
})?;
```

Each worker loads its own `Movie`. A stateful or persistent movie renders
on one thread whatever is asked (`render::effective_threads`).

## A live host

Nothing assumes an offline loop. Drive the clock yourself and feed inputs
before each frame:

```rust
movie.set_signal("mousex", pointer.x);
movie.set_signal("level", audio.rms());
let frame = movie.render_at(clock.seconds())?;
```

The program reads them with `/mousex signal`. The same program renders
offline with those signals loaded from a file.

## Errors

`psanim::Error` is data: `phase` (load, setup, frame *k*, step *n*),
`kind` (a PostScript error name, or `declaration`, `options`,
`stackleft`, `unclosedlayer`), `command`, `line` (setup only),
`backtrace` (the program's procedures, innermost first, with the failing
element marked), `stack` (the operands it was looking at), `hint`.
`Display` prints all of it; `to_json()` gives it to tools. After an
error the movie is still usable: the next frame starts from the
checkpoint.

## Debugging a program from the host

`Movie::take_output()` returns what the program printed with `=`, `==`,
`print` and `pstack` since the last call. Output is kept to 64 KB per
call; past that the text ends with a line `[output truncated at 64 KB]`.
`Movie::take_printed()` returns the same text with a `truncated` flag.
`Movie::last_frame_steps()` is the interpreter's step count for the last
frame when a step budget is set.

## Building blocks

Usable without a `Movie`: `psanim::ease` (easing curves by name),
`psanim::pathmath` (flatten, measure, trim, resample paths),
`psanim::raster` (composite, blur, flatten, downscale, frame statistics
and differences), `psanim::signals`, `psanim::sheet` (contact sheets),
`psanim::api` (the vocabulary as a table, Markdown or JSON).
