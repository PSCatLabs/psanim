# End Credits

The closing credits of an imaginary film, *The Lamplighters*: 63 s at
1280x720, 30 fps. Two cards hold; then the names roll upward at a constant
sixty units a second, two whole pixels a frame, over a night coast where a
lighthouse beam sweeps slowly across the stars; the roll slows to its stop
on the film's mark, centred, and the last card holds. Every name in it is
invented.

## What it is for

It is the showcase for `credits` (pktype): one call sets the whole roll,
and the rest of the file is the world behind it. The movie's own length
comes from the credits (`creditsduration`, called before `animation`), not
the other way round, so adding a line to the roll lengthens the film.

## Techniques

| psanim word | What it does here |
|---|---|
| `credits` | The roll: headings, roles right-aligned and names left-aligned on a gutter, centred names, notes, `[/space n]`, and a `[/proc ...]` entry (the studio's lamp). |
| `/Cards`, `/CardTime` | Two cards that fade in and hold before the roll starts. |
| `/Final`, `/FinalGap`, `/Settle` | The film's mark rides the same roll, which slows over its last 150 units and stops with it centred. |
| `creditsduration` | `<< /Duration roll opts creditsduration 6 add ... >> animation`: the cards, the roll to its stop and six held seconds. |
| `usetheme` | A custom theme (AvantGarde faces, a lamp-amber accent): the credits take every face and ink from the theme's roles. |
| `/Speed` as whole pixels a frame | 60 units a second at 30 fps and scale 1 is exactly 2 px a frame, so the roll has no 3-and-4 pixel judder as well as no shimmer. |
| `hash01`, `globaltime` | Stars that drift and twinkle, and the horizon's glow that fades as the film goes on. |
| `mixink`, `vignette`, `filmgrain` | The sky's gradient and the filmic surface. |

The background is pure: every frame is a function of its time. The roll
costs about 0.9 ms a frame at 720p, layout included (`psanim bench` on the catalog entry); the whole film takes a median 8.7 ms a frame.

## Render

```sh
psanim check  gallery/end-credits/end-credits.ps
psanim sheet  gallery/end-credits/end-credits.ps -o out/end-credits/sheet.png
psanim render gallery/end-credits/end-credits.ps -o out/end-credits/end-credits.mp4
```

`check` notes that the film opens from the background: the first card fades in.
