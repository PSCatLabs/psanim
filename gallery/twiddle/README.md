# Twiddle

A 23 second short (1280x720, 30 fps): Twiddle, the twiddle project's
mascot, is asleep in a dark kitchen beside an old Bakelite set. The set
warms up and is tuned through the hiss, its needle stopping between 1610
and 1620 AM, the gap he was born in. He wakes with a start on the first
beat and dances through seven bars, hopping higher as the song gets
louder and grinning `^ ^` on its big kicks. Then the set goes off. He
settles, his eyes droop, and he is asleep where he stands, with a z.

## What it is for

It shows the catalog's `/twiddle` and the `pkfigure` rig under him. He
is placed once, as a component, and given the song:

```postscript partial
/tw /twiddle << /Params << /From tuned /Until off /Accents song /Level 1 >> >> instance
```

`song` is a list with one loudness per beat. The speaker's breathing
and the notes it sends up are drawn from the same list on the same
clock, so everything he does answers something you can see. That is his
rule: nothing may look like a reaction to music unless it is one. Before
`/From` and after `/Until` there is no music, and the picture of him is
still.

## Techniques

- **A pose from the beat, then a drawing.** Inside the entry, the beat
  (index, seconds since it, loudness) goes to `dancepose`, which returns
  numbers: hop, squash, lean, arms, feet, eyes, mouth and how far the
  bobble lags. `squashed` bounces the drawing about his feet, and `limb`,
  `eye` and `mouth` draw the parts.
- **Pure frames.** The original critter in the terminal keeps springs and
  eased values from frame to frame. Here every one is a closed form in
  the time since the last beat: the arms ease with `1 - e^(-14 t)`, and
  the bobble is `spring`, a damped cosine. Any frame renders on its own.
- **Settling and waking** are `blendpose` between the dance and
  `restpose`.
- **The set**: the hiss is `hash01` on the frame, the needle an eased
  sweep, and the lamp's glow two dozen translucent discs.

A median of about 9 ms a frame at full size over the 48 frames `psanim bench` times; the Twiddle entry alone measured about 0.6 ms at 400x400.

## Render

```sh
psanim check  gallery/twiddle/twiddle.ps
psanim sheet  gallery/twiddle/twiddle.ps --from 2.4 --to 4.4 -o wake.png
psanim render gallery/twiddle/twiddle.ps -o twiddle.mp4
```
