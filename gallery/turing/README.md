# A Turing machine at work

A 55 second film (1280x720, 30 fps) of a Turing machine running two
programs. A paper tape lies on a steel bed. A head on a carriage reads the
cell under its window, stamps a mark or rubs one out, and moves one cell.
A dial shows the state, drum counters show the steps and the marks, and
the program card beside it lights the row in use.

1. **Unary addition.** The tape holds 2 + 3 as marks with a blank between
   them. The machine fills the gap, walks to the end and rubs out one mark,
   leaving five. It takes 8 steps, each slow enough to read the rule in use
   (spelled out under the deck: "In state B, reading a 1: write a 1, move
   right, go to B.").
2. **The four-state busy beaver.** It starts on a blank tape. The steps
   speed up from 1.7 s to 0.14 s, run fast through the long middle while
   the camera follows the head left, and slow down again into the halt. It
   halts after 107 steps with 13 marks, and the camera pulls back to show
   the whole tape.

## What it is for

The machine is the catalog entry `/turing`, so the film is a short host
that holds two tables and places two instances of it. The entry simulates
the table it is given when it is set up: at most `/MaxSteps` steps, with
every configuration kept. A frame finds the step its time falls in and
draws that step from the record. Tape, head, state, the lit row and both
counters all come from the run, and nothing is keyframed. A different
table gives a different film, and that film is just as correct.

```postscript partial
/tu-bbp << /Name (Busy beaver: 4 states, 2 symbols) /Table tu-beaver
  /Slow 1.7 /Fast 0.14 /Ramp 0.74 /Lead 3.4 /Tail 5.5
  /Expect << /Steps 107 /Ones 13 /Tape (10111111111111) >> >> def
/tu-bb /turing << /Params tu-bbp >> instance
```

`/Expect` is the check. If the run does not halt with that many steps,
that many marks and that tape, setup fails, and `psanim check` fails with
it. `--param report=1` prints both results:

```sh
psanim check gallery/turing/turing.ps --param report=1
#  turing: Unary addition: 2 + 3 steps 8 ones 5 halted true tape 11111
#  turing: Busy beaver: 4 states, 2 symbols steps 107 ones 13 halted true tape 10111111111111
```

`crates/psanim/tests/turing.rs` loads this file that way and asserts both
results. It also runs the entry on other tables: the two-state busy beaver,
a three-symbol binary increment, a table with a missing rule, and one that
never halts.

## Things worth knowing

- **Pacing is a function of the step number.** Step k takes
  `max(Fast, Slow * Ramp^k, Slow * Ramp^(n-1-k))` seconds, where n is the
  length of the run. One formula gives slow steps for an 8-step program,
  and a slow start, a fast middle and a slowdown into the halt for a long
  one. Setup adds up the step durations into each step's start time; the
  film is as long as the run plus `/Lead` and `/Tail`, so the host times
  itself with `componentduration`.
- **Inside a step, everything is a fraction of that step's duration**:
  the read lamp, the stamp, the move, the dial's swing and the counters.
  So at full speed they compress together.
- **The camera is a dead zone, worked out in setup, then averaged.** For
  each step, the camera moves only if the head would leave the middle of
  the frame. A frame averages that track over sixteen moments either side
  of its own time. The average looks ahead as far as it looks back, so it
  does not lag, and a fast run becomes a steady pan rather than one jerk
  per cell.
- **The rows fade like warm bulbs.** A row lights as its step begins, and
  the rows used before it fade out over about a tenth of a second. In the
  fast stretch the card therefore shows the last few rules, not a
  highlight jumping every four frames, which `check --all` would report as
  a pop.
- **A cell's symbol comes from its own list of writes.** Each cell keeps
  `[step symbol]` pairs, and a frame takes the last write before the
  current step, counting the current step's write once the stamp has
  landed. A cell that had a mark and lost it keeps a faint smudge.
- The instances are made above the declaration, because the film's length
  depends on theirs. Code above `animation` runs twice, so `--param
  report=1` makes them again below the declaration, and the report prints
  once.

## Render

```sh
psanim check  gallery/turing/turing.ps --all
psanim sheet  gallery/turing/turing.ps -o out/turing/sheet.png
psanim render gallery/turing/turing.ps -o out/turing/turing.mp4
```

A median frame at 1280x720 takes 20 ms (`psanim bench`, Apple M4).
