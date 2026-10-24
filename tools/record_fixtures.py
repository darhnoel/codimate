"""Record what the Python Surface does today, as fixtures the Authoring Core
must reproduce exactly (TDD for a port: the old behaviour is the test).

    python tools/record_fixtures.py slice2

Each case is a Python call, made on a fresh Scene or Group, and the Waist
data a Skin will send for it. The fixture keeps both the data and what Python
produced: the shape, or the error message.
"""

from __future__ import annotations

import json
import sys
from dataclasses import asdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))

import codimate as cm  # noqa: E402

OUT = Path(__file__).resolve().parents[1] / "crates/codimate-author/tests/fixtures"
ROOT = {"path": [], "ox": 0.0, "oy": 0.0, "w": 1280.0}
GROUP = {"path": ["g"], "ox": 100.0, "oy": 200.0, "w": 300.0}


def where(frame):
    scene = cm.Scene()
    return scene, (scene if not frame["path"] else
                   scene.group("g", at=(frame["ox"], frame["oy"]), w=frame["w"]))


def case(op, frame, draw, data):
    scene, target = where(frame)
    try:
        draw(target)
        (shape,) = scene._shapes.values()
        expected = {"ok": json.loads(json.dumps(asdict(shape)))}
    except ValueError as error:
        expected = {"error": str(error)}
    return {"op": op, "data": {"group": frame, **data}, "expected": expected}


def slice2():
    cases = []
    for frame in (ROOT, GROUP):
        cases += [
            case("line", frame, lambda g: g.line("l", start=(10, 20), end=(110, 70)),
                 {"key": ["l"], "start": [10, 20], "end": [110, 70], "w": 2.0}),
            case("line", frame, lambda g: g.line("l", start=(0, 0), end=(5, 5), w=6),
                 {"key": ["l"], "start": [0, 0], "end": [5, 5], "w": 6.0}),
            case("arrow", frame, lambda g: g.arrow("a", start=(0, 0), end=(200, 100)),
                 {"key": ["a"], "start": [0, 0], "end": [200, 100], "w": 4.0, "head": 16.0}),
            case("arrow", frame, lambda g: g.arrow("a", start=(50, 50), end=(55, 50), w=8, head=30),
                 {"key": ["a"], "start": [50, 50], "end": [55, 50], "w": 8.0, "head": 30.0}),
            case("arrow", frame, lambda g: g.arrow("a", start=(9, 9), end=(9, 9)),
                 {"key": ["a"], "start": [9, 9], "end": [9, 9], "w": 4.0, "head": 16.0}),
            case("polygon", frame, lambda g: g.polygon("p", [(0, 0), (100, 0), (50, 80)]),
                 {"key": ["p"], "points": [[0, 0], [100, 0], [50, 80]], "closed": True}),
            case("polygon", frame, lambda g: g.polygon("p", [(1, 2), (3, 4)], closed=False),
                 {"key": ["p"], "points": [[1, 2], [3, 4]], "closed": False}),
            case("polygon", frame, lambda g: g.polygon("p", []),
                 {"key": ["p"], "points": [], "closed": True}),
            case("curve", frame, lambda g: g.curve("c", [(0, 0), (50, 40), (100, 0)]),
                 {"key": ["c"], "points": [[0, 0], [50, 40], [100, 0]], "w": 2.0, "closed": False}),
            case("curve", frame, lambda g: g.curve("c", [(0, 0), (9, 9), (0, 9)], w=5, closed=True),
                 {"key": ["c"], "points": [[0, 0], [9, 9], [0, 9]], "w": 5.0, "closed": True}),
            case("curve", frame, lambda g: g.curve("c", [(0, 0)]),
                 {"key": ["c"], "points": [[0, 0]], "w": 2.0, "closed": False}),
            case("arc", frame, lambda g: g.arc("a", r=90, sweep=(0, 50), at=(10, 10)),
                 {"key": ["a"], "r": 90, "sweep": [0, 50], "at": [10, 10]}),
            case("arc", frame, lambda g: g.arc("a", r=(120, 60), sweep=(-30, 200), at=cm.at(x=5, top=0)),
                 {"key": ["a"], "r": [120, 60], "sweep": [-30, 200],
                  "at": {"x": 5, "y": None, "top": 0, "bottom": None}}),
        ]
    return cases


# Where the Python Surface was wrong, the Core is not: each fix is named here
# and applied to the recording, so the fixture says what is right and why.
FIXES = {
    "slice2": [(
        lambda c: c["op"] == "arc" and c["data"]["group"]["path"],
        "an arc's sweep is angles, not coordinates: a Group no longer moves it",
        lambda c: c["expected"]["ok"].update(x2=c["data"]["sweep"][0],
                                             y2=c["data"]["sweep"][1]),
    )],
}


if __name__ == "__main__":
    name = sys.argv[1]
    cases = globals()[name]()
    for applies, why, fix in FIXES.get(name, []):
        for c in cases:
            if applies(c) and "ok" in c["expected"]:
                fix(c)
                c["fixed"] = why
    path = OUT / f"{name}.json"
    path.write_text(json.dumps(cases, indent=1) + "\n")
    print(f"wrote {len(cases)} cases to {path.relative_to(OUT.parents[3])}")
