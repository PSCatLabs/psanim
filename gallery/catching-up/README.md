# Catching Up

A 29.5 second data story (1280x720, 30 fps): life expectancy at birth in
South Korea, Japan, the United States and China, 1960 to 2023. One point:
South Korea started sixteen years behind the United States, was level
with it in 2001, and ended five years ahead, within half a year of Japan.

1. **The headline and the axes**: in 1960 a baby in South Korea could
   expect 54 years.
2. **The years sweep across**: all four lines are drawn up to the same
   year, and a tracer on South Korea's line leads the pen with its
   reading. As the lines reach 2023 each is named at its end.
3. **A hold** on the four named lines.
4. **Level in 2001**: a bracket shows the 16.1 year gap in 1960, and the
   tracer goes back to 2001, where South Korea and the United States both
   read 76.9.
5. **Today**: the tracer comes forward to 2023, 84.3 years, and holds.

## The data

`life-expectancy.csv` is the [Our World in Data "Life
expectancy"](https://ourworldindata.org/grapher/life-expectancy) chart,
downloaded on 2026-11-27 for the four countries, 1960 to 2023. From 1950
Our World in Data takes these figures from the **UN World Population
Prospects (2024)**; the chart's metadata cites "Riley (2005); Zijdeman et
al. (2015); HMD (2025); UN WPP (2024), with major processing by Our World
in Data". The measure is period life expectancy at birth: how long a
baby born that year would live at that year's death rates.

`data.py` (Python standard library) reads the CSV, prints every number
the film states, and prints the data block at the top of the film, each
value rounded to two decimals.

| On screen | From the data |
|---|---|
| 54 years in 1960; 84 and 84.3 in 2023; 30.6 more | South Korea 53.76 in 1960, 84.33 in 2023: 30.57 more |
| 16.1 years behind | United States 69.83 minus South Korea 53.76, in 1960 |
| level in 2001, 76.9 in both | 76.9195 and 76.9194: the only year within 0.05 |
| ahead every year since | South Korea above the United States in every year 2002 to 2023 (checked by `data.py`) |
| 5.0 years ahead in 2023 | 84.33 against 79.30 |
| within half a year of Japan | Japan 84.71 in 2023, 0.38 more |
| China, 1960 and 1961: 33.4 and 40.2 (famine), below the axis | China's values for those years. The y axis starts at 50 so the story is not squeezed into the top of the chart; China's line enters from below it in 1962 (51.0), and the note says what is cut. Our World in Data's description of the series names this famine, the Great Leap Forward, as one of the large post-war drops |

While the tracer moves it reads the line between data points; its
reading is the value at the year it writes, so a year is never shown
beside another year's value.

## Techniques

- **`serieslabels`** names each line at its end, in its colour (line i is
  drawn in `chartroles i get`, the word's default), so there is no
  legend. Japan and South Korea end 0.4 years apart and the United States
  and China 1.3: the names are moved apart, in order, with short leaders.
  `/X year` makes each name arrive as its line reaches 2023.
- **`tracer`** rides South Korea's line at a year from `keyframes`, with
  a drop line to the axis and a readout (`/XDecimals 0 /Decimals 1
  /Suffix ( years)`) that moves to the left of the dot near the frame's
  right edge.
- The lines are drawn up to a year by clipping at that year's
  `plotpoint`, so all four ends share one x however steep a line is.
- `storyboard` beats; headlines on the film's clock with `onstage`.

## Commands

```sh
python3 data.py                                   # the numbers and the data block
psanim check  catching-up.ps --all
psanim sheet  catching-up.ps --count 12 --cols 4 -o sheet.png
psanim frame  catching-up.ps --at 19 -o poster.png
psanim render catching-up.ps -o catching-up.mp4
```
