# Widgets: psanim programs that keep running

A widget is a small psanim program that lives in a web page (or, later, on
a desktop) and keeps running: a clock, a build-status light, a weather
glass, a spinner beside a form. This note says what a widget is, how a
host talks to it, and what exists today. It ends with a brainstorm about
how far the idea could go.

## What exists (first step, issue #58)

- **`<psanim-widget>`** (`web/widget.js`): a custom element that opens a
  movie (a URL, inline text, or a catalog entry by name) with parameters,
  feeds it a wall-clock signal, the pointer and key presses, runs it
  indefinitely, and stops drawing when it is off screen, when the tab is
  hidden, or when nothing would change.
- **Live-use additions to the browser engine** (`crates/psanim-wasm`,
  `web/psanim.js`): parameters at open (`open(src, { params })`, read as
  `--param` reads them), catalog entries by name (`catalog(name)`,
  `catalogNames()`), and events from the page (`event(name, t)`,
  `clearEvents(name)`), backed by `Movie::push_event` /
  `Movie::clear_events` in the engine.
- **A demo**, `web/widgets.html`: a digital clock and an analog clock
  (the catalog's clocks on the `clock` signal), a pointer toy reacting to
  hover and clicks, three spinners, and a build-status pipeline driven by a
  signal the page sets from JavaScript.

What was run: the wasm crate's native ABI test, `scripts/wasm_smoke.mjs` in
Node 24, and the demo page in headless Chromium through Playwright (all
seven widgets drew without errors; a widget scrolled off screen stopped
drawing and resumed when it came back; a held movie stopped drawing after
its last frame and drew again when a signal changed; clicks and key
presses reached the program as events). Not run: Firefox, Safari, a
hidden tab (the code listens for `visibilitychange`, untested), a page
left open for hours.

Measured in that Chromium run (Apple M-series laptop): each widget's own
module instance holds 12 to 18 MB of linear memory; per-frame cost at the
sizes on the page was 0.1 to 0.2 ms for a 96-pixel spinner, 0.6 ms for the
288-pixel pointer toy, 1.3 ms for the 288-pixel analog clock, 1.2 to 2.7 ms
for the 952x190 pipeline and about 6 ms for the 952x317 digital clock.

## What a widget is

**A movie, plus live inputs, plus (optionally) state.**

- **The movie** is an ordinary psanim program. Nothing marks it as a
  widget: the same file renders offline with `psanim render`, plays in
  `<psanim-player>`, and runs as a widget. That is the main design
  decision, and it follows the rule that frames are pure functions of
  time: a widget is that function evaluated at the page's clock, with the
  page's inputs.
- **Inputs** are what psanim already has for offline renders: parameters
  (fixed at open), signals (numbers that vary with time) and event tracks
  (moments). Offline they come from `--param` and a CSV file; live they
  come from the host. The program cannot tell the difference, which keeps
  a widget testable with `psanim check` and `psanim sheet`.
- **State** lives in the host, not the program. The build-status demo is
  the pattern: the page knows which stage the build is in and sets a
  signal; the movie draws that stage. A stateful movie (`step`) works as
  a widget too, but its state is a simulation of its own time, not memory
  of what the viewer did.

A widget that reacts to the pointer:

```postscript
%!PS
<< /Size [240 240] /Duration 4 /Loop true /FPS 30 /Background [0.06 0.07 0.1] >> animation
/scene {
  /mx /mousex signal def  /my /mousey signal def  /hv /hover signal def
  /s /click sinceevent def                 % seconds since the last click
  20 20 220 { /x exch def
    20 20 220 { /y exch def
      /d x mx sub dup mul y my sub dup mul add sqrt def
      /near 1 d 90 div sub 0 max hv mul def
      /wave x y add 480 div now 4 div add 360 mul sin 0.5 mul 0.5 add def
      0.25 near 0.75 mul add  0.45 wave 0.3 mul add near 0.4 mul add  0.9 setrgbcolor
      x y 1.6 near 6 mul add wave 0.6 mul add 0 360 arc fill
    } for
  } for
  s 1 lt { 1 s sub setgray 2 setlinewidth mx my s 160 mul 0 360 arc stroke } if
} def
```

## How the host talks to it

```html
<script type="module" src="widget.js"></script>
<psanim-widget entry="clock-digital" time="run" fps="10"
               params='{"Signal": "clock", "Theme": "amber"}'></psanim-widget>
<psanim-widget id="ci" entry="pipeline" time="run" params='{"Signal": "build"}'></psanim-widget>
<script type="module">
  document.getElementById("ci").setSignal("build", 1.5);   // build done, tests half way
</script>
```

| Channel | When | Widget API | In the program |
|---|---|---|---|
| Parameters | at open | `params` attribute (JSON); changing it reopens | `/Name default param` |
| Signals | any time, hold until set again | `widget.setSignal(name, v)` | `/name signal` |
| Events | the moment they happen | `widget.event(name)` | `/name sinceevent`, `/name eventcount` |
| Wall clock | before every frame | automatic: `clock` (local seconds since midnight, with the fraction) | `/clock signal`, or a catalog clock with `/Signal (clock)` |
| Pointer | before every frame | automatic: `mousex`, `mousey` (canvas units, y up), `mousedown`, `hover` | `/mousex signal` |
| Pointer and keys | as they happen | automatic: events `click`, `keydown`; signal `key` (last key code) | `/click sinceevent` |

Parameters are text or numbers, read exactly as the command line reads
`--param` (a number if it parses as one, else a string). A catalog entry's
name-valued parameters (`/Theme /amber`) accept the string, because
PostScript compares a string and a name by their characters (checked in
Node: `Second: "sweep"` on the analog clock gives a different picture from
`"tick"` at the same moment). Array
parameters (`/Color [1 0 0]`) cannot be passed yet; the host has no way
to write an array value.

Nothing goes the other way yet: a widget cannot tell the page "the user
clicked the deploy stage". That needs hit regions (below).

## Time

A page has no end, and most movies have a duration. `time` chooses how
the page's clock (seconds since the widget opened) maps to movie time:

- **`loop`** (the default when the movie declares `/Loop true`): wrap at
  the duration. Events are on the movie's clock, so when it wraps the
  widget clears its event tracks; `eventcount` therefore counts events
  since the last wrap, not since the page opened. A widget that must count
  clicks for ever should take the count as a signal the page keeps.
- **`hold`** (the default otherwise): run to the end and stay on the last
  frame. A held widget stops drawing until an input changes. Its movie
  time is frozen there, so an event lands at that same time and
  `sinceevent` stays 0: a click ripple freezes instead of playing. Use
  `run` for a widget that reacts to events.
- **`run`**: seconds since open, past the declared duration. For movies
  that are written to be open-ended: the catalog clocks on a signal (their
  24-hour duration would otherwise hold after a day) and the signal-driven
  pipeline (which declares one second when it is driven, so `hold` would
  freeze its spinning ring). A pure movie can be asked for any time, but
  what it draws past its end is up to the program: checked in Node at twice
  the duration, confetti, fireworks, pipeline, progress, testdots, spinner
  and fire all still draw a picture; other entries were not checked.

Reopening a widget (new `params`, a resize) keeps its clock, so a clock
does not jump, but starts the movie afresh: its events are gone.

A stateful movie as a widget renders forward from its last tick each
frame, which is cheap; when a `loop` widget wraps, it replays from its
first checkpoint.

## Sizing

The movie declares its size in canvas units. The widget fits its
element's CSS width (300 pixels unless the page sets one; the canvas
never sizes the element, or a 2x screen would grow it on every resize): scale = element width x device pixel ratio (at most
2) / movie width, so a 960-unit clock in a 476-pixel column renders at
952 pixels on a 2x screen. A `scale` attribute fixes it instead. A
`ResizeObserver` reopens the movie at the new scale when the width
changes by more than 15% (debounced 200 ms); smaller changes just stretch
the canvas. Height follows the movie's aspect ratio.

