# Departures

A 26 second film (1280x720, 30 fps): a station departures board updating.
It fills from blank, trains are called to board, one is delayed and one
cancelled, and the first leaves, which moves every row up one and turns
most of the board over at once. A second, smaller board is the clock
above it; an odometer below counts the day's passengers. The stations
are invented.

## What it is for

It is the first piece made only of catalog entries (#48): three
`/splitflap` boards (the title, the clock, the services) and one
`/odometer`. The movie chooses what they say and when, and nothing else:

```postscript partial
/board /splitflap << /Seed 3 /Params << /States services /Drum drums /Cell [32 50] >> >> instance
/count /odometer << /Params << /From 18240 /To 19870 /Over 24 /Ease /linear >> >> instance
```

`services` is a list of `[ time rows ]`: each row is a 32 character
string, built by a small `row` helper from the time, destination,
platform and remarks, so the film's data reads like a timetable. At each
time every cell whose character changes flips forward through its drum
to the new one, a little late by its own random share of a third of a
second; a character that needs 25 flaps takes about 1.8 seconds, one
that needs 1 takes a tenth. `drums` gives the time columns a drum of
digits and a colon, so "09:15" never rattles through the alphabet.

## Things worth knowing

- A cell's state is computed from its own list of changes at the time
  asked, so any frame renders alone. A frame in the middle of the big
  turn-over costs about 10 ms at preview size, a resting one about 4 ms.
- Names a movie defines are visible inside the components it places
  (they share the movie's dictionary below their own). The first version
  of this film defined its own `scale` helper and broke the odometer's
  `scale`: call helpers something else (`putflap` here).

## Render

```sh
psanim check  gallery/departures/departures.ps
psanim sheet  gallery/departures/departures.ps -o out/departures/sheet.png
psanim render gallery/departures/departures.ps -o out/departures/departures.mp4
```
