//! The animation vocabulary as data: every name psanim adds to PostScript,
//! its stack effect and what it does.
//!
//! This table is the single source of truth for the reference. `psanim api`
//! prints it, the documentation site and the agent skill are generated from
//! it, and a test checks it against the interpreter: every entry is defined
//! after the prelude loads, and every public name the prelude or the
//! operators define has an entry.

/// One word of the vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApiEntry {
    pub name: &'static str,
    /// Operands before the dash, results after, topmost last, as in the
    /// PostScript Language Reference.
    pub stack: &'static str,
    pub summary: &'static str,
    pub category: &'static str,
}

/// The categories, in the order the reference presents them.
pub const CATEGORIES: &[(&str, &str)] = &[
    ("declaration", "Declaring the movie"),
    ("clock", "The clock"),
    ("scope", "Time scopes: re-basing the clock for a procedure"),
    ("value", "Values over time"),
    ("path", "Paths: measuring, following, drawing on, morphing"),
    ("layer", "Layers and effects"),
    ("canvas", "The canvas and the camera"),
    ("text", "Text"),
    ("reel", "Reels: clips and transitions"),
    ("input", "Parameters, signals and events"),
    ("space", "Viewports and libraries"),
    ("component", "Components: movies placed inside movies"),
    ("story", "Storyboards, scripts and cascades"),
    (
        "pkstage",
        "pkstage: boxes, layout, themes, the camera by box",
    ),
    ("pkmotion", "pkmotion: entrances, exits, emphasis"),
    ("pktype", "pktype: text in boxes, kinetic type, captions"),
    (
        "pkdiagram",
        "pkdiagram: nodes, connectors, flows, a presenter's marks",
    ),
    ("pkcode", "pkcode: source code on screen"),
    ("pkchart", "pkchart: data that moves"),
    ("pkink", "pkink: marks with a hand in them"),
];

/// The library a category's words live in, if they are not always loaded:
/// a category named after a library is that library's.
pub fn library_of(category: &str) -> Option<&'static str> {
    crate::LIBRARIES
        .iter()
        .map(|l| l.0)
        .find(|name| *name == category)
}

const fn e(
    category: &'static str,
    name: &'static str,
    stack: &'static str,
    summary: &'static str,
) -> ApiEntry {
    ApiEntry {
        name,
        stack,
        summary,
        category,
    }
}