This is a design choice worth revisiting: a widget could instead receive
its box (`width` and `height` from the host, as a component does from
`place`), so a status light could reflow into a wide or a tall slot. The
engine already supports this inside a movie (`within`, components); what
is missing is a host-side override of `/Size` at open.

## Sleeping when nothing changes

Each animation frame the widget decides whether to draw:

1. **Off screen or tab hidden**: it stops its `requestAnimationFrame`
   loop altogether (`IntersectionObserver`, `visibilitychange`), and
   restarts when it becomes visible. Measured: zero frames drawn while
   scrolled out of view.
2. **Same frame, same inputs**: it computes the frame number on the
   movie's clock at `min(fps attribute, movie /FPS)`. If that has not
   changed and no input changed (a signal set from the page, the pointer,
   an event), it skips the frame. For example, on a 120 Hz display a 30 fps
   movie would draw one animation frame in four (the headless Chromium run
   drew about one in two); a held movie at its end draws
   nothing until an input changes. The `clock` signal does not count as a
   change (it changes every frame); a clock widget is woken by its own
   frame rate, so `fps="10"` on a digital clock is a cheap way to make it
   sleep 90% of the time.

What is **not** done, and what it would take:

- **Knowing that a frame would be identical without rendering it.** The
  digital clock's picture changes once a second, but the widget redraws
  it at 10 fps because nothing says the program does not read `now`. The
  feasible fix is dependency tracking in the engine: record, per frame,
  whether the program read `now` (or any time word) and which signals it
  read, with which values. If a frame read no time and the signals it read
  have the same values, the next frame is the same picture. psanim owns the
  time words and `signal`, so this is a psanim change, not a PSCat one.
  It would also need care with components and time scopes, which read
  time indirectly.
