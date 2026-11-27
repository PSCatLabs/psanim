# Hello, world

A 32 second piece of kinetic type (1280x720, 30 fps). "Hello, world"
writes itself on letter by letter and waves, then moves up to be the
heading. Twelve greetings follow, each in its own language and script
(Español, Français, Português, Українська, Ελληνικά, Türkçe, Polski,
Tiếng Việt, Kiswahili, Te reo Māori, isiZulu, Svenska), each with its
language's name and flag. Next comes one wave in five skin tones, with a
row of joined emoji sequences. Then the film names the scripts it cannot
draw yet, and closes on the title over a turning world.

## What it is for

It shows psanim's text taking UTF-8. Every string in the file is written
as it reads, with no escapes, sign tables or pictures of letters:

```postscript partial
[ (Γειά σου)  (Ελληνικά)       (🇬🇷) ]
/hands [ (👋🏻) (👋🏼) (👋🏽) (👋🏾) (👋🏿) ] def
/joined [ (👩🏽‍💻) (🧑🏿‍🚀) (👨‍👩‍👧) (🏳️‍🌈) (🧑🏻‍🎨) ] def
```

`face` selects fonts that read UTF-8. A character a font's own face lacks
comes from PSCat's fallback faces (Liberation, Symbol, ZapfDingbats), and
an emoji comes from its colour emoji. `fitface`, `showin` and `placetext`
measure exactly what is drawn, so each greeting is fitted and centred
with the words a film uses for ASCII. `kinetic` brings each greeting in
`/glyphs` at a time. It steps by grapheme cluster, so `ã` and `ά` arrive
as one letter each, and a flag or a family is never split.

## Where it stops, said in the film

PSCat has no face for Korean, Japanese, Thai, Chinese, Arabic or the
Indic scripts. A character from any of them draws as an empty box
(.notdef). The fourth beat names three of these scripts on empty cards,
with each greeting in Latin letters, instead of drawing boxes where the
letters would go. If one of those strings were shown, `psanim check`
would warn about each character, for example:

```
warning: no glyph for U+C548 (안) in /Helvetica or its fallback faces; drew .notdef. ...
```

The film draws no Hebrew either. PSCat has Hebrew letters but sets them
left to right, because it does no bidi.

## Techniques

- **Timing from the data.** The greetings share the `/greet` beat
  equally (`/per`), so adding a greeting shortens each one's turn
  rather than overrunning the beat. The dots at the bottom count
  through the greetings on the same clock.
- **The title is one drawing in three places.** `titlebox` interpolates
  between the centre box and the heading box, and `fitface` sizes the
  title to whichever box it is in, so the move up is a single `between`.
- **The turning world** is the three globe emoji (🌍 🌎 🌏), each
  dissolving into the next inside `faded`: once very faint behind
  everything, and once in full colour at the close. In the last second
  the world stops turning and the hand comes to rest, so the film ends
  on a still picture.
- The wave is a `👋` turned about its wrist: three quick waves as it
  arrives, settling into a slow sway.
- `check --all` flags two one-tile "abrupt changes", at 0.7 s and 5.1 s.
  These are the tall `l` of "world" and the `j` of "Bonjour" beginning
  their `/mask-up` entrance. They are intended.

## Credits

Emoji graphics by Twemoji (Twitter, Inc and other contributors),
licensed CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/). The
closing frame says so too. The text is set in PSCat's built-in faces.

## Render

```sh
psanim check  gallery/hello-world/hello-world.ps
psanim sheet  gallery/hello-world/hello-world.ps --count 16 --cols 4 -o sheet.png
psanim render gallery/hello-world/hello-world.ps -o hello-world.mp4
```
