# Evaluating the vocabulary with fresh agents

Does the expressive vocabulary ([design](vocabulary-design.md)) make an
agent better at making films? This is what was run on 2026-11-24 to find
out, what it showed, and what it did not.

Everything here is from single runs. An agent's token count, time and
taste vary from run to run, and no task was repeated enough times to put
an error bar on anything. Read the numbers as observations, not as
measurements of an effect size. Where a result is the agent's own
judgement (the 1 to 10 scores, "acceptable") it says so.

## Method

Every agent was a fresh instance of the same model with no knowledge of
psanim. Each was given the path of the skill (`skills/psanim/`), a
frozen copy of the binary, a brief, and a required report format. They
were told not to read the engine, the library sources, the design
documents, the git history or other films. They worked alone, looked at
their own contact sheets and stills, and never saw each other's work.
Token counts, tool calls and wall time are the harness's totals for each
agent, which include reading the skill.

Three experiments:

1. **Productions.** Five ambitious films, one agent each, to find where
   the vocabulary fails on real work.
2. **Same film, two vocabularies.** One fixed brief (a 20 second
   explainer, "How a hash map finds a key", seven specified beats),
   written once with the libraries and once with only the core
   vocabulary and plain PostScript. Run again with the libraries after
   the skill was revised.
3. **Same change, two films.** The two films from experiment 2 were each
   given to a new agent with one four-part change request: restyle for a
   light theme; twelve buckets in two rows instead of eight in one;
   insert a two-second step in the middle; slow one cascade.

## 1. Productions

| film | what | seconds | lines | ms/frame | tokens | minutes | library bugs reported |
|---|---|---|---|---|---|---|---|
| [how-it-works](../gallery/how-it-works/README.md) | architecture explainer | 69.5 | 361 | 4.1 | 183k | 9.2 | 0 wrong output; 3 limitations |
| [circle-area](../gallery/circle-area/README.md) | visual proof, two ways | 63 | 346 | 2.4 | 176k | 10.0 | 0 |
| [birthday](../gallery/birthday/README.md) | data story | 67 | 260 | 4.5 | 171k | 7.6 | 0 |
| [fresh](../gallery/fresh/README.md) | kinetic type, 1080x1920 | 34 | 295 | 7.7 | 169k | 9.0 | 0 |
| [heron](../gallery/heron/README.md) | ink and wash, persistent | 46 | 180 | 15 | 136k | 9.0 | 0 |

All five finished, passed `psanim check --all` (the type film with five
warnings for its intended hard cuts), and rendered. Each took between
three and nine rounds of looking and fixing. No agent reported a library
word producing wrong output; the three "limitations" in the first row
are an edge label with no size option, a cylinder's label colliding with
its rim, and an arc route crossing a node, all since fixed.

### What carried the films

Reported by every agent, or by four of the five:

- **Motions by name** (`enters`, `onstage`, `staged`, `emphasized`,
  `swapped`). Nobody wrote the arithmetic of an entrance. The kinetic
  type film used eleven different motions.
- **`storyboard` with `inbeat`.** All four pure films structured
  themselves this way.
- **Boxes** for layout, and `between` on boxes for moves (a title
  becoming a heading, a figure match-moving between beats, a camera).
- **Theme roles.** No film hard-codes more than a handful of colours.
- **The path idea.** `trimpath` on `plotline`, `circled`, `struck`,
  `bracepath`; `pulse` and `arrow` on hand-built paths when `edge` was
  not enough.

### What they did not use

- **`script` / `then`.** Not one agent used it, in any experiment. With
  a storyboard giving each beat a local clock, they found absolute
  beat-local seconds clearer ("easier to align with caption times").
  This is the clearest negative result: the cursor is the abstraction
  the designer was fondest of and the authors did without. It stays,
  because it costs nothing and reads well in short examples; see
  experiment 3 for what its absence costs later.
- `spotlight`, `group`, `codediff`, `echoed`, pkchart's `donut` and
  `hbars`, pkink in the non-painting films: no need arose.

### What they had to build by hand

Ranked by how often it came up and how many lines it cost. The middle
column is what was done about it the same day.

| missing | built by hand in | now |
|---|---|---|
| maths: pi, "not equal", arrows, superscripts, fractions, terms that arrive one by one (a 50-line typesetter) | circle-area, both hash maps | `sym`, run lists: `showruns`, `showrun`, `runbox` |
| unrolling an arc to a strip of the same length | circle-area | `straightenpath` |
| annotating a data point: dot, drop lines, label; reference lines; clipping lines to the frame; a readout that follows a table | birthday (about 40 lines) | `plotmark`, `plotrule`, `inplot`, `valueat`, explicit ticks |
| bar labels that count through values that are not data | birthday | labels show the true value; `/Count true` opts in |
| an edge that leaves by a named side, or goes round a node | how-it-works, hash map | routes `[/s /n]` and `[x y ...]`; `/LabelSize` |
| putting `kinetic` or `typewrite` text in a box | fresh, how-it-works | `placetext` |
| time in beats, a value that pulses on the beat | fresh | `tempo`, `counts`, `countnow`, `kick` |
| fractional sub-boxes, one-axis inset, orientation | fresh | `subbox`, `insetxy`, `aspect`; `cells` takes two gaps |
| painting only the new part of a translucent stroke | heron | `revealband` |
| a stroke scheduler for a persistent canvas (about 60 lines) | heron | **not built**: first item of the [roadmap](roadmap.md) |
| a camera inside a region under fixed furniture | how-it-works | documented: `lookat` inside `within` |
| transitions between storyboard beats | birthday, circle-area, how-it-works | **not built**; the blank-frame trap is documented |
| tracking, cap-height fitting, vertical text | fresh | **not built** |
| a tracer on a series, unit charts, cross-fading ticks | birthday | **not built** |