- **Comparing pixels after rendering** costs the render and saves only
  the copy to the canvas; not worth it at these frame costs.
- **A declared wake rate** (`/Wake 1` in the declaration: "my picture
  changes at most once a second") is the cheap version of dependency
  tracking, at the cost of trusting the author.

## Sandboxing and budgets

What a widget can do is what a psanim program can do (`docs/security.md`):
no file writes, no network, no wall clock (the clock comes in as a
signal), and in the browser no file reads beyond PSCat's built-in
libraries. Each frame has a step budget (the engine's default of 50 M
steps from `web/psanim.js`), and a frame over budget is an error shown
on the widget, not a hung page. A frame within budget still runs on the
page's main thread, though, so a heavy widget makes the page janky.

Each widget instantiates its own copy of the module (the compiled code is
shared, the memory is not): 12 to 18 MB per widget measured. Seven widgets
on the demo page cost about 100 MB. Two ways down, neither built:

- **One engine, many movies.** The wasm state holds one `Movie`; holding
  a table of them by handle would let every widget on a page share one
  instance (and its fonts and libraries, which are most of the memory).
- **A Worker** (the roadmap's Worker-based player): one Worker rendering
  every widget on the page, posting frames back as transferable
  `ImageBitmap`s or drawing into `OffscreenCanvas`es. That takes rendering
  off the main thread, gives a place to enforce a per-widget time budget
  with the Worker's own clock, and lets a runaway widget be terminated
  without touching the page. This first step runs on the main thread.

## What belongs in PSCat and what in psanim

- **PSCat**: the interpreter, the sandbox (read roots, no writes, step
  and memory limits), drawing, fonts. A widget system needs nothing new
  from PSCat to exist. Things that would belong there if wanted: a
  cheaper way to share one set of fonts and libraries between many
  interpreters in one process (it would help any host, not only
  widgets), and a per-interpreter memory limit that does not need a
  process-wide tracking allocator.
- **psanim**: time, signals, events, parameters, components, the catalog,
  and everything in this note: the time policies, live events, dependency
  tracking for sleep, hit regions, the host elements. These are about
  animation and its inputs, which PSCat deliberately knows nothing about
  (`docs/HANDOFF.md`, decision 7).
- **The host** (a page, a desktop app): the wall clock, the pointer, the
  event loop, scheduling and visibility. psanim never reads these itself.

## Brainstorm: NeWS on steroids

NeWS ran PostScript inside the window server: each window was drawn by a
PostScript process that also handled its own input, and the client sent
it programs rather than pixels. Widgets written in PostScript, drawn by
PSCat, animated by psanim and composed from the catalog could go further.
What that system would need, beyond this first step:

- **Hit regions and output.** A widget draws paths; it should be able to
  name some of them (`/deploy { ...path... } hitregion`) so the host can
  report which region a click landed in, both as an event to the program
  (`/deploy sinceevent`) and as a DOM event to the page
  (`psanim-hit` with the name). That is the widget talking back without
  breaking purity: hit regions are a function of the frame, like pixels.
- **Composition across widgets.** Today a widget is one movie. The
  catalog's components already compose inside a movie (`place` with a
  box). A dashboard could be one movie placing many entries, each fed by
  its own signals (namespaced: `ci.build`, `weather.temp`), so a layout
  change is a PostScript edit, not a page edit.
- **Durable state, kept outside.** A to-do widget needs memory. Keep the
  program pure and give the host a small store: the program declares
  named values it would like kept, the host persists them and feeds them
  back as signals or parameters. State changes become events the program
  emits through hit regions. This keeps every frame reproducible from
  (time, inputs, stored values).
- **Text input.** Keys arrive as events today, with the last key code as
  a signal. Real text fields need the host to own the string (and IME,
  accessibility) and pass it as a parameter or a text signal; psanim
  signals are numbers only.
- **Accessibility.** A canvas is opaque to screen readers. A widget
  should declare its text alternative (`/Label (Build: tests failing)`),
  computed per frame like everything else, and the host should mirror it
  into `aria-label` or a live region.
- **Scheduling across many widgets.** One Worker (or one desktop
  process) rendering dozens of widgets needs a scheduler: frame rates,
  wake rates and dependency tracking decide who draws this vsync, and a
  per-widget budget decides who gets throttled.
- **Trust.** NeWS trusted its clients. Here, widgets from strangers are
  the point (a catalog anyone can publish to), so the sandbox must hold:
  no wall clock, no network, no files, budgets on every frame, and an OS
  boundary (a Worker, a separate process) around the interpreter.
- **Sharing the engine.** One interpreter's worth of fonts and libraries
  shared by all widgets (above), and the catalog's components compiled
  once.
- **Network data without network access.** Weather or CI status comes
  from a host-side fetcher that turns a JSON feed into signals; the
  widget declares which feeds it wants and never sees a URL.

## A desktop host (sketch, not built)

A macOS menu-bar or desktop widget host would link the `psanim` crate
natively (no wasm), so it gets threads, wall-clock budgets and full speed:

- **Shape**: a small Swift (AppKit) app with a Rust static library
  exposing the same C ABI as `crates/psanim-wasm` (load with params, set
  signal, push event, render at time). The menu-bar variant renders into
  an `NSStatusItem` button image (template image for monochrome, or full
  colour at 22 pt x 2); the desktop variant renders into a borderless,
  non-activating `NSPanel` at desktop level, one per widget, or a
  WidgetKit extension (which only updates on a timeline, so it suits
  `hold`-style status, not animation).
- **Clock**: a `CVDisplayLink` (or `CADisplayLink` on macOS 14+) per
  screen drives frames; the same skip rules as the web element apply.
  Occlusion (`NSWindow.occlusionState`) replaces `IntersectionObserver`;
  screen lock and sleep pause everything.
- **Inputs**: the `clock` signal from the system time; pointer and click
  from the panel's tracking area; and a local feed configuration (a JSON
  file naming widgets, their params and where their signals come from:
  a command's output, an HTTP endpoint polled by the host).
- **Isolation**: one render thread per widget or one shared pool,
  `Budget` with wall-clock limits (available natively), and widgets
  from untrusted sources in a separate process (an XPC service) so a
  crash or a runaway program cannot take down the host.
- **Order of work**: only after the web element and the Worker player
  have settled the time, sleep and input rules, so both hosts share them.
