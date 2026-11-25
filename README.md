# psanim

Programmable animation in PostScript, on the [PSCat](https://github.com/PSCatLabs/pscat) graphics engine.

A movie is one PostScript file: a declaration, and a `scene` procedure
that draws the frame for the current moment. psanim runs it on the PSCat
interpreter (as a library, in process) and gives you frames, contact
sheets and video. There is no timeline and no scene graph. **A frame is a
function of its time**, so any frame renders on its own, in any order, on
every core.

```postscript
%!PS
<< /Size [640 360] /Duration 4 /FPS 30 /Background [0.06 0.07 0.10] >> animation

/scene {
  % A ring of dots, each starting a little after the last.
  0 1 11 {
    /i exch def
    i 0.08 {
      gsave
        320 180 translate
        i 30 mul  0 2 /cubic-inout tween 360 mul add rotate
        0.3 1.2 /back-out tween 110 mul 0 translate
        i 12 div 0.7 1 sethsbcolor
        0 0  0.3 1.0 /spring tween 14 mul  0 360 arc fill
      grestore
    } staggered
  } for
  % The title draws itself on, then fills.
  /Helvetica-Bold findfont 54 scalefont setfont
  320 162 moveto (psanim) dup stringwidth pop 2 div neg 0 rmoveto
  true charpath
  gsave 1.2 2.6 /cubic-out tween { 1 setgray fill } faded grestore
  0 0.6 2.2 /cubic-inout tween trimpath
  1 0.85 0.3 setrgbcolor 1.5 setlinewidth stroke
} def
```

```sh
psanim check  examples/hello.ps                 # loads it, samples frames, reports problems
psanim sheet  examples/hello.ps -o sheet.png    # twelve frames on one image
psanim frame  examples/hello.ps --at 2.4 -o still.png
psanim render examples/hello.ps -o hello.mp4    # 120 frames in about 0.1 s
```

That is the core vocabulary. Most films are written a level up, with the
libraries: draw each thing at rest, in a box, in theme colours, then say
when it arrives and by what motion.

```postscript
%!PS
<< /Size [640 360] /Duration 6 /FPS 30 >> animation
/pkdiagram use
/slots canvasbox 10 vmin inset 0.35 splity exch pop 3 14 vmin cols def
/scene {
  ground
  /display 30 face /fg ink 36 300 moveto (How a frame is made) /glyphs 0.03 /mask-up kinetic
  {
    0.9 wait
    0.6 { /pop { (program) slots 0 get /surface node } entering } then
    0.6 { /pop { (clock) slots 1 get /surface node } entering
          slots 0 get slots 1 get << /Draw 0.1 0.6 /cubic-inout tween >> edge } then
    0.6 { /pop { (picture) slots 2 get << /Fill /accent /Ink /bg >> node } entering
          slots 1 get slots 2 get << /Draw 0.1 0.6 /cubic-inout tween /Pulse 1.2 2 pulsetrain >> edge } then
  } script
} def
```

## What is here

- **An engine** (`crates/psanim`): load a program, render any frame.
  Pure movies, stateful simulations with a `step` procedure, persistent
  canvases for paintings; motion blur; parameters, signals and events
  from outside; budgets and file confinement for programs you did not
  write.
- **A core vocabulary of about 110 words** added to PostScript: a clock (`now`),
  time scopes that re-base it for a procedure (`during`, `delayed`,
  `looped`, `staggered`), eased values (`tween`, `between`,
  `keyframes`), arc-length path operations (`trimpath`, `pathpoint`,
  `morphpath`), layers (`faded`, `glowing`, `blurred`), reels with
  transitions, text helpers. One table in the source generates the
  reference, `psanim api`, and the agent skill.
- **A CLI** (`crates/psanim-cli`): `info`, `check`, `frame`, `sheet`,
  `render` (MP4, WebM, GIF, PNG sequence), `bench`, `api`. Everything has
  `--json`.
- **A browser build** (`crates/psanim-wasm`, `web/`): the same engine in
  WebAssembly, a `<psanim-player>` element, a playground.
- **Libraries for saying what you mean** (`/pkmotion use` ...): boxes and
  themes, named motions that work on any drawing (`2 /pop { ... }
  enters`), kinetic type, diagrams with flowing edges, code panels and
  animated diffs, charts that move between datasets, brush marks. About
  105 more words in seven libraries, written in PostScript.
  [Design](docs/vocabulary-design.md), [evaluation](docs/vocabulary-evaluation.md).
- **A gallery** (`gallery/`): twenty-two films, each one file. [Index, with stills](docs/gallery.md).
- **An agent skill** (`skills/psanim/`) and a documentation site built
  from the same text.
- **A host API for PSCat**, on a feature branch: what PSCat needed to
  become a library other applications build on.
  [Details](docs/pscat-integration.md).

## Build

psanim builds against a PSCat feature branch checked out beside it.

```sh
# 1. PSCat's feature branch, checked out beside this repository
git clone -b feature/psanim-library-api https://github.com/PSCatLabs/pscat ../pscat-psanim-api

# 2. psanim
cargo build --release                # target/release/psanim
cargo test  --release                # about 140 tests; renders thousands of frames

# 3. optional
scripts/build_wasm.sh                # web/psanim.wasm (needs: rustup target add wasm32-unknown-unknown)
scripts/build_gallery.sh             # render every gallery film into site/gallery/
scripts/build_site.py                # the documentation site, into site/
python3 -m http.server -d site 8000
```

Video output needs `ffmpeg` on PATH. Everything else is Rust and the
standard library of Python 3 (for the site).

## Documentation

| | |
|---|---|
| [Writing an animation](docs/authoring.md) | the tutorial |
| [Staging](skills/psanim/references/stage.md), [type](skills/psanim/references/type.md), [diagrams, code, charts](skills/psanim/references/explain.md), [ink](skills/psanim/references/ink.md), [storytelling](skills/psanim/references/storytelling.md) | the libraries, by what you want to make; every example is run by the tests and rendered on the site |
| [Patterns](skills/psanim/references/patterns.md), [craft](skills/psanim/references/craft.md), [pitfalls](skills/psanim/references/pitfalls.md), [modes](skills/psanim/references/modes.md) | the chapters, shared with the agent skill; every example is run by the tests |
| [Vocabulary](skills/psanim/references/vocabulary.md), [library vocabulary](skills/psanim/references/vocabulary-libraries.md) | every word (generated) |
| [Command line](docs/cli.md) · [Rust API](docs/rust.md) · [Browser](docs/wasm.md) | reference |
| [Architecture](docs/architecture.md) | the model, the decisions, what was rejected |
| [The expressive vocabulary](docs/vocabulary-design.md) · [its evaluation](docs/vocabulary-evaluation.md) | the five ideas behind the libraries, and what fresh agents did with them |
| [psanim and PSCat](docs/pscat-integration.md) | the branch, the API, building both |
| [Security](docs/security.md) | what is and is not guaranteed for untrusted programs |
| [Benchmarks](docs/benchmarks.md) · [Testing](docs/testing.md) | measurements and how to repeat them |
| [Limits and roadmap](docs/roadmap.md) | what it does not do |

The same pages, with diagrams, films and a live player: `scripts/build_site.py`.

## For agents

`skills/psanim/SKILL.md` is a skill: how to author, check, look at and
render a movie, with tested recipes and a list of what goes wrong. Ten of
the gallery pieces were written by agents given that skill and nothing
else; their friction reports shaped the engine and then the libraries
([evaluation](docs/vocabulary-evaluation.md)).

The loop is built so an agent need not watch anything: `psanim check`
reports blank frames, stillness, pops and loop seams as data; errors name
the frame, the command, the procedure and the element, and suggest a fix;
`psanim sheet` puts motion on one image.

## Status

A first version: the engine, CLI, browser build, gallery and docs work
and are tested on macOS. It depends on a PSCat branch under review
([PSCatLabs/pscat#1410](https://github.com/PSCatLabs/pscat/pull/1410)), is not
published, and has the limits listed in [the roadmap](docs/roadmap.md).

## License

MIT OR Apache-2.0, as PSCat's code is. The gallery pieces are original
work in this repository under the same terms.
