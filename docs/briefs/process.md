# Brief: "Show me the deployment process"

A fixed brief for comparing authors (issue #108), in the style of the
briefs in [vocabulary-evaluation.md](../vocabulary-evaluation.md). It is
run twice on each model: once **with the template** (variant A) and once
**without** (variant B, the skill only). Both variants ask for the same
film, with the same content, beats and length.

## The request, as a user would put it

> Show me our deployment process: commit, build, test, stage, approve,
> release, and the rollback if a release goes wrong.

## The content (both variants)

Use it exactly as written, in ASCII.

- Heading: `How a change reaches production`
- Six steps, in order, each a name and one line:
  1. `commit`: "A developer pushes a change to the main branch."
  2. `build`: "The pipeline compiles the code and packages it."
  3. `test`: "Automated tests run against the new package."
  4. `stage`: "The package is deployed to a copy of production."
  5. `approve`: "A release manager checks staging and signs off."
  6. `release`: "The package goes out to every user."
- The step that matters most: `test`, because "A failing test stops the
  change before any user can see it."
- A path back: `rollback`, from `release` back to `stage`: "If errors
  rise after release, the previous package is restored."
- Closing line: "Every change passes the same six gates on its way to
  users."
- Look: a dark theme.

## Variant A: with the template

Start from `skills/psanim/templates/process.ps`. Copy it to your film's
file and put the content above into its CONTENT block (steps count from
0, so `test` is step 2, `stage` 3 and `release` 5). Do not change the
staging below the line "nothing below here needs to change" unless the
film cannot meet the brief without it; if you do, report what and why.

## Variant B: without the template

Do not open `skills/psanim/templates/`. Write the film from the skill.
1280x720, 30 fps, about 45 seconds (what the template makes of this
content), a dark theme. The beats, in order:

1. The heading arrives, top left, at a modest size, and stays.
2. The six steps appear as nodes, in order, joined by a line, before
   anything moves.
3. A token travels along the line from step to step. As it reaches each
   step, the step lights up and its one line appears beside it and stays.
   Each line is given time to be read.
4. At `test` the step is flagged in the accent colour, keeps that flag to
   the end, and the reason it matters is shown while the token is there.
5. After the token reaches `release`, the rollback path from `release`
   back to `stage` is drawn, labelled `rollback`, something travels it,
   and its line is shown. The path stays.
6. The closing line appears, and the whole picture (every step, every
   line, the flag, the rollback path) holds, still, for the last 2.5
   seconds.

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