The films were left as their authors wrote them, hand-built helpers
included.

### Where the skill misled

- `storytelling.md` was an empty file when the first round ran; four of
  five reports opened with it. It is written now.
- Non-ASCII characters show as blanks with no warning. Both hash-map
  agents and the maths agent found this by looking.
- Operands left outside a motion's braces (`(label) box 2 /pop { node }
  enters`) leak onto the stack while the thing is absent. Three agents
  hit it, one of them inside `cascade`.
- A film-clock value read inside `enters` or `onstage`, where the clock
  restarts. Two agents hit it though it was documented; it now has its
  own table ("which words restart the clock").
- `check` with its default 16 samples passed films with single blank
  frames at beat boundaries; `check --all` is now a numbered step of the
  loop.
- The chapters' examples are drawn at 640x360, and two agents carried
  their small type sizes into 1280x720 films.

## 2. Same film, two vocabularies

| | core only | libraries, first skill | libraries, revised skill |
|---|---|---|---|
| lines of code (non-blank, non-comment) | 157 | 77 | 61 |
| helpers written by hand | 6 (about 45 lines) | 2 | 0 named helpers |
| `check` errors before it ran clean | 0 | 1 | 0 |
| fix rounds after the first draft | 1 | 3 | 1 |
| tool calls | 13 | 25 | 14 |
| tokens | 86k | 90k | 97k |
| wall time | 2.4 min | 2.6 min | 2.2 min |
| ms/frame at 1280x720 | 6.0 | 3.1 | 5.4 |
| author's own score | 7/10 | 7/10 | 7/10 |

What this does and does not say:

- **The code is less than half the size** with the libraries, and what
  disappears is the bookkeeping: a rounded rectangle, text centred in a
  box, an arrow that draws on, a fade-and-rise entrance, a dim-all-but
  one, a shake. The core-only author wrote each of those and listed them
  as what they wished had existed.
- **It was not faster or cheaper to write.** Tokens and time are the
  same to within what one run can show. The task is small, and reading
  the skill dominates: the library skill is longer.
- **The first library run needed more fixing, not less** (three rounds
  to one). Its three defects were a stack leak from operands outside a
  motion body, an auto-routed connector that ran through text, and
  missing glyphs. All three were then addressed (a documented trap,
  side routes, `sym`), and the re-run needed one round and hit none of
  them. One run each way does not prove the revision caused that.
- **Quality, as judged by the authors, was the same.** All three scored
  their film 7. A higher-level vocabulary did not make a more ambitious
  film from a fixed brief; it made the same film from less.

## 3. Same change, two films

One change request, applied by a new agent to each of the first two
films above.

| | library film | core-only film |
|---|---|---|
| lines changed (`diff`) | 51 | 81 |
| restyle to a light theme | 3 places | 14 places |
| twelve buckets in two rows | 9 places | 12 places |
| insert a two-second step | 1 new block + 9 retimed numbers | 1 new block + 11 retimed numbers |
| slow the cascade | 1 number (+ retiming) | 1 number (+ retiming) |
| broke without `check` noticing | spotlight too dark on paper | dim level too faint; 3 layout knock-ons |
| tokens / time | 71k / 1.7 min | 71k / 1.9 min |
| editor's own score | 8/10 | 8/10 |

- **The restyle is where the vocabulary pays**: two theme lines against
  a palette block, two new font helpers and four call sites. The
  library film's editor wrote that no drawing code changed.
- **The layout change was easier but not easy.** Things derived from
  boxes followed the new layout by themselves; a hand-measured box
  around one glyph and a literal repeated four times did not.
- **The time change cost the same in both**, and for the same reason:
  both authors had written absolute seconds. This is the cost of the
  authors' preference in experiment 1. A film written as a `script`
  would have absorbed the insertion; no agent wrote one. The skill now
  says so where it introduces storyboards and scripts.
- The edit exposed two design flaws no production had: `spotlight`
  darkened toward black, which is wrong on a light theme (it now washes
  toward the theme's ground), and `cells` took one gap for both axes.

## What the evaluation supports

- The vocabulary removes the mechanical part of the work: code size
  roughly halves, hand-written helpers go to zero, and a film's look
  becomes one line. This held in every comparison.
- Fresh agents can make long, structured, correct films from the skill
  alone, in about ten minutes and 170k tokens each, across explainers,
  mathematics, data, type and paint.
- The vocabulary is discoverable: authors found and used motions,
  boxes, themes, storyboards, charts, code panels and diagram words
  without help, and reported no library word producing wrong output.

## What it does not support

- That films are written faster or with fewer tokens. On a small task
  they were not.
- That films are better. Authors' self-scores did not move, and no
  independent viewer judged anything. Nobody, human or agent, watched
  a film play: every agent judged motion from stills.
- That `script` helps. It was never chosen.
- Any claim about reliability: each cell of each table is one run.

## What to run next

- Each comparison five times, to see whether the differences in fix
  rounds and time are real.
- Films scored blind by a reader who did not write them, from the
  rendered video and not from stills.
- The modification task on a film written as a `script`, to measure
  what experiment 3 could only infer.
- A larger fixed brief (60 seconds, three sections), where reading the
  skill is a smaller share of the cost and the vocabulary has room to
  matter.
