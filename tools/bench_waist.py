"""How much one crossing of the Waist costs from Python, and a budget for it.

    python tools/bench_waist.py

Builds and colours many shapes the way a view does (`rect(...).fill(...)`,
two crossings) and reports microseconds per crossing. Fails above BUDGET.

When slice 1 landed a crossing cost about 10 µs on a laptop, and a real film
(C lesson 06, about 41,000 crossings) built 17–23% slower than with the old
pure-Python Surface (ADR 0032). The budget is generous enough for a slow CI
machine and tight enough to catch what actually regresses this: a second parse,
an import on every call, a deep copy where a shallow one does.
"""

from __future__ import annotations

import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "skins/python"))

import codimate as cm  # noqa: E402

BUDGET = 25.0  # microseconds per crossing
SHAPES = 5000


def build() -> None:
    scene = cm.Scene()
    for i in range(SHAPES):
        scene.rect(("bar", i), h=40, w=20, at=cm.at(x=i % 1280, bottom=600)).fill(
            "#2997ff", edge="#ffffff")


def main() -> int:
    build()  # load the extension and warm up
    best = min(_timed() for _ in range(5))
    per_crossing = best / (2 * SHAPES) * 1e6
    verdict = "ok" if per_crossing <= BUDGET else "OVER BUDGET"
    print(f"{per_crossing:.1f} µs per crossing (budget {BUDGET:.0f} µs): {verdict}")
    return 0 if per_crossing <= BUDGET else 1


def _timed() -> float:
    start = time.perf_counter()
    build()
    return time.perf_counter() - start


if __name__ == "__main__":
    raise SystemExit(main())
