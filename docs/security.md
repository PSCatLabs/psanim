# Security: running programs you did not write

A psanim movie is a program in a Turing-complete language. This page says
what the engine does about that, and what it does not.

## What a program can never do

- **Write files.** The engine never enables PSCat's write roots, so
  `(path) (w) file`, `renamefile` and `deletefile` fail with
  `invalidfileaccess`. There is no option to turn this on.
- **Use the network or start processes.** The interpreter has no
  operators for either.
- **Read the wall clock.** `realtime` and `usertime` return the movie
  time.

## What is bounded when the host asks

| Risk | Control | Default (library) | `psanim --untrusted` |
|---|---|---|---|
| Reading files | `Options::read_roots` (PSCat read roots: `..` and symlink escapes refused) | open | built-in libraries only |
| | | CLI default: the program's own directory | |
| Endless or heavy setup | `Budget::setup` steps and time | none | 400 M steps, 30 s |
| Endless or heavy frame or tick | `Budget::frame` steps and time | none | 200 M steps, 20 s |
| Memory | `Limits::max_memory` (needs `pscat::TrackingAllocator`; the CLI installs it) | none | 1 GiB of heap growth |
| A render that must stop now | `Options::interrupt` flag | unset | (Ctrl-C kills the process) |
| Huge canvases | 8000 pixels a side, enforced by PSCat | always | always |
| Layer and scope depth | 16 layers, 256 nested time scopes | always | always |

A limit error cannot be caught by the program (`stopped` does not see
it), and the movie remains usable: the next frame starts from the
checkpoint as usual.

```sh
psanim render stranger.ps -o out.mp4 --untrusted
psanim check  stranger.ps --max-steps 5e7 --max-seconds 5 --max-memory-mb 512
```

## What is not guaranteed

Be precise about these before hosting psanim for others:

- **Limits are looked at between interpreter steps.** One long-running
  operator (a single enormous `fill`, a huge `image`) finishes before the
  step or time limit is checked. PSCat bounds the worst of these
  internally, but a single step can still take well over the nominal
  time. For a hard deadline, run the render in a process you can kill.
- **The memory limit counts the process heap**, not one movie's share.
  With several worker threads the cap is on the total. Each open layer is
  a full canvas (4 bytes a pixel); the limit of 16 layers bounds that at
  16 canvases per interpreter.
- **Total work is frames × per-frame budget.** A program may declare a
  long duration; bound `Options::duration` and the output size yourself.
- **In the browser** (`psanim-wasm`) there is no clock, so only the step
  limit applies, and the module shares a thread with the page: a frame
  that takes long blocks it. Use a step limit (the player sets 50 M) or a
  worker.
- **Not a sandbox.** These are resource and access controls inside one
  process, built on PSCat's (see PSCat's `SECURITY.md`). They have not had
  an adversarial audit. For hostile input, add an operating-system
  boundary: a container or a `wasm` runtime, with its own memory and time
  caps.

## For embedders

```rust
use psanim::{Budget, Movie, Options, Scale};

let options = Options {
    scale: Scale::Width(640),
    duration: Some(20.0),                 // do not let the program choose
    budget: Budget::untrusted(),
    read_roots: Some(vec![]),             // built-in libraries only
    ..Options::default()
};
let movie = Movie::load(untrusted_source, &options)?;
```
