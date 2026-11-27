#!/usr/bin/env python3
"""Every number shown in catching-up.ps, from life-expectancy.csv. Standard library only.

    python3 data.py          # prints the facts, then the PostScript data block

life-expectancy.csv is Our World in Data's "Life expectancy" chart
(https://ourworldindata.org/grapher/life-expectancy), downloaded on
2026-11-27 for China, Japan, South Korea and the United States, 1960-2023.
From 1950 OWID takes these values from the UN World Population Prospects
(2024). Period life expectancy at birth, in years.
"""
import csv
from pathlib import Path

HERE = Path(__file__).resolve().parent
ORDER = ["South Korea", "Japan", "United States", "China"]
PS_NAME = {"South Korea": "kor", "Japan": "jpn", "United States": "usa", "China": "chn"}

series = {name: {} for name in ORDER}
with open(HERE / "life-expectancy.csv", newline="") as f:
    for row in csv.DictReader(f):
        series[row["entity"]][int(row["year"])] = float(row["life_expectancy_0"])

years = sorted(series["South Korea"])
assert years == list(range(1960, 2024)), years
for name in ORDER:
    assert sorted(series[name]) == years, name

kor, usa, jpn, chn = (series[n] for n in ["South Korea", "United States", "Japan", "China"])

print("Facts shown on screen")
print(f"  South Korea 1960: {kor[1960]:.1f}   2023: {kor[2023]:.1f}   gain: {kor[2023] - kor[1960]:.1f} years")
print(f"  United States 1960: {usa[1960]:.1f}   2023: {usa[2023]:.1f}")
print(f"  gap 1960 (US - Korea): {usa[1960] - kor[1960]:.1f}   gap 2023 (Korea - US): {kor[2023] - usa[2023]:.1f}")
level = [y for y in years if abs(kor[y] - usa[y]) < 0.05]
print(f"  years within 0.05 of each other: {level}  ({kor[2001]:.4f} against {usa[2001]:.4f} in 2001)")
behind_after = [y for y in years if y > 2001 and kor[y] <= usa[y]]
print(f"  years after 2001 with Korea not ahead of the US: {behind_after or 'none'}")
ahead_before = [y for y in years if y < 2001 and kor[y] >= usa[y]]
print(f"  years before 2001 with Korea level or ahead: {ahead_before or 'none'}")
print(f"  Japan 2023: {jpn[2023]:.1f}   China 1960: {chn[1960]:.1f}   China 2023: {chn[2023]:.1f}")
print(f"  ends 2023, Japan - Korea: {jpn[2023] - kor[2023]:.2f}   US - China: {usa[2023] - chn[2023]:.2f}")
print()
print("% ---- data: printed by data.py (life expectancy at birth, years, 1960..2023) ----")
for name in ORDER:
    vals = [f"{series[name][y]:.2f}" for y in years]
    lines = [" ".join(vals[i:i + 16]) for i in range(0, len(vals), 16)]
    print(f"/{PS_NAME[name]} [")
    for line in lines:
        print(f"  {line}")
    print("] def")
