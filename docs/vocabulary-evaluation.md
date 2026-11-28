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
| transitions between storyboard beats | birthday, circle-area, how-it-works | built later: `[seconds /transition]` between two beats of a `storyboard` (how-it-works now uses it) |
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

## 4. Three briefs on Opus, Sonnet and Haiku

Experiments 1 to 3 all ran on one model (Fable 5.2). The films this
tool is for will mostly be written by Opus and Sonnet, so after the
skill was revised (a template film, "write it to be changed", a table
of the most used words) three new fixed briefs were each given to a
fresh Opus, Sonnet and Haiku agent: a binary-search explainer (24 s), a
compound-interest data story (22 s) and a kinetic-type square (15 s).
The agents were told the film would be edited later by someone else.

The nine films were then scored **blind by a different model** (Codex),
which saw them as A, B and C per brief with a contact sheet, four
full-size frames and the video to pull frames from, and scored
legibility, composition, motion and overall out of 10 against the
brief's numbered beats. In its scale 8 is acceptable professional work
and 6 to 7 needs revision.

| | Opus | Sonnet | Haiku |
|---|---|---|---|
| finished, `check --all` without errors, rendered | 3 of 3 | 3 of 3 | 3 of 3 |
| reviewer's overall (binary search, interest, type) | 8, 8, 7 | 7, 7, 8 | 6, 7, 6 |
| mean overall | 7.7 | 7.3 | 6.3 |
| ranked first of three | 2 briefs | 1 brief | 0 |
| ranked last of three | 0 | 0 | 3 briefs |
| author's own score | 7, 8, 7 | 7, 8, 8 | 7, 7, 7 |
| tokens | 86k, 77k, 90k | 95k, 94k, 99k | 116k, 111k, 121k |
| wall time, minutes | 2.2, 1.7, 2.6 | 2.7, 2.1, 2.7 | 3.5, 3.5, 3.8 |
| code lines | 127, 71, 75 | 123, 76, 87 | 96, 53, 74 |

Measured from the nine sources: **none contains a literal coordinate
box, none sets a raw colour, and every one is built on a storyboard or
a script.** Both kinetic-type films from Opus and Sonnet use no
absolute-time word (`enters`, `onstage`, `during`) at all; the other
seven mix relative steps with a few absolute times. That is the change from experiments
1 to 3, where no agent wrote a script; whether the revised skill, the
sentence in the brief ("this film will be edited later") or the
different models caused it, this design cannot say.

What the reviewer marked down, across models: a closing entrance that
eats into a required hold (seven of the nine films), static pauses, faint secondary
text, oversized headings. Specific to the weakest films: a ruled-out
cell that kept its highlight (Haiku), a word that passed through
another on its way in (Haiku).

What this shows and does not:

- **All three models can make a correct, structured film from the skill
  alone**, including Haiku, in two to four minutes. Haiku's films are
  the ones a reviewer would send back.
- **Quality orders as the models do**, by about one and a half points
  from Opus to Haiku. The reviewer never preferred Haiku.
- **Authors cannot see the difference.** Self-scores are 7 to 8 for
  every film on every model; the blind scores range from 6 to 8. A
  reviewer that is not the author is worth having in the loop.
- **The smaller the model, the more it costs to run to completion** in
  tokens and time, though not necessarily in money.
- There is **no core-only control** on these models, so this says
  nothing about how much the libraries helped them. One reviewer, one
  run per cell, judged from stills and extracted frames.

The runs also found one real bug (`plotmark` needed a library that
`pkchart` did not load, hit by two agents) and led to: `ground` setting
the text colour (three agents drew black text on a dark ground),
`strikethrough`, a larger default for mark labels, `psanim api` answering
for motion names, and a list in the skill of what reviewers mark down.
The films, their sources and the review are in `out/eval4/`, and
`scripts/build_site.py` turns them into a page (`site/eval/`).

## 5. Templates for everyday explainers

