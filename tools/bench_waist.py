"""How much one crossing of the Waist costs from Python, and a budget for it.

    python tools/bench_waist.py

Builds and colours many shapes the way a view does (`rect(...).fill(...)`,
two crossings), then does the same in plain Python: the same call data, through
`json` and back. The budget is the ratio of the two, so it means the same on a
fast laptop and on a slow, shared CI machine; microseconds are printed too.

When slice 1 landed a crossing cost about 10 µs on a laptop, and a real film
(C lesson 06, about 41,000 crossings) built 17–23% slower than with the old
pure-Python Surface (ADR 0032). The same laptop measures a ratio near 6. The
budget catches what actually regresses this: a second parse, an import on
every call, a deep copy where a shallow one does.
"""

from __future__ import annotations

import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "skins/python"))

import codimate as cm  # noqa: E402

BUDGET = 10.0  # a crossing may cost this many plain-Python round trips
SHAPES = 5000


def build() -> None:
    scene = cm.Scene()
    for i in range(SHAPES):
        scene.rect(("bar", i), h=40, w=20, at=cm.at(x=i % 1280, bottom=600)).fill(
            "#2997ff", edge="#ffffff")


def baseline() -> None:
    for i in range(SHAPES):
        for data in ({"op": "rect", "name": ["bar", i], "h": 40, "w": 20,
                      "at": {"x": i % 1280, "bottom": 600}},
                     {"op": "fill", "colour": "#2997ff", "edge": "#ffffff"}):
            json.loads(json.dumps(data))


def main() -> int:
    build()  # load the extension and warm up
    baseline()
    waist = min(_timed(build) for _ in range(5))
    plain = min(_timed(baseline) for _ in range(5))
    ratio = waist / plain
    verdict = "ok" if ratio <= BUDGET else "OVER BUDGET"
    print(f"{waist / (2 * SHAPES) * 1e6:.1f} µs per crossing, {ratio:.1f}× a plain "
          f"round trip (budget {BUDGET:.0f}×): {verdict}")
    return 0 if ratio <= BUDGET else 1


def _timed(work) -> float:
    start = time.perf_counter()
    work()
    return time.perf_counter() - start


if __name__ == "__main__":
    raise SystemExit(main())
