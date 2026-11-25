#!/usr/bin/env python3
"""Write day.csv: a synthetic but plausible day on a 20-machine build farm.

Nothing here was measured: it is made up, deterministically (fixed seed),
so the film is reproducible. Run it from anywhere:

    python3 gallery/buildfarm/gen_day.py      # rewrites gallery/buildfarm/day.csv

The story of the day (2026-11-18, a Wednesday):
  02:00-04:30  the nightly builds keep every machine busy
  09:00        developers arrive; load climbs through the morning
  11:10        lin-07 drops off the network for 50 minutes; the queue grows
  12:30        lunch: a dip
  13:20-14:40  a runaway test pins win-02 at 100%
  16:30        a release candidate: everything at once
  18:00        evening tail-off

Columns: CPU % per builder; queue (jobs waiting); builds and failed
(totals so far today); p95 (build time, minutes); cache (hit rate %);
util (mean CPU %); rate (builds in the last hour); incident! (events).

Time is in movie seconds: the day is replayed in 60 s, so 1 s is 24
minutes and a row every 2 minutes is a row every 1/12 s.
"""
import math
import random
from pathlib import Path

SEED = 1118
STEP_MIN = 2
MIN_PER_SEC = 24.0
HOSTS = [f"mac{i:02d}" for i in range(1, 5)] + [f"lin{i:02d}" for i in range(1, 13)] + [f"win{i:02d}" for i in range(1, 5)]


def bump(h, centre, width):
    return math.exp(-((h - centre) / width) ** 2)


def demand(h):
    """How much work is waiting to be done at hour h, 0..~1.3."""
    # the nightly builds: a ramp up at 02:00, fading out by 04:30
    nightly = 0.95 / (1 + math.exp(-(h - 2.05) * 9)) * (1 - 1 / (1 + math.exp(-(h - 3.9) * 3.5)))
    office = 0.75 / (1 + math.exp(-(h - 9.6) * 2.2)) * (1 - 1 / (1 + math.exp(-(h - 18.2) * 1.6)))
    lunch = -0.28 * bump(h, 12.6, 0.45)
    afternoon = 0.22 * bump(h, 15.0, 1.4)
    rc = 0.55 * bump(h, 16.7, 0.35)
    return max(0.02, nightly + office + lunch + afternoon + rc + 0.04)


def main():
    rng = random.Random(SEED)
    rows = []
    queue = 0.0
    builds = 0.0
    failed = 0
    # Each host has its own share of the work and its own noise.
    share = {h: 0.85 + 0.3 * rng.random() for h in HOSTS}
    phase = {h: rng.random() * 6.28 for h in HOSTS}
    noise = {h: 0.0 for h in HOSTS}
    events = []
    history = []
    for k in range(0, 24 * 60 // STEP_MIN + 1):
        m = k * STEP_MIN
        h = m / 60.0
        d = demand(h)
        cpus = {}
        for host in HOSTS:
            noise[host] = 0.85 * noise[host] + 0.15 * rng.gauss(0, 1)
            kind = host[:3]
            weight = {"mac": 0.9, "lin": 1.0, "win": 0.8}[kind]
            v = 66 * min(1.15, d * share[host] * weight) + 14 * noise[host] + 6 * math.sin(h * 2.1 + phase[host])
            v = 4 + v
            # Spikes: a build landing on this machine.
            if rng.random() < 0.04 * d:
                v += 25
            if host == "lin07" and 11 + 10 / 60 <= h < 12:
                v = 0
            if host == "win02" and 13 + 20 / 60 <= h < 14 + 40 / 60:
                v = 100
            if host == "mac03" and 15.5 <= h < 16.2:
                v = min(v, 12)  # waiting on a full disk
            # an idle machine still shows a little load; only an unreachable one reads 0
            cpus[host] = 0.0 if host == "lin07" and 11 + 10 / 60 <= h < 12 else max(1.0, min(100.0, v))
        up = sum(1 for c in cpus.values() if c > 0.5)
        capacity = up * 0.66
        arrivals = 13.5 * d + rng.random() * 1.2
        done = min(0.8 * queue + arrivals, capacity * (0.6 + 0.4 * min(1.0, d)))
        queue = max(0.0, queue + arrivals - done)
        builds += done
        fails = sum(1 for _ in range(int(done)) if rng.random() < (0.035 + (0.08 if 16.3 < h < 17.2 else 0)))
        failed += fails
        p95 = 6.5 + 7 * min(1.2, d) + 0.25 * queue + rng.gauss(0, 0.4)
        cache = 92 - 18 * bump(h, 16.7, 0.4) - 9 * bump(h, 2.3, 0.4) + rng.gauss(0, 0.8)
        if k in (int((11 + 10 / 60) * 60 / STEP_MIN), int(13.34 * 60 / STEP_MIN), int(16.5 * 60 / STEP_MIN)):
            events.append(k)
        util = sum(cpus.values()) / len(cpus)
        history.append(builds)
        rate = builds - history[max(0, len(history) - 1 - 60 // STEP_MIN)]  # builds in the last hour
        rows.append((m / MIN_PER_SEC, cpus, queue, builds, failed, p95, cache, util, rate, "x" if k in events else ""))
    out = Path(__file__).with_name("day.csv")
    with out.open("w") as f:
        f.write("# A synthetic day on a build farm, made by gen_day.py (seed %d). Not measured.\n" % SEED)
        f.write("# t is movie seconds: the day 2026-11-18 from 00:00, 1 s = 24 min, a row every 2 min.\n")
        f.write("t," + ",".join(HOSTS) + ",queue,builds,failed,p95,cache,util,rate,incident!\n")
        for t, cpus, q, b, fl, p, c, u, r, ev in rows:
            f.write(f"{t:.4f}," + ",".join(f"{cpus[h]:.1f}" for h in HOSTS) + f",{q:.1f},{b:.0f},{fl},{p:.2f},{c:.1f},{u:.1f},{r:.0f},{ev}\n")
    print(f"wrote {out} ({len(rows)} rows)")


if __name__ == "__main__":
    main()