pub const API: &[ApiEntry] = &[
    e(
        "declaration",
        "animation",
        "dict  animation  -",
        "Declare the movie. Put it before anything that takes time or draws (definitions it needs, such as a reel array for `reelduration`, may come first). Keys, all optional: /Size [w h] (canvas units, default [1280 720]), /Duration seconds (5), /FPS (30), /Seed (1), /Background [r g b] or /transparent (black), /Title, /Description, /Loop bool, /Scene /name (scene), /Step /name (step), /StepRate ticks per second (FPS), /Persist bool, /MotionBlur samples, /Shutter 0..1 (0.5).",
    ),
    e(
        "clock",
        "now",
        "-  now  t",
        "The current time in seconds, in the innermost time scope. At the top of a scene it is the movie time.",
    ),
    e(
        "clock",
        "progress",
        "-  progress  u",
        "How far through the current time scope's window `now` is, 0 to 1, clamped. At the top of a scene: now / duration.",
    ),
    e(
        "clock",
        "span",
        "-  span  seconds",
        "The length of the current time scope's window.",
    ),
    e(
        "clock",
        "globaltime",
        "-  globaltime  t",
        "The movie time in seconds, whatever time scopes are open.",
    ),
    e(
        "clock",
        "frame",
        "-  frame  int",
        "The index of the frame being drawn (the tick index inside `step`).",
    ),
    e("clock", "fps", "-  fps  num", "The movie's frame rate."),
    e(
        "clock",
        "dt",
        "-  dt  seconds",
        "The length of one frame; inside `step`, the length of one simulation tick.",
    ),
    e(
        "clock",
        "duration",
        "-  duration  seconds",
        "The movie's length.",
    ),
    e(
        "clock",
        "width",
        "-  width  num",
        "The canvas width in units.",
    ),
    e(
        "clock",
        "height",
        "-  height  num",
        "The canvas height in units.",
    ),
    e(
        "clock",
        "movieseed",
        "-  movieseed  int",
        "The movie's seed.",
    ),
    e(
        "scope",
        "during",
        "a b proc  during  -",
        "Run proc only while a <= now < b. Inside, now runs from 0 to b-a and progress from 0 to 1.",
    ),
    e(
        "scope",
        "after",
        "a proc  after  -",
        "Run proc only once now >= a. Inside, now starts from 0.",
    ),
    e(
        "scope",
        "before",
        "b proc  before  -",
        "Run proc only while now < b. Inside, progress runs 0 to 1 over 0..b.",
    ),
    e(
        "scope",
        "delayed",
        "d proc  delayed  -",
        "Run proc with its clock d seconds behind: whatever it does happens d seconds later.",
    ),
    e(
        "scope",
        "staggered",
        "i gap proc  staggered  -",
        "Run proc delayed by i * gap seconds: item i of a row starts gap after item i-1.",
    ),
    e(
        "scope",
        "timescaled",
        "k proc  timescaled  -",
        "Run proc with its clock running k times as fast (k < 1 is slow motion).",
    ),
    e(
        "scope",
        "looped",
        "period proc  looped  -",
        "Run proc with now wrapping to 0 every period seconds; progress runs 0 to 1 each cycle.",
    ),
    e(
        "scope",
        "pingpong",
        "period proc  pingpong  -",
        "Run proc with now running 0 to period, back to 0, and so on.",
    ),
    e(
        "scope",
        "reversed",
        "proc  reversed  -",
        "Run proc with now running backward through the current window.",
    ),
    e(
        "scope",
        "globally",
        "proc  globally  -",
        "Run proc with now set back to the movie time, whatever time scopes are open. For a value shared by things in different scopes (a sun that the whole film sees): `{ [ ... ] keyframes } globally`.",
    ),
    e(
        "scope",
        "attime",
        "t proc  attime  -",
        "Run proc with now frozen at t: a still of a moving thing.",
    ),
    e(
        "scope",
        "timewindow",
        "t dur proc  timewindow  -",
        "Run proc with now set to t in a window dur seconds long.",
    ),
    e(
        "scope",
        "sequence",
        "[ dur proc dur proc ... ]  sequence  -",
        "Play procs one after another, each for its duration, each with its own clock from 0. A proc may be given by name (/intro); `null` in place of a proc is a gap.",
    ),
    e(
        "value",
        "ease",
        "x /name  ease  y",
        "Pass x (clamped to 0..1) through an easing curve. Names: linear smooth smoother hold spring, and quad cubic quart quint sine expo circ back elastic bounce each with -in, -out, -inout (as /cubic-out).",
    ),
    e(
        "value",
        "tween",
        "a b /name  tween  u",
        "The eased progress of now from a to b: 0 before a, 1 after b. The usual way to animate anything: `1 2.5 /cubic-out tween 100 400 between`.",
    ),
    e(
        "value",
        "ramp",
        "a b  ramp  u",
        "The linear progress of now from a to b, clamped to 0..1.",
    ),
    e(
        "value",
        "between",
        "u v0 v1  between  v",
        "Interpolate from v0 (at u=0) to v1 (at u=1). v0 and v1 are numbers, or arrays of numbers of one length (points, colours); u is not clamped, so an overshooting easing overshoots.",
    ),
    e(
        "value",
        "keyframes",
        "[ t v /ease t v ... ]  keyframes  v",
        "The value at now of a track through the keys. Each key is a time and a value (number or array), optionally followed by the easing used to arrive at it. Holds the first value before the first key and the last after the last.",
    ),
    e(
        "value",
        "keyat",
        "t [ t v /ease ... ]  keyat  v",
        "Like keyframes, at time t instead of now.",
    ),
    e(
        "value",
        "clamp",
        "x lo hi  clamp  y",
        "x limited to lo..hi.",
    ),
    e(
        "value",
        "fmod",
        "x m  fmod  r",
        "x modulo m for reals, always in 0..m (PostScript's mod is integers only): the wrap-around every repeating particle needs.",
    ),
    e(
        "value",
        "smoothstep",
        "e0 e1 x  smoothstep  u",
        "0 below e0, 1 above e1, a smooth S-curve between.",
    ),
    e(
        "value",
        "remap",
        "x a b c d  remap  y",
        "Map x from the range a..b to the range c..d, linearly, unclamped.",
    ),
    e(
        "value",
        "wiggle",
        "key freq amp  wiggle  v",
        "Smooth random motion: a value wandering in -amp..amp about freq times a second. Different keys (a number, or an array of numbers) give independent wanderings; the same key always gives the same one.",
    ),
    e(
        "clock",
        "turns",
        "k  turns  degrees",
        "k whole turns spread over the current window: k * 360 * (now / span). The angle for anything that must come home when a loop closes; k must be a whole number (or a whole number divided by the part's rotational symmetry).",
    ),
    e(
        "value",
        "loopnoise",
        "key radius  loopnoise  v",
        "Smooth random motion in -1..1 (key is a number or an array of numbers) that returns exactly to its start at the end of the current window: noise sampled around a circle. A larger radius wanders more. For organic drift in a seamless loop, where wiggle would not close.",
    ),
    e(
        "value",
        "hash01",
        "key  hash01  u",
        "A repeatable random number in 0..1 determined by key (a number or an array of numbers) and the movie seed. Stateless: the same key gives the same value on every frame, which keeps per-particle properties from flickering.",
    ),
    e(
        "path",
        "pathlength",
        "-  pathlength  len",
        "The length of the current path, all subpaths together.",
    ),
    e(
        "path",
        "pathpoint",
        "u  pathpoint  x y angle",
        "The point u (0..1) of the way along the current path, and the direction of travel there in degrees. Does not change the path. For moving something along a curve: `u pathpoint 3 1 roll translate rotate`.",
    ),
    e(
        "path",
        "trimpath",
        "u0 u1  trimpath  -",
        "Replace the current path by the part of it between fractions u0 and u1 of its length, subpaths taken one after another. `0 u trimpath stroke` draws a path on as u goes 0 to 1.",
    ),
    e(
        "path",
        "trimeach",
        "u0 u1  trimeach  -",
        "Like trimpath, but every subpath is trimmed to the same fractions of its own length, so all of them grow together.",
    ),
    e(
        "path",
        "resamplepath",
        "n  resamplepath  array",
        "n points evenly spaced along the current path, as [x0 y0 x1 y1 ...], in the user space current now (y up, like every coordinate). Does not change the path.",
    ),
    e(
        "path",
        "polypath",
        "array  polypath  -",
        "Replace the current path by the open polyline through [x0 y0 x1 y1 ...], in the current user space. Follow with closepath for a polygon. The fast way to replay geometry computed in setup: build the array once, then translate/rotate and polypath it each frame.",
    ),
    e(
        "path",
        "morphpath",
        "arrayA arrayB u  morphpath  -",
        "Replace the current path by the closed polygon u of the way from A to B, two arrays from resamplepath with the same n. The points are placed in the current user space, so the transformation in force applies to them as to any path.",
    ),
    e(
        "layer",
        "beginlayer",
        "-  beginlayer  -",
        "Start drawing on a separate transparent layer. The graphics state is unchanged.",
    ),
    e(
        "layer",
        "endlayer",
        "alpha  endlayer  -",
        "Finish the layer and composite it over what is beneath at opacity alpha.",
    ),
    e(
        "layer",
        "endlayerblend",
        "alpha /mode  endlayerblend  -",
        "Finish the layer and composite it with a blend mode: /normal /add /screen /multiply.",
    ),
    e(
        "layer",
        "blurlayer",
        "radius  blurlayer  -",
        "Blur everything drawn on the current layer so far (Gaussian, radius in canvas units).",
    ),
    e(
        "layer",
        "glowlayer",
        "radius strength  glowlayer  -",
        "Add a soft halo around everything on the current layer: a blurred copy added back, strength 0..2. The halo's light is the shape's own, spread out: a thin line needs a larger radius and a strength above 1 to show.",
    ),
    e(
        "layer",
        "faded",
        "alpha proc  faded  -",
        "Draw proc as one group at opacity alpha. Unlike setalpha, overlapping shapes inside do not show through each other.",
    ),
    e(
        "layer",
        "blended",
        "alpha /mode proc  blended  -",
        "Draw proc as a group composited with a blend mode.",
    ),
    e(
        "layer",
        "blurred",
        "radius proc  blurred  -",
        "Draw proc out of focus.",
    ),
    e(
        "layer",
        "glowing",
        "radius strength proc  glowing  -",
        "Draw proc with a halo of light. Try 8 1 for solid shapes, 24 1.6 for thin strokes.",
    ),
    e(
        "layer",
        "crossfade",
        "procA procB u  crossfade  -",
        "Draw A, then B over it at opacity u.",
    ),
    e(
        "layer",
        "vignette",
        "amount  vignette  -",
        "Darken the corners of everything drawn so far, 0..1.",
    ),
    e(
        "layer",
        "filmgrain",
        "amount  filmgrain  -",
        "Add film grain to everything drawn so far, 0..1. The grain changes every frame.",
    ),
    e(
        "canvas",
        "backdrop",
        "r g b  backdrop  -",
        "Fill the canvas rectangle (0 0 width height) with a colour, in the current coordinates and inside the current clip. Call it first, before any camera. A clip that starts with a backdrop can be wiped, pushed or irised by a reel's transitions.",
    ),
    e(
        "canvas",
        "camera",
        "cx cy zoom angle  camera  -",
        "Set the transformation so the world point (cx, cy) is at the centre of the frame, magnified by zoom and rotated by angle. Wrap in gsave/grestore.",
    ),
    e(
        "text",
        "typed",
        "string u  typed  string",
        "The first u (0..1) of the string, by characters: a typewriter reveal.",
    ),
    e(
        "text",
        "showcenter",
        "string  showcenter  -",
        "Show the string centred horizontally on the current point.",
    ),
    e(
        "text",
        "eachglyph",
        "string proc  eachglyph  -",
        "Run proc once per character with `(c) i` on the stack (the character as a one-character string, and its index) and the current point at that character's origin. The font must be set. proc consumes both operands; `{ pop show }` reproduces show. The current point is set afresh before each call, so proc may move it, build paths and paint; graphics state it changes (colour, CTM) stays changed, so wrap transforms in gsave/grestore.",
    ),
    e(
        "text",
        "spacedwidth",
        "string track  spacedwidth  w",
        "The width of the string when letterspaced: track extra units after each character but the last. For centring text drawn with eachspaced.",
    ),
    e(
        "text",
        "eachspaced",
        "string track proc  eachspaced  -",
        "Like eachglyph, letterspaced: each character's origin is track units further along than its natural place. `(TITLE) 6 { pop show } eachspaced` sets spaced capitals.",
    ),
    e(
        "text",
        "eachonpath",
        "string u0 proc  eachonpath  -",
        "Set the string along the current path, starting u0 (0..1) of the way along. proc runs per character with `(c) i` on the stack, in coordinates whose origin is the character's place on the path and whose x axis follows the path, with the current point at the character's origin. The path is left as it was. `{ pop show }` sets the text.",
    ),
    e(
        "reel",
        "reel",
        "[ [dur proc] [dur /transition] [dur proc] ... ]  reel  -",
        "Play clips in order with transitions between them. A transition overlaps the end of the clip before it and the start of the clip after it. Transitions: /cut /dissolve /dip /wipe-left /wipe-right /wipe-up /wipe-down /iris /push-left /push-right /zoom. A clip need not paint a ground: clips that draw only their own subject transition over whatever the scene drew before the reel.",
    ),
    e(
        "reel",
        "reelduration",
        "[ ... ]  reelduration  seconds",
        "The total length of a reel array: the clips' durations added up, minus the transitions' (a transition overlaps the clips on both sides, so a clip's own duration includes the transitions into and out of it). Use it for /Duration: define the reel array first, then `<< /Duration film reelduration ... >> animation`. It runs before the clips and your own transitions exist, so it takes the built-in transitions and any name starting `t-` for transitions and every other name for a clip; once the program has loaded, a measurement that a custom transition (any other name) would have changed is an error naming it, not a longer film.",
    ),
    e(
        "reel",
        "transitions",
        "-  transitions  dict",
        "The dictionary of transitions. To define your own, put a procedure there: `transitions /t-shutter { ... } put`, then use `[dur /t-shutter]` in a reel. Name it with a `t-` prefix, or define it before the reel is measured with reelduration: otherwise reelduration counts the entry as a clip, and loading the movie fails, naming it. It is called with `procA procB u`: draw the frame that is u (0 to 1, linear) of the way from A to B. A and B are ready-made procedures that draw each clip with its own clock still running; call them inside gsave/grestore with whatever clip, transform or layer the transition needs.",
    ),
    e(
        "input",
        "param",
        "/name default  param  value",
        "A value the host supplied for name (`--param name=value`), or default. Call it in setup.",
    ),
    e(
        "input",
        "signal",
        "/name  signal  num",
        "The value of the host's signal name at the current movie time; 0 if there is none.",
    ),
    e(
        "input",
        "sinceevent",
        "/name  sinceevent  seconds",
        "Seconds since the most recent event on track name; a million if there has been none.",
    ),
    e(
        "input",
        "eventcount",
        "/name  eventcount  int",
        "How many events on track name have happened so far.",
    ),
    // --- generated block: libraries ---
    e(
        "space",
        "within",
        "box proc  within  -",
        "Run proc inside the box [x y w h]: the origin moves to the box's corner, drawing is clipped to it, and `width` and `height` report the box's size. The spatial twin of `during`: a scene written against width and height can be placed in any box (a panel, one half of a comparison, a picture in a picture).",
    ),
    e(
        "space",
        "fitted",
        "box w h proc  fitted  -",
        "Run proc, which draws for a canvas w by h, scaled to fit inside the box and centred there, clipped. Inside, `width` and `height` are w and h. For placing a finished scene in a panel whatever its aspect.",
    ),
    e(
        "component",
        "component",
        "/name dict-or-(file.ps)-or-/entry  component  -",
        "Define a component and set up its first instance under the same name. A name is an entry of the catalog of ready-made elements (`psanim catalog` lists them: /busy /spinner component). A component is written like a movie: from a file, the file's own `animation` declaration, setup and `scene`; inline, a dictionary with the declaration's keys (/Size, /Duration, /Loop, /Seed, /Background) and /Setup { } and /Scene { }. Its setup runs once, in a dictionary of its own, so its names do not reach the movie or another instance; inside it `width`, `height`, `duration`, `globaltime`, `movieseed` and `param` are the component's. A file name is read from the movie's own directory. Call it in setup.",
    ),
    e(
        "component",
        "instance",
        "/new /name << /Seed s /Params << ... >> >>  instance  -",
        "Another instance of a component or a catalog entry, set up again: /Seed for its randomness, /Params for its `param` lookups (a name not given gets the default the component wrote). Inside a component, `param` reads only these, never the host's --param values. Call it in setup.",
    ),
    e(
        "component",
        "place",
        "/name opts  place  -",
        "Draw an instance into the scene. opts (all optional): /At the movie time it starts (0), /Speed (1), /Box [x y w h] its declared size is fitted into (the whole view), /Before /hide or /hold (its first frame), /After /hide /hold (its last frame) or /loop (default /loop if it declared /Loop true, else /hide). Inside, `now` and `globaltime` run from 0 at /At. It paints its /Background only if it declares one. Place the same instance as often as you like: each placement is drawn from its own time.",
    ),
    e(
        "component",
        "componentduration",
        "/name  componentduration  seconds",
        "The /Duration an instance declared, for timing the movie around it.",
    ),
    e(
        "component",
        "componentsize",
        "/name  componentsize  w h",
        "The /Size an instance declared, for choosing a box with its aspect.",
    ),
    e(
        "space",
        "use",
        "/library  use  -",
        "Load one of psanim's libraries, once: /pkstage /pkmotion /pktype /pkdiagram /pkcode /pkchart /pkink. Call it in setup, after `animation`. A library loads the ones it needs. `psanim api --libs` lists them; `psanim api --lib pkmotion` lists one's words.",
    ),
    e(
        "story",
        "storyboard",
        "[ /name seconds /name seconds ... ]  storyboard  -",
        "Name the stretches of the film, in order. Call it before `animation` and use `storylength` for /Duration; then place things by beat name instead of by second, so retiming the film is editing one list.",
    ),
    e(
        "story",
        "storylength",
        "-  storylength  seconds",
        "The total length of the storyboard.",
    ),
    e(
        "story",
        "beat",
        "/name  beat  t",
        "The movie time at which the named beat starts. Add to it for a moment inside the beat: `/build beat 0.5 add`.",
    ),
    e(
        "story",
        "beatspan",
        "/name  beatspan  a b",
        "The start and end of the named beat, on the movie's clock: operands for `during` at the top level of a scene.",
    ),
    e(
        "story",
        "inbeat",
        "/name proc  inbeat  -",
        "Run proc during the named beat, with the clock starting from 0 at the beat's start and `span` its length, whatever time scopes are open.",
    ),
    e(
        "story",
        "frombeat",
        "/name proc  frombeat  -",
        "Run proc from the start of the named beat to the end of the film, with the clock starting from 0 there: for something a beat introduces that then stays.",
    ),
    e(
        "story",
        "script",
        "proc  script  -",
        "Run proc as a script: a cursor starts at 0 and each `then`, `thenfor` and `wait` inside moves it on, so things happen one after another without any times written down. Scripts nest; the cursor outside is untouched.",
    ),
    e(
        "story",
        "then",
        "seconds proc  then  -",
        "In a script: run proc from the cursor on (it stays for good, its clock starting from 0), and move the cursor seconds forward. The seconds are how long before the next step starts, not how long proc lasts.",
    ),
    e(
        "story",
        "thenfor",
        "seconds proc  thenfor  -",
        "In a script: run proc for seconds starting at the cursor (it is gone afterwards; inside, `span` is seconds), and move the cursor past it.",
    ),
    e(
        "story",
        "wait",
        "seconds  wait  -",
        "In a script: move the cursor forward, leaving a pause. Negative overlaps the next step with the last.",
    ),
    e(
        "story",
        "cursor",
        "-  cursor  t",
        "The script cursor: the time, in the current scope, at which the next `then` will start.",
    ),
    e(
        "story",
        "tempo",
        "bpm  tempo  -",
        "Set the tempo in beats a minute (in setup; 120 if never set), for films cut to a pulse.",
    ),
    e(
        "story",
        "counts",
        "n  counts  seconds",
        "n beats at the tempo, in seconds: write times and storyboard lengths in beats (`[ /intro 8 counts /drop 16 counts ] storyboard`, `2 counts /pop { } enters`).",
    ),
    e(
        "story",
        "countnow",
        "-  countnow  n",
        "The beat the film is on, as a number that grows by 1 every beat (movie clock): its whole part is the beat's index, `countnow 4 mod` the place in a bar of four.",
    ),
    e(
        "story",
        "kick",
        "-  kick  u",
        "1 on every beat, falling quickly to 0 before the next: multiply a size, a brightness or a blur by it and the picture keeps time.",
    ),
    e(
        "story",
        "cascade",
        "array gap proc  cascade  -",
        "Run proc once per element with `item i` on the stack, each element's clock gap seconds behind the one before: a row of things arriving one after another, written as if each were alone.",
    ),
    e(
        "story",
        "cascadespan",
        "array gap  cascadespan  seconds",
        "How long after a cascade starts its last item starts: (n - 1) x gap. Use it for the time of the step that follows a cascade, or of item k (`k gap mul`), instead of working the number out and typing it: `buckets gap cascadespan 0.6 add wait`.",
    ),
    e(
        "value",
        "smoothkeys",
        "[ t v t v ... ]  smoothkeys  v",
        "The value at now of a smooth track through the keys (numbers, or arrays of one length: points, boxes, colours): it passes through every key without stopping or turning a corner, and starts and ends at rest. For camera paths and anything that should flow; use `keyframes` where each move should ease in and out on its own.",
    ),
    e(
        "value",
        "valueat",
        "array x  valueat  v",
        "The array read at a fractional index: element x where x is whole, the linear blend of its neighbours between (clamped to the ends). For a readout that follows a table: `table people valueat 1 fmtnum show` while `people` is a tween.",
    ),
    e(
        "value",
        "ranks",
        "[ v ... ]  ranks  [ r ... ]",
        "The rank of each value, 0 for the largest. Interpolating two rank arrays with `between` slides rows past each other: a bar-chart race.",
    ),
    e(
        "value",
        "niceticks",
        "lo hi n  niceticks  [ v ... ]",
        "About n round numbers covering lo..hi (steps of 1, 2, 2.5 or 5 times a power of ten): tick positions for an axis.",
    ),
    e(
        "path",
        "onpath",
        "u proc  onpath  -",
        "Run proc with the origin at the point u (0..1) of the way along the current path and the x axis along the direction of travel: draw the thing that rides the path as if at 0 0 facing right. The path is kept.",
    ),
    e(
        "path",
        "straightenpath",
        "u anchor  straightenpath  -",
        "Unbend the current path by u (0 as it is, 1 a straight line), keeping its length: every turn is scaled by 1 - u. The point anchor (0..1 of the way along) stays where it is, with its direction. An arc that unrolls into a strip, a circle's circumference laid flat, a rope pulled taut: `morphpath` would shrink them on the way.",
    ),
    e(
        "path",
        "taperpath",
        "[ w ... ]  taperpath  -",
        "Replace the current path by the outline of a stroke along it whose width follows the list, spread evenly along its length: [12 0] is a comet, [0 9 0] a brush mark. Fill the result.",
    ),
    e(
        "text",
        "strcat",
        "(a) (b)  strcat  (ab)",
        "Join two strings.",
    ),
    e(
        "text",
        "fmtnum",
        "num decimals  fmtnum  string",
        "The number as text with that many decimal places and commas between thousands: 1234567.8 1 fmtnum is (1,234,567.8). For counters: `1 3 /cubic-out tween 0 48213 between 0 fmtnum show`.",
    ),
    e(
        "text",
        "fmtfixed",
        "num decimals  fmtfixed  string",
        "The number as text with that many decimal places and no thousands separators (years, versions).",
    ),
    e(
        "text",
        "zeropad",
        "int width  zeropad  string",
        "The integer as text, padded with zeros on the left to that width: 7 2 zeropad is (07).",
    ),
    e(
        "text",
        "fmtclock",
        "seconds  fmtclock  string",
        "Seconds as a clock: (01:05), or (1:01:05) from an hour up.",
    ),
    e(
        "text",
        "splitwords",
        "(text)  splitwords  [ (word) ... ]",
        "The words of a string, split at spaces and line breaks.",
    ),
    e(
        "text",
        "splitlines",
        "(text)  splitlines  [ (line) ... ]",
        "The lines of a string, split at \\n.",
    ),
    e(
        "text",
        "codetokens",
        "(source) /language  codetokens  lines",
        "Source code as data, one array per line of alternating text and kind: [ (fn) /keyword ( ) /plain (main) /call ... ]. Kinds: /plain /keyword /string /number /comment /type /call /name /punct. Languages: /rust /python /javascript /typescript /c /go /postscript /shell /json /plain. Call it in setup; draw with pkcode.",
    ),
    e(
        "text",
        "difflines",
        "(old) (new)  difflines  rows",
        "A line diff as data: an array of [ /same|/del|/add (line) oldindex newindex ] rows, in the order they would be printed.",
    ),
    e(
        "pkstage",
        "canvasbox",
        "-  canvasbox  box",
        "The whole canvas as a box, [0 0 width height]. A box is an array [x y w h], x y its bottom-left corner. Inside `within`, the enclosing box.",
    ),
    e(
        "pkstage",
        "vmin",
        "n  vmin  units",
        "n hundredths of the canvas's shorter side, in canvas units: a size that suits the frame whatever its shape. `4 vmin` is a comfortable text size, `6 vmin` a margin.",
    ),
    e(
        "pkstage",
        "inset",
        "box m  inset  box",
        "The box shrunk by m on every side; a negative m grows it.",
    ),
    e(
        "pkstage",
        "insetxy",
        "box mx my  insetxy  box",
        "The box shrunk by mx at the left and right and by my at the top and bottom.",
    ),
    e(
        "pkstage",
        "subbox",
        "box fx fy fw fh  subbox  box",
        "The part of a box that starts fx across and fy up and is fw of its width and fh of its height, all as fractions: `stage 0 0.6 1 0.4 subbox` is its top two fifths.",
    ),
    e(
        "pkstage",
        "aspect",
        "-  aspect  ratio",
        "The canvas's width divided by its height: above 1 is landscape. For a layout that changes with the frame's shape: `aspect 1 gt { wide } { tall } ifelse`.",
    ),
    e(
        "pkstage",
        "shifted",
        "box dx dy  shifted  box",
        "The box moved by dx, dy.",
    ),
    e(
        "pkstage",
        "boxpoint",
        "box /anchor  boxpoint  x y",
        "A point of the box: /c centre, /n /s /e /w the middles of its edges, /ne /nw /se /sw its corners, or [fx fy] as fractions across and up.",
    ),
    e(
        "pkstage",
        "placed",
        "box w h /anchor  placed  box",
        "A new box w by h, set inside the given box against that anchor: `canvasbox 600 200 /c placed` is centred, /sw sits in the bottom-left corner.",
    ),
    e(
        "pkstage",
        "boxat",
        "x y w h /anchor  boxat  box",
        "A box w by h whose anchor point is at (x, y): `640 360 300 120 /c boxat` is centred on that point.",
    ),
    e(
        "pkstage",
        "cols",
        "box n gap  cols  [ box ... ]",
        "Cut the box into n equal columns with gap between them, left to right.",
    ),
    e(
        "pkstage",
        "rows",
        "box n gap  rows  [ box ... ]",
        "Cut the box into n equal rows with gap between them, top to bottom.",
    ),
    e(
        "pkstage",
        "cells",
        "box ncols nrows gap  cells  [ box ... ]",
        "Cut the box into a grid of equal cells, row by row from the top left. gap is one number, or [gapx gapy] for different spacing across and down.",
    ),
    e(
        "pkstage",
        "splitx",
        "box f  splitx  left right",
        "Cut the box in two at fraction f of its width.",
    ),
    e(
        "pkstage",
        "splity",
        "box f  splity  top bottom",
        "Cut the box in two at fraction f of its height, measured from the top.",
    ),
    e(
        "pkstage",
        "boxunion",
        "boxA boxB  boxunion  box",
        "The smallest box that holds both: the frame around a group, or a camera target that takes in two things.",
    ),
    e(
        "pkstage",
        "boxpath",
        "box  boxpath  -",
        "Add the box to the current path as a rectangle.",
    ),
    e(
        "pkstage",
        "roundpath",
        "box r  roundpath  -",
        "Add the box to the current path with corners of radius r (limited to half its shorter side, so a large r makes a pill).",
    ),
    e(
        "pkstage",
        "disc",
        "x y r  disc  -",
        "Fill a circle in the current colour.",
    ),
    e(
        "pkstage",
        "lookat",
        "box  lookat  -",
        "Point the camera at a box: set the transformation so the whole box is in frame, centred, as large as fits. Wrap in gsave/grestore. The box can come from `keyframes`, `smoothkeys` or `between` over boxes, which is a camera move: `[ 0 whole  3 detail /cubic-inout ] keyframes lookat`.",
    ),
    e(
        "pkstage",
        "visiblebox",
        "-  visiblebox  box",
        "The part of the world the frame shows under the current transformation (the bounding box, if the view is rotated): test against it to skip drawing what is off screen.",
    ),
    e(
        "pkstage",
        "themes",
        "-  themes  dict",
        "The built-in themes by name: /midnight /paper /blueprint /terminal /poster /dusk. Each maps roles to colours (/bg /surface /line /fg /muted /accent /accent2 /accent3 /good /bad), fonts (/title /body /mono /display /serif) and sizes (/corner /stroke).",
    ),
    e(
        "pkstage",
        "theme",
        "-  theme  dict",
        "The theme in force: read a role with `theme /accent get`, change one with `theme /accent [1 0.3 0.2] put` (in setup).",
    ),
    e(
        "pkstage",
        "usetheme",
        "/name  usetheme  -",
        "Make a built-in theme the one in force (in setup, or inside a scene for a part of it); or give a dictionary of roles to change just those: `<< /accent [1 0.3 0.2] /title /Bookman-Demi >> usetheme`.",
    ),
    e(
        "pkstage",
        "rgbof",
        "role  rgbof  r g b",
        "The colour of a theme role (or of an [r g b] array, unchanged) as three numbers.",
    ),
    e(
        "pkstage",
        "ink",
        "role  ink  -",
        "Set the colour by theme role: `/accent ink`. Every pk word that takes a colour takes a role name or an [r g b] array.",
    ),
    e(
        "pkstage",
        "mixink",
        "roleA roleB u  mixink  [ r g b ]",
        "The colour u of the way from one role (or [r g b]) to another: `/muted /accent 1 2 ramp mixink ink` is a colour change over time.",
    ),
    e(
        "pkstage",
        "face",
        "/role size  face  -",
        "Select a font by theme role at a size: /title /body /mono /display /serif. Any font name works too (`/Palatino-Italic 30 face`).",
    ),
    e(
        "pkstage",
        "ground",
        "-  ground  -",
        "Fill the canvas (or the enclosing `within` box) with the theme's background colour, and set the current colour to the theme's text colour (/fg), so text drawn without choosing a colour is visible. The first line of most scenes.",
    ),
    e(
        "pkstage",
        "spotlight",
        "box-or-boxes amount  spotlight  -",
        "Wash everything drawn so far, except the box (or an array of boxes), toward the theme's ground colour by amount 0..1 (darker on a dark theme, paler on a light one): directs the eye to one part of a busy picture. Animate the amount; draw afterwards anything that should stay bright.",
    ),
    e(
        "pkmotion",
        "entering",
        "spec proc  entering  -",
        "Draw proc arriving by a motion at the start of the current time scope. spec is /name, [/name seconds] or [/name seconds /easing]; motions: /fade /rise /fall /slide-left /slide-right /pop /grow /zoom /drop /focus /spin /flip /unfold /unroll /wipe-right /wipe-left /wipe-up /wipe-down /mask-up /mask-down /mask-right. The motion needs no anchor: the drawing is measured, so /pop scales it about its own centre. proc must contain everything it needs: operands left outside the braces stay on the stack while the thing is absent. Inside a `cascade` body, `def` the item and index first.",
    ),
    e(
        "pkmotion",
        "leaving",
        "spec proc  leaving  -",
        "Draw proc departing by a motion, run backward, over the last seconds of the current time scope (so the scope must have an end: inside `during`, `thenfor`, `inbeat`).",
    ),
    e(
        "pkmotion",
        "enters",
        "t spec proc  enters  -",
        "Nothing before t; at t proc arrives by the motion; then it stays. Inside proc the clock starts from 0 at t (read a film-wide value with `globally`). The usual way to bring something on: `1.5 /pop { ... } enters`.",
    ),
    e(
        "pkmotion",
        "leaves",
        "t spec proc  leaves  -",
        "proc is there until t, departing by the motion (run backward) so that it is gone at t.",
    ),
    e(
        "pkmotion",
        "onstage",
        "a b specin specout proc  onstage  -",
        "proc arrives at a by one motion and has left by b by another; inside, the clock runs from 0. A whole appearance in one line: `2 6 /rise /fade { ... } onstage`.",
    ),
    e(
        "pkmotion",
        "staged",
        "specin specout proc  staged  -",
        "proc arrives by one motion at the start of the current time scope and departs by another at its end: a whole appearance inside `inbeat`, `thenfor` or `during`. `/rise /fade { ... } staged`.",
    ),
    e(
        "pkmotion",
        "emphasized",
        "t spec proc  emphasized  -",
        "proc is drawn throughout; at t it performs an emphasis and returns to rest. Emphases: /pulse /shake /wobble /jump /squash /blink /tada /dim. Nest it around an entrance to do both.",
    ),
    e(
        "pkmotion",
        "motion",
        "u spec proc  motion  -",
        "Draw proc at stage u of a motion, driven by you: 0 is absent and 1 in place for an entrance, 0 and 1 both in place for an emphasis. For a motion tied to a signal, a scroll position or your own tween. The spec's time and easing are ignored.",
    ),
    e(
        "pkmotion",
        "swapped",
        "t spec procA procB  swapped  -",
        "A until t, B from t: A leaves by the motion (gone at t) and B enters by it. One thing replacing another in the same place. Inside B the clock starts from 0 at t; inside A it is the outer clock.",
    ),
    e(
        "pkmotion",
        "motions",
        "-  motions  dict",
        "Your own motions: put a procedure there that takes u and returns a dictionary of any of /dx /dy (canvas units) /dxrel /dyrel (multiples of the drawing's own width and height) /scale /sx /sy /rotate /alpha /blur /wipe [l b r t] (the part of its box shown) /mask (clip to its resting box) /pivot [fx fy]. Then use its name wherever a motion goes.",
    ),
    e(
        "pkmotion",
        "motionnames",
        "-  motionnames  [ /name ... ]",
        "The names of the built-in motions, entrances first, for a catalogue or a test.",
    ),
    e(
        "pkmotion",
        "rank",
        "i n /order  rank  j",
        "Where item i of n comes in a cascade: /forward, /reverse, /center (the middle first), /edges (the ends first) or /random. Use the result as the index for `staggered`.",
    ),
    e(
        "pkmotion",
        "echoed",
        "n gap proc  echoed  -",
        "Draw proc as it was at n earlier moments, gap seconds apart, oldest faintest, then as it is now: a trail behind anything that moves. Works because proc is a function of its clock; there is no state.",
    ),
    e(
        "pktype",
        "fontsize",
        "-  fontsize  size",
        "The size of the current font.",
    ),
    e(
        "pktype",
        "showright",
        "(text)  showright  -",
        "Show the string so that it ends at the current point: right-aligned, which keeps a changing number steady.",
    ),
    e(
        "pktype",
        "sym",
        "/name  sym  -",
        "Show a mathematical sign or Greek letter at the current point at the current font's size, keeping the font: /neq /le /ge /approx /pm /times /div /dot /infinity /degree /arrow /larrow /uparrow /downarrow /implies /sum /product /sqrt /integral /in /bullet, and /alpha /beta /gamma /delta /epsilon /theta /lambda /mu /pi /rho /sigma /tau /phi /omega /Delta /Sigma /Pi /Omega. The text fonts have no such characters: a non-ASCII character in a string shows as a blank. `(kiwi ) show /neq sym ( plum) show`.",
    ),
    e(
        "pktype",
        "symwidth",
        "/name  symwidth  w",
        "The width `sym` advances by for that sign, for centring a line that contains one.",
    ),
    e(
        "pktype",
        "symbols",
        "-  symbols  dict",
        "The signs `sym` knows, by name, as strings in the Symbol font.",
    ),
    e(
        "pktype",
        "showruns",
        "runs  showruns  -",
        "Show a line of mixed type at the current point, in the current font and colour. A run list is an array of (text), /name (a sign from `symbols`: /pi /times ...), [/it (r)] (italic, as variables are set), [/sup x], [/sub x] and [/frac a b]: `[ (Area = ) /pi [/it (r)] [/sup (2)] ] showruns`.",
    ),
    e(
        "pktype",
        "showrun",
        "runs i  showrun  -",
        "Show only run i of a run list, at the place it has in the line that starts at the current point: an equation brought in term by term, each term with its own time and motion. The current point is left for the next call to start from only if you reset it: `moveto` before each.",
    ),
    e(
        "pktype",
        "runwidth",
        "runs  runwidth  w",
        "The width of a run list in the current font, for centring it.",
    ),
    e(
        "pktype",
        "runbox",
        "runs i  runbox  box",
        "The box of run i in the line that starts at the current point: the target for a `marker`, a `circled`, a `note` or a `spotlight` on one term of an equation.",
    ),
    e(
        "pktype",
        "textbox",
        "(text)  textbox  box",
        "The box the string would cover if shown at the current point in the current font: for a plate behind it, a marker over it, a camera move onto it.",
    ),
    e(
        "pktype",
        "partbox",
        "(text) (part)  partbox  box",
        "The box of the first occurrence of part within text, were text shown at the current point: to mark, underline or point at one word of a line.",
    ),
    e(
        "pktype",
        "showin",
        "(text) box /anchor  showin  -",
        "Show the string inside a box, set against an anchor: /c centres it both ways, /w sets it left and vertically centred, /nw top left.",
    ),
    e(
        "pktype",
        "placetext",
        "(text) box /anchor  placetext  -",
        "Move to where `showin` would start the string, without showing it: the way to put `kinetic`, `typewrite`, `showruns` or a `charpath` in a box. `(TITLE) box /c placetext (TITLE) /glyphs 0.05 /rise kinetic`.",
    ),
    e(
        "pktype",
        "fitface",
        "(text) box /role  fitface  -",
        "Select the role's font (or any font name) at the largest size at which the string's width fits the box's width and its capital height the box's height: a headline that fills its column whatever the words.",
    ),
    e(
        "pktype",
        "wraplines",
        "(text) width  wraplines  [ (line) ... ]",
        "Break text into lines no wider than width in the current font; \\n forces a break.",
    ),
    e(
        "pktype",
        "textlayout",
        "(text) box opts  textlayout  [ [ (line) x y ] ... ]",
        "Wrap text into a box and return each line with where it goes. opts is /left, /center or /right, or << /Align /left /Leading 1.3 /VAlign /top >> (/VAlign /top /middle /bottom).",
    ),
    e(
        "pktype",
        "paragraph",
        "(text) box opts  paragraph  -",
        "Set wrapped text in a box. opts as for `textlayout`.",
    ),
    e(
        "pktype",
        "eachword",
        "(text) proc  eachword  -",
        "Like `eachglyph`, by words: proc runs once per word with `(word) i` on the stack and the current point at the word's origin.",
    ),
    e(
        "pktype",
        "kinetic",
        "(text) /unit gap spec  kinetic  -",
        "Show text at the current point with each /glyphs, /words or the /whole entering by a motion, gap seconds apart: `(MOVE) /glyphs 0.05 /mask-up kinetic`. Its clock is the current scope's: wrap it in `after` or `then` to start it later, in `leaving` to take it away.",
    ),
    e(
        "pktype",
        "kineticparagraph",
        "(text) box opts /unit gap spec  kineticparagraph  -",
        "A wrapped paragraph whose /glyphs, /words or /lines enter one after another by a motion. opts as for `textlayout`.",
    ),
    e(
        "pktype",
        "marker",
        "box u  marker  -",
        "A highlighter bar over the first u (0..1) of a box, in the current colour: draw it before the text it marks, with the box from `textbox` or `partbox`. Keeps the current point.",
    ),
    e(
        "pktype",
        "underline",
        "box u  underline  -",
        "A rule under a box, drawn on as u goes 0 to 1, in the current colour and line width. Keeps the current point.",
    ),
    e(
        "pktype",
        "strikethrough",
        "box u  strikethrough  -",
        "A straight line through the middle of a box, drawn on as u goes 0 to 1, in the current colour and line width: a word struck out. Use it with `textbox` or `partbox`. Keeps the current point. pkdiagram's `struck` is the hand-drawn version.",
    ),
    e(
        "pktype",
        "typewrite",
        "(text) u  typewrite  -",
        "Show the first u (0..1) of the string with a block caret after it; the caret blinks once the typing is done.",
    ),
    e(
        "pktype",
        "captions",
        "[ [ t0 t1 (text) ] ... ] box opts  captions  -",
        "Subtitles: show the caption whose time has come, centred at the bottom of the box on a plate, fading in and out. opts: << /Size s /Role /body /Plate true /Karaoke false >>; with /Karaoke the words light up across the caption's time. Times are on the current scope's clock.",
    ),
    e(
        "pkdiagram",
        "node",
        "(label) box opts  node  -",
        "Draw a labelled node filling a box. opts is a fill role (/surface, /accent) or << /Fill /surface /Stroke /line /Ink /fg /Shape /box /Sub (second line) /Size s /Font /title >>; shapes: /box /pill /circle /diamond /cylinder /plain. The label is sized to fit.",
    ),
    e(
        "pkdiagram",
        "group",
        "(label) box opts  group  -",
        "Draw a container: a dashed outline around a box with its name on the top edge. opts is a role for the line, or << /Stroke /muted /Ink /muted /Size s >>. `boxA boxB boxunion -30 inset` is the box around two nodes.",
    ),
    e(
        "pkdiagram",
        "connect",
        "boxA boxB /route  connect  -",
        "Make the current path a connector from the edge of box A to the edge of box B. Routes: /straight, /elbow (right angles, sideways or vertical as the boxes' places call for), /hv and /vh (one corner: across then up, or up then across), /arc and /arc-right (a curve bulging left or right of the direction of travel), [/sideA /sideB] to name the side of each box it leaves and arrives by ([/s /n] goes down, across and down), or [ x y x y ... ], points to pass through (to go round a node). Being a path, it takes `trimpath`, `onpath`, `pathpoint`, `arrow`, `pulse`.",
    ),
    e(
        "pkdiagram",
        "connectgap",
        "-  connectgap  units",
        "The space left between a box and a connector's end (6). Redefine it in setup to change it.",
    ),
    e(
        "pkdiagram",
        "arrow",
        "u  arrow  -",
        "Stroke the first u (0..1) of the current path with an arrowhead riding its tip, in the current colour and line width: an arrow that draws itself. Consumes the path.",
    ),
    e(
        "pkdiagram",
        "pulse",
        "u len  pulse  -",
        "A comet at u (0..1) along the current path, len of the path long, in the current colour: a message in flight. Draws nothing for u outside 0..1. Keeps the path, so several can ride one connector.",
    ),
    e(
        "pkdiagram",
        "pulsetrain",
        "period n  pulsetrain  [ u ... ]",
        "n evenly spaced positions, each travelling 0 to 1 every period seconds: a steady flow for an edge's /Pulse.",
    ),
    e(
        "pkdiagram",
        "edge",
        "boxA boxB opts  edge  -",
        "Draw a connector between two boxes. opts is a route name, or << /Route /elbow /Draw u /Label (text) /Pulse u-or-array /Role /line /Head true /Dash false /Width w /LabelRole /muted /LabelSize s /PulseRole /accent /PulseLen 0.18 >>. Animate /Draw to draw it on; the label fades in over the second half; /Pulse puts comets at those places once it is drawn (a place of exactly 0 or 1, or outside, draws none, so a clamped tween shows one message and then nothing).",
    ),
    e(
        "pkdiagram",
        "circled",
        "box  circled  -",
        "Make the current path a loose hand-drawn ring around a box. Draw it on: `0 u trimpath stroke`.",
    ),
    e(
        "pkdiagram",
        "struck",
        "box  struck  -",
        "Make the current path a strike-through line across a box.",
    ),
    e(
        "pkdiagram",
        "bracepath",
        "box /side  bracepath  -",
        "Make the current path a curly brace along one side of a box (/n /s /e /w), opening away from it.",
    ),
    e(
        "pkdiagram",
        "bracepoint",
        "box /side  bracepoint  x y",
        "The tip of the brace `bracepath` draws on that side: where its label goes.",
    ),
    e(
        "pkdiagram",
        "note",
        "(text) box /side u  note  -",
        "A label beside a box with a leader line to it: the line draws on and the text fades in as u goes 0 to 1. /side is where the label sits (/n /s /e /w /ne /nw /se /sw). Uses the current font and colour.",
    ),
    e(
        "pkcode",
        "codeblock",
        "lines box opts  codeblock  -",
        "Draw tokenized code (from `codetokens`) in a box as a panel. opts: << /Reveal u (typed so far, 0..1) /Focus [ line ... ] (keep these bright, dim the rest) /FocusAmount a (0..1, animate it) /Scroll n (lines scrolled off the top) /Numbers bool /Title (file.rs) /Size s /Leading 1.4 /Plate true >>. The size is chosen to fit unless given.",
    ),
    e(
        "pkcode",
        "codelinebox",
        "lines box opts n  codelinebox  box",
        "The box where line n (from 0) of a code block is drawn, given the same lines, box and opts: for notes, arrows, circles and camera moves onto a line.",
    ),
    e(
        "pkcode",
        "codecolors",
        "-  codecolors  dict",
        "Which theme role each kind of token is drawn in (/keyword /string /number /comment /type /call /name /punct /plain). Change an entry in setup to restyle.",
    ),
    e(
        "pkcode",
        "diffrows",
        "(old) (new) /language  diffrows  rows",
        "A line diff with each line tokenized, for `codediff`. Call it in setup.",
    ),
    e(
        "pkcode",
        "codediff",
        "rows box opts u  codediff  -",
        "Draw a change to code: the old version at u = 0 becomes the new at u = 1. Removed lines redden and close up, then added lines open in green, and unchanged lines slide to their new places. opts as for `codeblock` (/Title /Size /Leading /Plate).",
    ),
    e(
        "pkchart",
        "plotframe",
        "box [ x0 x1 ] [ y0 y1 ]  plotframe  -",
        "Say where the plot is and what range of data it spans. Every pkchart word after it draws in that frame. Call it in the scene (or in setup, if there is one chart).",
    ),
    e(
        "pkchart",
        "plotbox",
        "-  plotbox  box",
        "The current plot frame's box.",
    ),
    e(
        "pkchart",
        "plotpoint",
        "x y  plotpoint  X Y",
        "Data coordinates to canvas coordinates, in the current plot frame: for placing a note, a marker or a label at a data point.",
    ),
    e(
        "pkchart",
        "plotaxes",
        "opts  plotaxes  -",
        "Draw the axes, ticks, labels and grid of the current plot frame. opts: << /Draw u (drawn on, 0..1) /YTicks 5 /XTicks 5 /Grid true /Decimals 0 /XDecimals 0 /XNames [ (a) ... ] (names under evenly spaced slots, for bars) /XValues [ ... ] /YValues [ ... ] (ticks exactly there) /Suffix (%) /XLabel (text) /YLabel (text) /Size s >>. The plot frame's box is the data rectangle; labels and titles are drawn outside it, about three times /Size to the left and below, so inset the box from the page.",
    ),
    e(
        "pkchart",
        "bars",
        "[ v ... ] opts  bars  -",
        "Columns across the plot frame, one per value, on the frame's y scale. opts: << /Grow u (bars rise one after another as u goes 0..1) /Stagger 0.6 /Role /accent or /Roles [ ... ] /Dim [ i ... ] (draw these muted) /Accent [ i ... ] (draw all but these muted) /Gap 0.3 /Values true /Count false (true: labels count up with the bars; otherwise they show the true value) /Decimals 0 /Prefix () /Suffix () /Size s >>. To move between datasets, interpolate the values: `u A B between`.",
    ),
    e(
        "pkchart",
        "hbars",
        "[ v ... ] opts  hbars  -",
        "Rows down the plot frame, on the frame's x scale. opts as for `bars`, plus /Names [ (a) ... ] written left of the rows and /Ranks [ r ... ], the row each bar sits in: `u A ranks B ranks between` slides bars past each other as their order changes.",
    ),
    e(
        "pkchart",
        "plotline",
        "[ [ x y ] ... ]  plotline  -",
        "Make the current path the line through the data points. Stroke it; `0 u trimpath` first draws it on.",
    ),
    e(
        "pkchart",
        "seriesline",
        "[ y ... ]  seriesline  -",
        "Like `plotline` for values spread evenly across the frame's x range.",
    ),
    e(
        "pkchart",
        "plotarea",
        "-  plotarea  -",
        "Close the current line (from `plotline`) down to the bottom of the plot frame, to fill the area under it.",
    ),
    e(
        "pkchart",
        "dots",
        "[ [ x y ] ... ] u radius  dots  -",
        "A dot at each data point, popping in one after another as u goes 0 to 1, in the current colour.",
    ),
    e(
        "pkchart",
        "donut",
        "[ v ... ] box opts  donut  -",
        "A ring of shares in a box, swept on clockwise from the top. opts: << /Grow u /Roles [ ... ] /Inner 0.62 /Gap 2 /Center (text) /CenterRole /fg >>.",
    ),
    e(
        "pkchart",
        "inplot",
        "proc  inplot  -",
        "Run proc clipped to the plot frame. Lines are not clipped by themselves, so use it whenever data can leave the frame's range, which is always when the range is animated (a zoom is `u [0 365] [0 80] between` as the x range).",
    ),
    e(
        "pkchart",
        "plotrule",
        "opts  plotrule  -",
        "A reference line across the plot frame at a data value: << /Y 50 (or /X v) /Label (even odds) /Draw u /Dash true /Role /muted /Size s >>.",
    ),
    e(
        "pkchart",
        "plotmark",
        "x y opts  plotmark  -",
        "Mark a data point: a dot, dashed drop lines to both axes, and a label on a leader line. opts: << /Label (23 people) /Side /ne /Draw u /Drop true /Role /accent /Radius r /Size s >> (/Side is where the label sits: /n /s /e /w /ne /nw /se /sw; /Size defaults to 3 vmin). The one-call form of the annotation every data story needs.",
    ),
    e(
        "pkchart",
        "chartroles",
        "-  chartroles  [ /role ... ]",
        "The roles marks cycle through when none are given.",
    ),
    e(
        "pkink",
        "revealalong",
        "u width proc  revealalong  -",
        "Run proc clipped to a band of that width along the first u (0..1) of the current path: proc draws a whole mark (a paintkit brush stroke, a chalk line, a hatched ribbon) and only the part a pen moving along the path has reached shows. A textured stroke that draws itself. Consumes the path.",
    ),
    e(
        "pkink",
        "revealband",
        "u0 u1 width proc  revealband  -",
        "Like `revealalong`, for the stretch of the path between u0 and u1 only. On a persistent canvas give it where the pen was a frame ago and where it is now, so a translucent medium is laid down once and not again on every frame.",
    ),
    e(
        "pkink",
        "around",
        "n proc  around  -",
        "Run proc n times, each turned 360/n further about the origin, with the copy's index on the stack: petals, spokes, ticks.",
    ),
    e(
        "pkink",
        "brushstroke",
        "[ w ... ]  brushstroke  -",
        "Fill the current path as a stroke whose width follows the list, in the current colour: `taperpath fill`.",
    ),
    e(
        "pkink",
        "boxgradient",
        "box c0 c1 angle  boxgradient  -",
        "Fill a box with a gradient between two colours (roles or [r g b]) along an angle: 0 runs left to right, 90 bottom to top. `canvasbox /bg /surface 90 boxgradient` is a ground.",
    ),
    e(
        "pkink",
        "jitter",
        "key amount  jitter  dx dy",
        "A small steady offset within amount either way, fixed by key: the hand-set unevenness of a row of marks. The same key always gives the same offset.",
    ),
    // --- end generated block ---
];

