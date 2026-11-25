# Two Yellow Kayaks

A 41 second film (1280x720, 30 fps) of an invented text conversation: two
friends plan a dawn paddle on a lake, in a phone. The sky behind the phone
warms from night to sunrise while they talk, and the plan on the right fills
in, line by line, as each message settles it.

The conversation, whose script is the first thing in the file:

> are you free saturday? / depends. what are we doing? / sunrise kayaking at
> the lake. I found two kayaks for $20 / that is suspiciously cheap / they're
> yellow. very yellow. / ...ok I'm in / 6:15 at the dock? bring coffee /
> 6:15 AM??? / the sun is worth it. pastries after / fine. but you're buying
> the pastries / deal / see you at the dock

## Techniques

- **The `chat` catalog entry**, placed as a component in a box, with a
  `/Script` of `[ sender (text) options ]`. It supplies the typing dots,
  the bubbles that size to their wrapped text, the tails, the timestamp,
  the reactions, Delivered turning to Read, the typed text in the field,
  and the smooth scroll. The film adds none of that.
- **Explicit times.** Every message has an `/At`, so the film can hang other
  things on them: each line of the plan on the right enters the moment the
  message that settles it arrives.
- **Emoji drawn by psanim**: `:eyes:` `:party:` `:laugh:` `:sad:`
  `:thumbsup:` `:check:` `:cool:` `:heart:` in the messages and as
  reactions, and `emojiat` for the stamp on the plan. A message of nothing
  but an emoji is shown large, without a bubble.
- **A sky that follows the story**: `boxgradient` between two palettes by
  one eased number, a sun rising on its own curve with `glowing`, and a lake
  with a reflection that strengthens as it climbs.
- **Kinetic title** (`kinetic`, `/mask-up`) and entrances (`enters`).

## Commands

```sh
psanim check  kayaks.ps --all
psanim sheet  kayaks.ps --count 12 --cols 4 -o sheet.png
psanim render kayaks.ps -o kayaks.mp4
```

Change the look in one line of the script: `/Theme /mint` (or `/light`,
`/dark`, or a dictionary of colours); `/Frame false` drops the phone and
shows the screen alone.