Experiment 4 left Haiku's films at 6 to 7 and every author scoring
itself 7 to 8. Issue #108 asked whether a finished film to fill in would
lift the cheaper models, and at what cost. Two templates were written
(`skills/psanim/templates/poem.ps`, "explain this short poem", and
`process.ps`, "show me this process"), each a whole film with its content
in one block and the staging derived from it, and run on 2026-11-27/28.

**Method.** Two fixed briefs ([poem](briefs/poem.md): Basho's old pond,
three notes, about 38 s; [process](briefs/process.md): a six-step
deployment with a rollback, about 45 s), each in two variants with the
same content, beats and length: A starts from the template, B is written
from the skill alone. Sonnet and Haiku each ran both variants of both
briefs: eight runs, one per cell. Each run had its own directory with its
own copy of the skill; in the B copies `templates/` was removed and the
SKILL.md section on templates cut out. All eight used one frozen binary,
built from this branch before UTF-8 text (#115) reached main. Codex then
reviewed all eight blind, from anonymised folders (a contact sheet, six
full-size frames and the mp4), and scored fidelity, layout, legibility,
pacing, motion and overall out of 10. In its scale 8 or more is "show
unchanged".

**Measurements.** From the agent harness and the authors' reports:

| Brief | Model | Template | Wall time | Tokens | Edit rounds | psanim errors | Self-score |
|---|---|---|---|---|---|---|---|
| poem | sonnet | yes | 24 s | 70k | 1 | 0 | 8 |
| poem | haiku | yes | 39 s | 68k | 1 | 0 | 8 |
| poem | sonnet | no | 102 s | 83k | 3 | 2 (tween given `now`; a string-join word that does not exist) | 8 |
| poem | haiku | no | 153 s | 98k | 1 | 1 (helper `def` overwrote caller's variable) | 7 |
| process | sonnet | yes | 35 s | 60k | 1 | 0 | 8 |
| process | haiku | yes | 40 s | 68k | 1 | 0 | 8 |
| process | sonnet | no | 243 s | 106k | 5 | 1 (`mixink` operand missing) + one check --all pop | 7 |
| process | haiku | no | 214 s | 102k | 2 | 0, + tiny labels (`/Size` unit) + one check --all pop | 7 |

All eight passed `check --all` and rendered. Films ran 37.7 to 45 s.
Non-comment lines: 163 (poem) and 188 (process) in the template films,
61 to 105 from scratch.

**The blind review.** Overall scores, and sub-scores (fidelity, layout,
legibility, pacing, motion, overall):

| Brief | Film | Model | Template | Sub-scores | Overall | Reviewer's order |
|---|---|---|---|---|---|---|
| poem | P | haiku | yes | 7 7 7 7 7 7 | 7 | 3rd, tied with R |
| poem | R | sonnet | yes | 7 7 7 7 7 7 | 7 | 3rd, tied with P |
| poem | Q | sonnet | no | 7 7 9 7 7 7 | 7 | 2nd |
| poem | S | haiku | no | 7 8 8 7 7 7 | 7 | 1st |
| process | X | haiku | yes | 9 8 9 7 8 8 | 8 | 1st, tied with Z |
| process | Z | sonnet | yes | 9 8 9 7 8 8 | 8 | 1st, tied with X |
| process | Y | sonnet | no | 9 7 7 7 8 7 | 7 | 3rd |
| process | W | haiku | no | 7 7 8 7 7 7 | 7 | 4th |

P and R are pixel-identical over all 1131 frames, X and Z over all 1339:
with the template, the model made no difference to the film. Codex would
show X/Z unchanged and sent the other six back.

**What the reviewer marked down.**

- The poem template: a large unused band under the heading; commentary
  and attribution small against the poem; the card empty between notes
  (13.0 s); the closing picture complete by 27.6 s of 37.7 s; a short note
  lingering as long as a long one.
- The process template: the key step's reason leaves the foot when the
  walk moves on, though its emphasis stays; the closing line replaces the
  rollback's sentence, so the final picture has the arrow without it; the
  test step held through three sheet samples (11.9 to 17.8 s); 44.6 s
  felt leisurely.
- The from-scratch films: a quotation filling the card's width, a bold
  quotation competing with the poem, a slow opening (Q); a card empty
  across 12.67 to 13.0 s, a long tail (S); a rollback curve with no
  arrowhead, a sentence 20 px from the edge, a rationale smaller than the
  step text (W); descriptions wrapped into fragments in six narrow
  columns, explanations far from their nodes (Y).
- Across all eight: reading time generous but monotonous; secondary
  information treated worse than primary; text swaps that break
  continuity.
- All four poem films lost marks for a curly apostrophe in "Basho's",
  which the brief required in ASCII. That was an artefact: every author
  typed `'`, and the frozen binary, which predates #115, drew it curly in
  StandardEncoding. It hit all four poem films equally. The current
  binary draws `'` straight.

**What this shows and does not.**

- **A template makes the model irrelevant.** Both models produced the
  same film, pixel for pixel, in 24 to 40 seconds with no errors and one
  edit round. From scratch they took 102 to 243 seconds, 83k to 106k
  tokens, and hit four psanim errors between them.
- **The film is then as good as the template, no better.** The process
  template scored a point above the from-scratch films (8 against 7 and
  7); the poem template scored the same 7 as both and was ranked behind
  them. Round 2 (below) shows that a one-point difference, and an order
  among films that score alike, can reverse in a second review of the
  same films, so this round supports "no better than the template", not
  that either template was better or worse than writing from scratch.
- **Self-scores still do not track quality.** Every template author
  scored 8, for the films ranked first and the films ranked last.
- One run per cell, two briefs, one reviewer, motion judged from sampled
  frames. Wall times are the harness's; the authors' own clocks were
  coarser.

**What was changed as a result.** Both templates were revised against
the review (these revisions are what round 2, below, reviewed blind), and against a code review of the PR that found six defects
in validation and timing. What follows, written before round 2, is
from `check --all`, contact sheets
and full-size frames of the two brief contents, re-run through the
revised templates.

- `poem.ps`: heading and content form one group centred on the page, so
  no band is left under the heading. The commentary is set at 4.6 vmin
  (was 3.8), and the attribution follows the poem's size. Each note's
  explanation, and the closing reading, arrives a line at a time at about
  0.24 s a word. The next note's text rises in while the last fades, so
  the card is never empty (for two or three frames at each swap the two
  quotations overlap at partial opacity). A note's time follows its own
  length. The finished picture is still for the last line's reading time
  plus 2.5 s. The brief's film went from 37.7 s to 29.6 s; its picture
  is complete 4 to 4.5 s before the end (it was 10.1 s).
- `process.ps`: the key step's reason is set under its line, at the
  same size as the steps' lines (W was marked down for a smaller one),
  and stays. The path back's sentence, with its label, has a foot line of its own
  above the summary, so the finished picture keeps all three. Steps take
  0.24 s a word and half a second's rest: the brief's film went from
  44.6 s to 36.2 s, and the test step, by the template's own sums, from
  about 8.2 s to 6.2 s.
- From the code review: a word wider than the card is refused by name;
  `/loop`, which shadowed the operator and broke library helpers, is now
  `/pathback`; empty text is refused by name; reading time counts words,
  not spaces; the path back's label is measured and blamed by name; the
  default content works under every theme; a film over 90 s is refused.
  Each has a test that failed before its fix.
- **Against the two from-scratch poem films** (their sheets rendered
  with their own frozen binary, and S's card swap frame by frame): the
  revised template no longer has the defects that ranked it last. Its
  card never empties, where S's is empty from 12.6 to 13.0 s. Its
  finished picture is still for the last 4 to 4.5 s, Q's for the last 8
  to 8.5 s and S's for the last 12 to 13 s (compared frame against last
  frame, half a second apart). Its poem (8.5 vmin) is larger than Q's
  (7.2) and S's (8). It is still behind Q on the size of the commentary
  (4.6 vmin against Q's 5), and behind S on use of the page: it gathers
  heading, poem and card in the middle, with about a fifth of the height
  empty above and below, where S spreads the poem over the full height. Whether it now ranks with or above them was
  for a second blind review to say; round 2 said it ranked last.
- The skill gaps the from-scratch authors reported were checked against
  the skill text. Real, and fixed: pkdiagram's `/Size` gave no unit (now
  "canvas units, `4 vmin` not 4" in `node`'s entry and explain.md); no
  join word in the most-used table (`strcat` is a core word, not
  pktype's as one author thought; now listed); the helper-`def` warning
  lived in pitfalls.md and as a bare rule in SKILL.md (the rule now names
  the symptom and links to it). Already documented, not changed: `enters`
  rebasing `now` (its entry says so; the table now repeats it), `tween`
  taking no `now` (its entry and SKILL.md's third idea say so), and
  `check` noting a pop that `--all` makes a warning (SKILL.md says so).
  Not reproduced: `/Pulse` ending abruptly; two test edges with a
  clamped pulse passed `check --all`, and no B film still uses `/Pulse`
  to show the case.

**Round 2: the revised templates, reviewed blind** (2026-11-28,
`out/eval6/review-summary.md`). A binary built from main `e7f20c0`
(after #119 merged the revised templates and #115 the UTF-8 text). Two
Sonnet authors made the two brief films from the revised templates. The
blind set held, per brief, the revised template's film, the round-1
template's film and the two round-1 from-scratch films, all re-rendered
on the new binary (so the round-1 curly-apostrophe artefact was gone),
under new random tags; Codex reviewed them in a fresh thread. Author
side:

| Brief | Wall time | Tokens | Edit rounds | Errors | Film length | Changed outside content |
|---|---|---|---|---|---|---|
| poem, revised template | 27 s | 66k | 1 | 0 | 29.63 s (was 37.7) | nothing |
| process, revised template | 34 s | 58k | 1 | 0 | 36.2 s (was 44.62) | nothing |

All eight films passed `check --all` on the new binary. Blind scores
(fidelity, layout, legibility, pacing, motion, overall):

| Brief | Film | Sub-scores | Overall | Rank |
|---|---|---|---|---|
| poem | round-1 template | 9 8 8 7 8 | 8 | 1 ("show unchanged") |
| poem | REVISED template | 7 6 8 5 4 | 5 | 4 |
| poem | Sonnet from scratch | 8 6 8 6 8 | 7 | 3 |
| poem | Haiku from scratch | 8 8 8 6 7 | 7 | 2 |
| process | REVISED template | 9 8 8 8 8 | 8 | 1 ("show unchanged") |
| process | round-1 template | 9 7 9 6 8 | 7 | 2 |
| process | Haiku from scratch | 7 7 8 5 7 | 6 | 4 |
| process | Sonnet from scratch | 9 7 7 7 6 | 7 | 3 |

- **The process revision worked**: first, above the round-1 template,
  for exactly the intended gains (the final picture keeps the test
  rationale and the rollback explanation; 36 s against 45 s).
- **The poem revision made the template worse**: 5, last. The reviewer
  named three defects. At frame 480 (16.00 s) the outgoing and incoming
  notes are both visible in the card (both kinds, both quotes, the
  outgoing explanation): the cross-fade that replaced the empty card
  overlaps. Explanations and the closing reading are revealed line by
  line, so a whole sentence is on screen only briefly (the completed
  reading about 4 s) and a frame at about 16.3 s shows the unfinished
  "The poem ends on a sound, and,". Curly quotation marks were added
  around quoted words the brief gave in ASCII.
- **Reviewer noise is about one point and can reverse a ranking.** The
  round-1 poem template's film scored 7 and ranked last of four in
  round 1; the same source, re-rendered, scored 8 and ranked first in
  round 2. Part of that is the removed apostrophe artefact (its fidelity
  went 7 to 9; the two from-scratch films' 7 to 8), part is the
  reviewer weighing the same things differently (round 1: "large unused
  band", "commentary small"; round 2: "clean composition"). The round-1
  process template's film went 8 to 7. One review of one run cannot
  rank films a point apart; differences of two or more (the revised poem
  at 5, the Haiku process film at 6) are more believable. The rankings
  in round 1 above rest on such differences, and on ties.
- Still true in both rounds: with a template the author's work is 25 to
  40 seconds with no errors, and every film that is not a template film
  has pacing defects (long closing holds: about 9 s and about 12 s for
  the two from-scratch poem films; slow reveals).

**A third poem revision** (issue #108, after round 2). `process.ps` is
unchanged. `poem.ps` goes back to the round-1 film's presentation and
composition and keeps what round 2 did not criticise:

- The round-1 layout: the heading in a band at the top, the poem in the
  left column, the card beside it centred on the poem.
- Each note comes onto the card whole: its kind, its words and its
  explanation fade in together (0.25 s), stay for their reading time
  (0.24 s a word, words and explanation) and a second more, and fade out
  (0.2 s); the marked words in the poem light and dim with it. The next note starts the moment the last has gone. The
  card's frame, and a rule under the kind, stay from the first note to
  the end. The reading comes in the same way, whole.
- The card adds no quotation marks: the words appear as written, in the
  accent colour of the marked words in the poem.
- Kept from round 2's version: the commentary at 4.6 vmin (smaller,
  down to 4, only when a note or the reading would not otherwise fit),
  a note's time following its own length, a 90 s ceiling, and every
  validation fix with its test.
- The underlines under the marked words leave as the reading comes, so
  the finished picture is the poem and the reading (round 2 marked the
  round-1 film down for three underlines left in it).
- The finished picture holds for the reading's time and 2 s more, at
  least 3 s: the brief's film runs 30.4 s, its picture still for its
  last 7.27 s (round 2's film 4.17 s, round 1's 10.10 s; all three by
  the hold test below, frames compared at 160 px wide).

The defect classes are now tests in `crates/psanim/tests/templates.rs`,
run on all seven poem films (the template, the brief's content as
`poem-basho.ps`, five test contents; four of the six themes). The three
card tests render every frame at 480 px wide and read the card's column;
the hold test compares frames at 160 px; the quotation-mark test reads
the template's code:

| Test | Fails when | Round-2 template (main `e7f20c0`) | Round-1 template (`3ae7b11`) |
|---|---|---|---|
| `the_card_never_shows_two_notes_legibly_at_once` | in a handoff, the outgoing text is still legible in place while other ink is legible | fails: 1 to 3 frames at every handoff, 50 in all; the brief's at frames 334-335, 479-480, 639-640 | fails on one frame of one film, where the reading is still rising alone (a false positive of the measure on rising text) |
| `the_card_is_never_empty_for_more_than_a_blink` | no legible ink on the card for more than 2 frames | passes (its handoffs overlap instead) | passes (one empty frame per handoff, the frame both reviews caught at 13.0 s) |
| `no_sentence_is_shown_unfinished` | a text sits still, unfinished, for more than 6 frames before it is whole | fails on 27 of the 28 texts of the seven films, for 1.2 to 6.7 s each | passes |
| `the_finished_picture_holds_as_long_as_the_reading_takes_to_read` | the picture is still for less than the reading's time (0.24 s a word) + 1.5 s or more than + 3.5 s | fails on all seven: 2.7 to 4.2 s | fails on all seven: 5.6 to 12.8 s |
| `the_poem_card_adds_no_quotation_marks` | the template's code adds a quotation mark | fails (adds curly marks) | fails (adds straight marks) |

The third revision passes all five. The empty-card test cannot fail on
the round-2 template, whose defect was the opposite one, nor on the
round-1 template; it holds the revision to at most two frames between
notes and would catch a blank like the 12 frames (12.60 to 13.00 s) the
reviews found in a from-scratch film. The revision still has, at some
handoffs, one frame with no legible text on the card (frames 335, 501
and 682 of the brief's film): the kind of frame both reviews caught in
the round-1 film at 13.0 s. The overlap test's false positive on the
round-1 template shows that its measure can misfire on text that rises
into place; the revision's card text does not move.

**Round 3: the third poem revision, two independent blind reviews**
(2026-11-28, `out/eval7/review-summary.md`). A binary built from main
`c85baae` (after #123). The third revision's film is `poem-basho.ps`
rendered as is (30.4 s). Each review held four films: the third
revision, the round-1 template's film, and the Sonnet and Haiku
from-scratch films, all re-rendered on the new binary and all passing
`check --all`. The two reviews ran in fresh Codex threads, each with its
own random tags and order, and each was asked to decode frames around
every change of text.

| Film | Review 1 (overall, rank) | Review 2 (overall, rank) |
|---|---|---|
| Template v3 | 8, 1st ("no required revision") | 8, 1st ("can ship") |
| Template v1 | 7, 3rd | 7, 2nd |
| Haiku from scratch | 7, 2nd | 7, 3rd |
| Sonnet from scratch | 6, 4th | 6, 4th |

Sub-scores (fidelity, layout, legibility, pacing, motion, overall):

| Film | Review 1 | Review 2 |
|---|---|---|
| Template v3 | 9 8 8 8 8 8 | 10 8 8 8 8 8 |
| Template v1 | 9 6 8 7 8 7 | 10 7 7 6 8 7 |
| Haiku from scratch | 9 7 8 6 7 7 | 10 7 7 6 6 7 |
| Sonnet from scratch | 9 7 9 5 8 6 | 10 6 8 5 6 6 |

- **The third revision ranked first in both reviews.** The two reviews
  agree on every overall score and on first and last place; they swap
  second and third, both at 7. Sub-scores differ by at most one, except
  the Sonnet film's motion (8 and 6).
- **The round-2 defects are gone.** Neither review reported an overlap,
  a fragment or an added quotation mark.
- **Defects still named for the third revision, all judged minor:**
  review 1 found the outgoing "turn" note caught mid-fade as ghost text
  at about 16.72 s, a small category label and muted attribution, and
  the closing paragraph's last line close to the card's right padding.
  Review 2 found an empty upper-middle area, "instant." alone on the
  last line of the turn note, and the category label lighter than the
  quoted words.
- **Noise, measured again.** Over three rounds and four reviews the same
  round-1 template film scored 7 (round 1, with the apostrophe artefact),
  8 (round 2), and 7 and 7 (round 3). A single review is good to about a
  point; two reviews reduce that but do not remove it.
- **For #108:** both templates now rank first in their sets under blind
  review: the process template's second version at 8 (round 2, one
  review), the poem template's third version at 8 and 8 (round 3, two
  reviews).

## What the evaluation supports

- The vocabulary removes the mechanical part of the work: code size
  roughly halves, hand-written helpers go to zero, and a film's look
  becomes one line. This held in every comparison.
- Fresh agents can make long, structured, correct films from the skill
  alone, in about ten minutes and 170k tokens each, across explainers,
  mathematics, data, type and paint.
- A finished template takes the model out of the result: Sonnet and
  Haiku made the same film, pixel for pixel, in well under a minute
  (experiment 5).
- The vocabulary is discoverable: authors found and used motions,
  boxes, themes, storyboards, charts, code panels and diagram words
  without help, and reported no library word producing wrong output.

## What it does not support

- That films are written faster or with fewer tokens. On a small task
  they were not.
- That films are better. Authors' self-scores did not move, and no
  independent viewer judged anything. Nobody, human or agent, watched
  a film play: every agent judged motion from stills.
- That `script` helps. In experiments 1 to 3 it was never chosen; in
  experiment 4, after the skill argued for it, most films used it, but
  no edit was then made to them to measure the benefit.
- Any claim about reliability: each cell of each table is one run.
- That templates make better films in general. Two templates, each
  revised after blind review, now rank first in their sets, one on a
  single review and one on two; that is two briefs, one run each. A
  template is only as good as itself: the poem template's second
  version ranked last.
- Any claim about smaller models with and without the libraries: only
  the with-libraries arm was run on Opus, Sonnet and Haiku.

## What to run next

- A second independent review of the process template's second
  version, which has one review so far.

- Each comparison five times, to see whether the differences in fix
  rounds and time are real.
- Films scored blind by a reader who did not write them, from the
  rendered video and not from stills.
- The modification task on a film written as a `script`, to measure
  what experiment 3 could only infer.
- A larger fixed brief (60 seconds, three sections), where reading the
  skill is a smaller share of the cost and the vocabulary has room to
  matter.