/// The entry for `name`.
pub fn lookup(name: &str) -> Option<&'static ApiEntry> {
    API.iter().find(|e| e.name == name)
}

fn markdown_for(keep: impl Fn(&str) -> bool) -> String {
    let mut out = String::new();
    for (key, title) in CATEGORIES.iter().filter(|c| keep(c.0)) {
        out.push_str(&format!("## {title}\n\n"));
        if let Some(lib) = library_of(key) {
            out.push_str(&format!("Load with `/{lib} use`.\n\n"));
        }
        for entry in API.iter().filter(|e| e.category == *key) {
            out.push_str(&format!(
                "### `{}`\n\n`{}`\n\n{}\n\n",
                entry.name, entry.stack, entry.summary
            ));
        }
    }
    out
}

/// The core reference as Markdown, grouped by category: every word that
/// is defined without loading a library.
pub fn to_markdown() -> String {
    markdown_for(|c| library_of(c).is_none())
}

/// One library's reference as Markdown; `None` if there is no such library.
pub fn library_markdown(lib: &str) -> Option<String> {
    library_of(lib).map(|l| markdown_for(|c| c == l))
}

/// The reference as JSON: an array of `{name, stack, summary, category}`.
pub fn to_json() -> serde_json::Value {
    serde_json::Value::Array(
        API.iter()
            .map(|e| {
                serde_json::json!({
                    "name": e.name, "stack": e.stack, "summary": e.summary, "category": e.category,
                    "library": library_of(e.category),
                })
            })
            .collect(),
    )
}
