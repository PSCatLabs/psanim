#!/usr/bin/env python3
"""Every number shown in birthday.ps, computed here. Standard library only.

    python3 data.py          # prints the facts, then the PostScript data block

Model: 365 equally likely birthdays, no leap day, people independent.
"""
from fractions import Fraction
from math import comb
import random

DAYS = 365


def p_shared(n):
    """Exact probability that at least two of n people share a birthday."""
    q = Fraction(1)
    for k in range(n):
        q *= Fraction(DAYS - k, DAYS)
    return 1 - q


exact = [p_shared(n) for n in range(0, DAYS + 2)]
first_half = next(n for n, p in enumerate(exact) if p >= Fraction(1, 2))
first_99 = next(n for n, p in enumerate(exact) if p >= Fraction(99, 100))
naive_half = next(n for n in range(DAYS + 1) if Fraction(n, DAYS) >= Fraction(1, 2))

# The simulation: rooms of 23 random birthdays, a fixed seed so it repeats.
ROOMS = 100_000
rng = random.Random(23)
hits = []
for _ in range(ROOMS):
    room = [rng.randrange(DAYS) for _ in range(first_half)]
    hits.append(len(set(room)) < len(room))
first100 = hits[:100]

pairs = comb(first_half, 2)
pair_approx = 1 - (Fraction(DAYS - 1, DAYS)) ** pairs   # treat pairs as independent

steps = [5, 10, 15, 20, 23]

if __name__ == "__main__":
    print("% facts")
    print(f"% naive guess, n/365 >= 1/2 first at n = {naive_half}")
    print(f"% exact p >= 1/2 first at n = {first_half}: p = {float(exact[first_half]):.4f}"
          f"   (n = {first_half - 1}: {float(exact[first_half - 1]):.4f})")
    print(f"% exact p >= 0.99 first at n = {first_99}: p = {float(exact[first_99]):.4f}"
          f"   (n = {first_99 - 1}: {float(exact[first_99 - 1]):.4f})")
    for n in (10, 23, 30, 40, 50, 57, 70, 100):
        print(f"% p({n}) = {float(exact[n]) * 100:.2f}%   naive {n / DAYS * 100:.2f}%")
    print(f"% pairs among {first_half}: {pairs};  among {first_99}: {comb(first_99, 2)}")
    print(f"% 1 - (364/365)^{pairs} = {float(pair_approx):.4f}")
    print(f"% days nobody can have used with {first_99} people: at least {DAYS - first_99}")
    print(f"% chance one given pair matches: 1/365 = {100 / DAYS:.3f}%")
    print(f"% simulation, seed 23: {sum(hits)} of {ROOMS} rooms of {first_half} had a match"
          f" = {sum(hits) / ROOMS * 100:.2f}%;  first 100 rooms: {sum(first100)}")
    print(f"% pairs for n in {steps}: {[comb(n, 2) for n in steps]}")
    print()
    print("% ---- data block for birthday.ps (percent, n = 0..100) ----")
    print("/bd-exact [")
    row = []
    for n in range(0, 101):
        row.append(f"{float(exact[n]) * 100:.3f}")
        if len(row) == 10:
            print("  " + " ".join(row)); row = []
    if row:
        print("  " + " ".join(row))
    print("] def")
    print("/bd-rooms (" + "".join("1" if h else "0" for h in first100) + ") def")
