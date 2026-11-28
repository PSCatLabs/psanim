# Brief: "Explain this haiku"

A fixed brief for comparing authors (issue #108), in the style of the
briefs in [vocabulary-evaluation.md](../vocabulary-evaluation.md). It is
run twice on each model: once **with the template** (variant A) and once
**without** (variant B, the skill only). Both variants ask for the same
film, with the same content, beats and length.

## The request, as a user would put it

> Explain this haiku to me: Basho's old pond.

## The content (both variants)

The poem is Matsuo Basho's haiku of 1686 (*furu ike ya / kawazu
tobikomu / mizu no oto*), which is in the public domain. The English
below is a literal rendering written for this brief, not a
published translation. Use it exactly as written, in ASCII.

- Heading: `Basho's old pond`
- Attribution: `Matsuo Basho, 1686; literal rendering`
- The poem, three lines:
  1. `old pond`
  2. `a frog jumps in`
  3. `the sound of water`
- Three notes, in this order:
  1. the words `old pond` in line 1, kind `image`: "The scene is set in
     two words: something old, still and silent, before anything
     happens."
  2. the words `a frog jumps in` in line 2, kind `turn`: "The only
     movement in the poem, and it is over in an instant."
  3. the words `the sound of water` in line 3, kind `sound`: "The poem
     ends on a sound, and on the silence that comes back after it."
- Closing reading: "A brief sound makes the stillness around it
  noticeable. The pond is quiet before and after; the frog is only a
  moment."
- Look: a light, paper-like theme.

## Variant A: with the template

Start from `skills/psanim/templates/poem.ps`. Copy it to your film's file
and put the content above into its CONTENT block (lines count from 0,
so the three notes are on lines 0, 1 and 2). Do not change the
staging below the line "nothing below here needs to change" unless the
film cannot meet the brief without it; if you do, report what and why.

## Variant B: without the template

Do not open `skills/psanim/templates/`. Write the film from the skill.
1280x720, 30 fps, about 38 seconds (what the template makes of this
content), a light theme. The beats, in order:

1. The heading arrives, top left, at a modest size, and stays.
2. The three lines of the poem arrive one at a time, each given time to
   be read, with the attribution under them.
3. The walk: for each note in turn, its words in the poem change to the
   accent colour and are underlined, and a card beside the poem shows the
   note's kind, the quoted words and the explanation. The explanation
   stays long enough to read (about 0.3 s a word and a second). Each
   underline stays after its note; the colour returns to normal.
4. The card shows the closing reading.
5. The finished picture (poem, all three underlines, the reading) holds,
   still, for the last 2.5 seconds.

## What the author is given

- The skill, `skills/psanim/` (SKILL.md and what it links to).
- A frozen binary, `out/bin/psanim` (a copy of `target/release/psanim`
  that does not change during the run).
- This brief (variant A or B only, not both).

## Rules

- Read the skill: `skills/psanim/SKILL.md`, then the chapters it points
  to as you need them. Do not read `crates/`, `docs/` (beyond this
  brief), `gallery/`, `out/` (beyond your own outputs) or the git history.
- Use the frozen binary for `check`, `sheet`, `frame`, `render`, `api`.
- Write only your film file and renders beside it with the same stem. Do
  not edit anything else; do not commit.
- Loop: `check`, `sheet`, look at the PNGs (and at least two full-size
  `frame`s), fix, repeat until it is acceptable: every beat present and in
  order, no overlapping or clipped text, legible sizes, time to read,
  nothing popping. Finish with `psanim check <file> --all` clean and
  `psanim render <file> -o <stem>.mp4`.
- Stop when it is acceptable or after 25 minutes, whichever is first. If
  it does not render, say so plainly.

## The report (at most 200 words)

1. Did it finish: the `check --all` result, rendered or not.
2. Time taken, in minutes.
3. Counts: `check` runs, `sheet` and `frame` runs, edit rounds; every
   error hit, one line each with its cause.
4. Variant A: every change you made outside the CONTENT block, and why
   (or "none"). Variant B: the two things that cost you most.
5. Anything in the skill (or the template) that misled you.
6. Your own score for the film, 1 to 10.
