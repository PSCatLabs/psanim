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
];

const fn e(category: &'static str, name: &'static str, stack: &'static str, summary: &'static str) -> ApiEntry {
    ApiEntry { name, stack, summary, category }
}

pub const API: &[ApiEntry] = &[
    e("declaration", "animation", "dict  animation  -",
      "Declare the movie. The first statement of a program. Keys, all optional: /Size [w h] (canvas units, default [1280 720]), /Duration seconds (5), /FPS (30), /Seed (1), /Background [r g b] or /transparent (black), /Title, /Description, /Loop bool, /Scene /name (scene), /Step /name (step), /StepRate ticks per second (FPS), /Persist bool, /MotionBlur samples, /Shutter 0..1 (0.5)."),

    e("clock", "now", "-  now  t",
      "The current time in seconds, in the innermost time scope. At the top of a scene it is the movie time."),
    e("clock", "progress", "-  progress  u",
      "How far through the current time scope's window `now` is, 0 to 1, clamped. At the top of a scene: now / duration."),
    e("clock", "span", "-  span  seconds",
      "The length of the current time scope's window."),
    e("clock", "globaltime", "-  globaltime  t",
      "The movie time in seconds, whatever time scopes are open."),
    e("clock", "frame", "-  frame  int",
      "The index of the frame being drawn (the tick index inside `step`)."),
    e("clock", "fps", "-  fps  num", "The movie's frame rate."),
    e("clock", "dt", "-  dt  seconds",
      "The length of one frame; inside `step`, the length of one simulation tick."),
    e("clock", "duration", "-  duration  seconds", "The movie's length."),
    e("clock", "width", "-  width  num", "The canvas width in units."),
    e("clock", "height", "-  height  num", "The canvas height in units."),
    e("clock", "movieseed", "-  movieseed  int", "The movie's seed."),

    e("scope", "during", "a b proc  during  -",
      "Run proc only while a <= now < b. Inside, now runs from 0 to b-a and progress from 0 to 1."),
    e("scope", "after", "a proc  after  -",
      "Run proc only once now >= a. Inside, now starts from 0."),
    e("scope", "before", "b proc  before  -",
      "Run proc only while now < b. Inside, progress runs 0 to 1 over 0..b."),
    e("scope", "delayed", "d proc  delayed  -",
      "Run proc with its clock d seconds behind: whatever it does happens d seconds later."),
    e("scope", "staggered", "i gap proc  staggered  -",
      "Run proc delayed by i * gap seconds: item i of a row starts gap after item i-1."),
    e("scope", "timescaled", "k proc  timescaled  -",
      "Run proc with its clock running k times as fast (k < 1 is slow motion)."),
    e("scope", "looped", "period proc  looped  -",
      "Run proc with now wrapping to 0 every period seconds; progress runs 0 to 1 each cycle."),
    e("scope", "pingpong", "period proc  pingpong  -",
      "Run proc with now running 0 to period, back to 0, and so on."),
    e("scope", "reversed", "proc  reversed  -",
      "Run proc with now running backward through the current window."),
    e("scope", "attime", "t proc  attime  -",
      "Run proc with now frozen at t: a still of a moving thing."),
    e("scope", "timewindow", "t dur proc  timewindow  -",
      "Run proc with now set to t in a window dur seconds long."),
    e("scope", "sequence", "[ dur proc dur proc ... ]  sequence  -",
      "Play procs one after another, each for its duration, each with its own clock from 0. A proc may be given by name: /intro."),

    e("value", "ease", "x /name  ease  y",
      "Pass x (clamped to 0..1) through an easing curve. Names: linear smooth smoother hold spring, and quad cubic quart quint sine expo circ back elastic bounce each with -in, -out, -inout (as /cubic-out)."),
    e("value", "tween", "a b /name  tween  u",
      "The eased progress of now from a to b: 0 before a, 1 after b. The usual way to animate anything: `1 2.5 /cubic-out tween 100 400 between`."),
    e("value", "ramp", "a b  ramp  u",
      "The linear progress of now from a to b, clamped to 0..1."),
    e("value", "between", "u v0 v1  between  v",
      "Interpolate from v0 (at u=0) to v1 (at u=1). v0 and v1 are numbers, or arrays of numbers of one length (points, colours); u is not clamped, so an overshooting easing overshoots."),
    e("value", "keyframes", "[ t v /ease t v ... ]  keyframes  v",
      "The value at now of a track through the keys. Each key is a time and a value (number or array), optionally followed by the easing used to arrive at it. Holds the first value before the first key and the last after the last."),
    e("value", "keyat", "t [ t v /ease ... ]  keyat  v",
      "Like keyframes, at time t instead of now."),
    e("value", "clamp", "x lo hi  clamp  y", "x limited to lo..hi."),
    e("value", "smoothstep", "e0 e1 x  smoothstep  u",
      "0 below e0, 1 above e1, a smooth S-curve between."),
    e("value", "remap", "x a b c d  remap  y",
      "Map x from the range a..b to the range c..d, linearly, unclamped."),
    e("value", "wiggle", "key freq amp  wiggle  v",
      "Smooth random motion: a value wandering in -amp..amp about freq times a second. Different keys give independent wanderings; the same key always gives the same one."),
    e("value", "hash01", "key  hash01  u",
      "A repeatable random number in 0..1 determined by key (a number or an array of numbers) and the movie seed. Stateless: the same key gives the same value on every frame, which keeps per-particle properties from flickering."),

    e("path", "pathlength", "-  pathlength  len",
      "The length of the current path, all subpaths together."),
    e("path", "pathpoint", "u  pathpoint  x y angle",
      "The point u (0..1) of the way along the current path, and the direction of travel there in degrees. Does not change the path. For moving something along a curve: `u pathpoint 3 1 roll translate rotate`."),
    e("path", "trimpath", "u0 u1  trimpath  -",
      "Replace the current path by the part of it between fractions u0 and u1 of its length, subpaths taken one after another. `0 u trimpath stroke` draws a path on as u goes 0 to 1."),
    e("path", "trimeach", "u0 u1  trimeach  -",
      "Like trimpath, but every subpath is trimmed to the same fractions of its own length, so all of them grow together."),
    e("path", "resamplepath", "n  resamplepath  array",
      "n points evenly spaced along the current path, as [x0 y0 x1 y1 ...]. Does not change the path."),
    e("path", "morphpath", "arrayA arrayB u  morphpath  -",
      "Replace the current path by the closed polygon u of the way from A to B, two arrays from resamplepath with the same n."),

    e("layer", "beginlayer", "-  beginlayer  -",
      "Start drawing on a separate transparent layer. The graphics state is unchanged."),
    e("layer", "endlayer", "alpha  endlayer  -",
      "Finish the layer and composite it over what is beneath at opacity alpha."),
    e("layer", "endlayerblend", "alpha /mode  endlayerblend  -",
      "Finish the layer and composite it with a blend mode: /normal /add /screen /multiply."),
    e("layer", "blurlayer", "radius  blurlayer  -",
      "Blur everything drawn on the current layer so far (Gaussian, radius in canvas units)."),
    e("layer", "glowlayer", "radius strength  glowlayer  -",
      "Add a soft halo around everything on the current layer: a blurred copy added back, strength 0..2."),
    e("layer", "faded", "alpha proc  faded  -",
      "Draw proc as one group at opacity alpha. Unlike setalpha, overlapping shapes inside do not show through each other."),
    e("layer", "blended", "alpha /mode proc  blended  -",
      "Draw proc as a group composited with a blend mode."),
    e("layer", "blurred", "radius proc  blurred  -", "Draw proc out of focus."),
    e("layer", "glowing", "radius strength proc  glowing  -", "Draw proc with a halo of light."),
    e("layer", "crossfade", "procA procB u  crossfade  -",
      "Draw A, then B over it at opacity u."),
    e("layer", "vignette", "amount  vignette  -",
      "Darken the corners of everything drawn so far, 0..1."),
    e("layer", "filmgrain", "amount  filmgrain  -",
      "Add film grain to everything drawn so far, 0..1. The grain changes every frame."),

    e("canvas", "backdrop", "r g b  backdrop  -",
      "Fill the whole frame with a colour, whatever the transformation and clip are."),
    e("canvas", "camera", "cx cy zoom angle  camera  -",
      "Set the transformation so the world point (cx, cy) is at the centre of the frame, magnified by zoom and rotated by angle. Wrap in gsave/grestore."),

    e("text", "typed", "string u  typed  string",
      "The first u (0..1) of the string, by characters: a typewriter reveal."),
    e("text", "showcenter", "string  showcenter  -",
      "Show the string centred horizontally on the current point."),
    e("text", "eachglyph", "string proc  eachglyph  -",
      "Run proc once per character with `(c) i` on the stack (the character as a one-character string, and its index) and the current point at that character's origin. The font must be set. proc consumes both operands; `{ pop show }` reproduces show."),

    e("reel", "reel", "[ [dur proc] [dur /transition] [dur proc] ... ]  reel  -",
      "Play clips in order with transitions between them. A transition overlaps the end of the clip before it and the start of the clip after it. Transitions: /cut /dissolve /dip /wipe-left /wipe-right /wipe-up /wipe-down /iris /push-left /push-right /zoom."),
    e("reel", "reelduration", "[ ... ]  reelduration  seconds",
      "The total length of a reel array: use it for /Duration."),
    e("reel", "transitions", "-  transitions  dict",
      "The dictionary of transitions. Add `procA procB u` procedures under names starting with t- to define your own."),

    e("input", "param", "/name default  param  value",
      "A value the host supplied for name (`--param name=value`), or default. Call it in setup."),
    e("input", "signal", "/name  signal  num",
      "The value of the host's signal name at the current movie time; 0 if there is none."),
    e("input", "sinceevent", "/name  sinceevent  seconds",
      "Seconds since the most recent event on track name; a million if there has been none."),
    e("input", "eventcount", "/name  eventcount  int",
      "How many events on track name have happened so far."),
];

/// The entry for `name`.
pub fn lookup(name: &str) -> Option<&'static ApiEntry> {
    API.iter().find(|e| e.name == name)
}

/// The reference as Markdown, grouped by category.
pub fn to_markdown() -> String {
    let mut out = String::new();
    for (key, title) in CATEGORIES {
        out.push_str(&format!("## {title}\n\n"));
        for entry in API.iter().filter(|e| e.category == *key) {
            out.push_str(&format!("### `{}`\n\n`{}`\n\n{}\n\n", entry.name, entry.stack, entry.summary));
        }
    }
    out
}

/// The reference as JSON: an array of `{name, stack, summary, category}`.
pub fn to_json() -> serde_json::Value {
    serde_json::Value::Array(
        API.iter()
            .map(|e| {
                serde_json::json!({
                    "name": e.name, "stack": e.stack, "summary": e.summary, "category": e.category,
                })
            })
            .collect(),
    )
}
