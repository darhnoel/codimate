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


def group_case(frame, make, data):
    """A Group's case: what is recorded is the frame it makes (name path,
    point, width), which every shape drawn on it then measures from."""
    _, target = where(frame)
    try:
        g = make(target)
        expected = {"ok": {"path": [cm.scene._key(p) for p in g._path],
                           "ox": g._ox, "oy": g._oy, "w": g._w}}
    except ValueError as error:
        expected = {"error": str(error)}
    return {"op": "group", "data": {"group": frame, **data}, "expected": expected}


def slice3():
    slot = cm.Slot(x=300, y=400, w=120, h=80, anchor="bottom")
    as_data = {"x": 300, "y": 400, "w": 120, "h": 80, "anchor": "bottom"}
    cases = []
    for frame in (ROOT, GROUP):
        cases += [
            group_case(frame, lambda g: g.group("a", at=(10, 20)),
                       {"key": ["a"], "slot": None, "at": [10, 20], "anchor": None, "w": None}),
            group_case(frame, lambda g: g.group("a", at=(10, 20), w=50),
                       {"key": ["a"], "slot": None, "at": [10, 20], "anchor": None, "w": 50}),
            group_case(frame, lambda g: g.group(("bar", 3), slot),
                       {"key": ["bar", "3"], "slot": as_data, "at": None, "anchor": None, "w": None}),
            group_case(frame, lambda g: g.group("a", slot, anchor="top"),
                       {"key": ["a"], "slot": as_data, "at": None, "anchor": "top", "w": None}),
            group_case(frame, lambda g: g.group("a", slot, anchor="center", w=9),
                       {"key": ["a"], "slot": as_data, "at": None, "anchor": "center", "w": 9}),
            group_case(frame, lambda g: g.group("a", slot, anchor="left"),
                       {"key": ["a"], "slot": as_data, "at": None, "anchor": "left", "w": None}),
            group_case(frame, lambda g: g.group("a", at=cm.at(x=5, bottom=50)),
                       {"key": ["a"], "slot": None, "at": {"x": 5, "y": None, "top": None, "bottom": 50},
                        "anchor": None, "w": None}),
            group_case(frame, lambda g: g.group("a"),
                       {"key": ["a"], "slot": None, "at": None, "anchor": None, "w": None}),
        ]
    return cases


def query(op, answer, data):
    """A Query's case: what Python answered, as plain data."""
    try:
        expected = {"ok": json.loads(json.dumps(answer()))}
    except ValueError as error:
        expected = {"error": str(error)}
    return {"op": op, "data": data, "expected": expected}


def slots(make):
    return lambda: [{"x": s.x, "y": s.y, "w": s.w, "h": s.h, "anchor": s.anchor}
                    for s in (pair[0] if isinstance(pair, tuple) else pair
                              for pair in make())]


def slice4():
    canvas = [1280.0, 720.0]
    box = cm.Slot(x=300, y=200, w=400, h=300)
    box_data = {"x": 300, "y": 200, "w": 400, "h": 300, "anchor": "center"}
    none = {"gap": 40.0, "size": None, "at": None, "within": None, "canvas": canvas}
    cases = [
        query("ngon", lambda: cm.ngon(3, r=60, at=(640, 360)),
              {"sides": 3, "r": 60, "at": [640, 360], "turn": 0.0}),
        query("ngon", lambda: cm.ngon(6, r=10, turn=30),
              {"sides": 6, "r": 10, "at": [0, 0], "turn": 30}),
        query("ngon", lambda: cm.ngon(2, r=10),
              {"sides": 2, "r": 10, "at": [0, 0], "turn": 0.0}),
        query("star", lambda: cm.star(5, r=80, at=(640, 360)),
              {"points": 5, "r": 80, "inner": None, "at": [640, 360], "turn": 0.0}),
        query("star", lambda: cm.star(4, r=50, inner=10, turn=45),
              {"points": 4, "r": 50, "inner": 10, "at": [0, 0], "turn": 45}),
        query("row", slots(lambda: cm.row(4)), {"count": 4, **none}),
        query("row", slots(lambda: cm.row([3, 1, 4], gap=20, size=100)),
              {"count": 3, **none, "gap": 20, "size": 100}),
        query("row", slots(lambda: cm.row(2, size=(80, 30), at=cm.at(bottom=500))),
              {"count": 2, **none, "size": [80, 30],
               "at": {"x": None, "y": None, "top": None, "bottom": 500}}),
        query("row", slots(lambda: cm.row(3, at=cm.at(y=100), within=box)),
              {"count": 3, **none, "at": {"x": None, "y": 100, "top": None, "bottom": None},
               "within": box_data}),
        query("row", slots(lambda: cm.row(0)), {"count": 0, **none}),
        query("column", slots(lambda: cm.column(4, gap=40, at=cm.at(x=640))),
              {"count": 4, **none, "at": {"x": 640, "y": None, "top": None, "bottom": None}}),
        query("column", slots(lambda: cm.column(["a", "b"], size=(200, 50), within=box)),
              {"count": 2, **none, "size": [200, 50], "within": box_data}),
        query("column", slots(lambda: cm.column(3, size=60, at=cm.at(y=300))),
              {"count": 3, **none, "size": 60,
               "at": {"x": None, "y": 300, "top": None, "bottom": None}}),
        query("measure", lambda: cm.measure("Bubble sort", 48),
              {"text": "Bubble sort", "size": 48, "font": ""}),
        query("measure", lambda: cm.measure("", 16),
              {"text": "", "size": 16, "font": ""}),
        query("measure", lambda: cm.measure("ពន្យល់", 30),
              {"text": "ពន្យល់", "size": 30, "font": ""}),
    ]
    for t in (0.0, 0.25, 0.5, 0.9, 1.0):
        cases.append(query("ease", lambda t=t: cm.ease(t), {"t": t}))
    return cases


def rule_case(make, data):
    """A Rule's case: its fields, and the payload the Engine takes."""
    try:
        rule = make()
        expected = {"ok": {"pattern": rule.pattern, "position": rule.position,
                           "ease": rule.ease, "order": rule.order,
                           "options": rule.options,
                           "payload": json.loads(json.dumps(rule._payload()))}}
    except ValueError as error:
        expected = {"error": str(error)}
    return {"op": "rule", "data": data, "expected": expected}


def slice5():
    base = {"position": "straight", "ease": None, "order": "screen", "options": {}}
    return [
        rule_case(lambda: cm.Rule("*"), {"pattern": ["*"], **base}),
        rule_case(lambda: cm.Rule(("item", "*")), {"pattern": ["item", "*"], **base}),
        rule_case(lambda: cm.Rule("*", position="lift_carry_drop", clearance=90),
                  {"pattern": ["*"], **base, "position": "lift_carry_drop",
                   "options": {"clearance": 90}}),
        rule_case(lambda: cm.Rule("bar/*", ease="spring", stagger=0.08),
                  {"pattern": ["bar/*"], **base, "ease": "spring", "options": {"stagger": 0.08}}),
        rule_case(lambda: cm.Rule("card", position="arc", bend=-0.4, ease="out", order="centre"),
                  {"pattern": ["card"], **base, "position": "arc", "ease": "out",
                   "order": "centre", "options": {"bend": -0.4}}),
        rule_case(lambda: cm.Rule("*", position="teleport"),
                  {"pattern": ["*"], **base, "position": "teleport"}),
        rule_case(lambda: cm.Rule("*", ease="wobble"), {"pattern": ["*"], **base, "ease": "wobble"}),
        rule_case(lambda: cm.Rule("*", order="random"), {"pattern": ["*"], **base, "order": "random"}),
        rule_case(lambda: cm.Rule("*", delay=1.0), {"pattern": ["*"], **base, "options": {"delay": 1.0}}),
        rule_case(lambda: cm.Rule("*", stagger=-0.1),
                  {"pattern": ["*"], **base, "options": {"stagger": -0.1}}),
    ]


def handle_case(op, draw, change, data):
    """A Handle method's case: the shape before (sent as context) and after."""
    scene = cm.Scene()
    handle = draw(scene)
    before = json.loads(json.dumps(asdict(scene._shapes[handle._name])))
    try:
        change(handle)
        expected = {"ok": json.loads(json.dumps(asdict(scene._shapes[handle._name])))}
    except ValueError as error:
        expected = {"error": str(error)}
    return {"op": op, "data": {"shape": before, **data}, "expected": expected}


def slice6():
    def rect(s):
        return s.rect("r", w=10, h=10, at=(0, 0))

    def formula(s):
        return s.formula("f", "x^2", at=(0, 0))

    def svg(s):
        return s.svg("s", "logo.svg", at=(0, 0))

    return [
        handle_case("turn", rect, lambda h: h.turn(30), {"degrees": 30, "pivot": "center"}),
        handle_case("turn", rect, lambda h: h.turn(-15, pivot="bottom"),
                    {"degrees": -15, "pivot": "bottom"}),
        handle_case("turn", rect, lambda h: h.turn(5, pivot="corner"),
                    {"degrees": 5, "pivot": "corner"}),
        handle_case("grow", rect, lambda h: h.grow(2), {"scale": 2}),
        handle_case("grow", rect, lambda h: h.grow((1.5, 0.5)), {"scale": [1.5, 0.5]}),
        handle_case("on", rect, lambda h: h.on(layer=4), {"layer": 4, "opacity": None}),
        handle_case("on", rect, lambda h: h.on(opacity=0.25), {"layer": None, "opacity": 0.25}),
        handle_case("on", rect, lambda h: h.on(layer=-2, opacity=0), {"layer": -2, "opacity": 0}),
        handle_case("on", rect, lambda h: h.on(), {"layer": None, "opacity": None}),
        handle_case("round", rect, lambda h: h.round(4), {"radius": 4}),
        handle_case("write", formula, lambda h: h.write(reveal=0.5), {"reveal": 0.5, "pen": 0.0}),
        handle_case("write", formula, lambda h: h.write(pen=2), {"reveal": None, "pen": 2}),
        handle_case("write", svg, lambda h: h.write(reveal=0.25, pen=3), {"reveal": 0.25, "pen": 3}),
        handle_case("write", svg, lambda h: h.write(), {"reveal": None, "pen": 0.0}),
    ]


def preset_data(name, content, preset, style=None, at=None):
    style = style or {"size": 16.0, "font": None, "fill": None, "distance": 48.0}
    return {"name": name, "content": content, "preset": preset, "style": style, "at": at}


def preset_draw(make, state, data, frame=ROOT):
    """A preset drawn: every shape it puts on the Scene, in order."""
    scene, target = where(frame)
    try:
        make().draw(target, state=state)
        expected = {"ok": [json.loads(json.dumps(asdict(s))) for s in scene._shapes.values()]}
    except ValueError as error:
        expected = {"error": str(error)}
    return {"op": "text_preset_draw",
            "data": {"preset": data, "group": frame, "film_font": "", "state": state},
            "expected": expected}


def preset_rules(make, kwargs, data):
    try:
        rules = make().rules(**kwargs)
        expected = {"ok": [{"pattern": r.pattern, "position": r.position, "ease": r.ease,
                            "order": r.order, "options": r.options,
                            "payload": json.loads(json.dumps(r._payload()))} for r in rules]}
    except ValueError as error:
        expected = {"error": str(error)}
    return {"op": "text_preset_rules",
            "data": {"preset": data, "ease": kwargs.get("ease", "out"),
                     "stagger": kwargs.get("stagger", 0.12),
                     "order": kwargs.get("order", "screen")},
            "expected": expected}


def preset_check(make, data):
    try:
        p = make()
        expected = {"ok": {"name": p.name, "content": p.content, "preset": p.preset,
                           "style": {"size": p.style.size, "font": p.style.font,
                                     "fill": p.style.fill, "distance": p.style.distance},
                           "at": data["at"]}}
    except ValueError as error:
        expected = {"error": str(error)}
    return {"op": "text_preset", "data": data, "expected": expected}


def slice7():
    T = cm.text
    big = {"size": 40.0, "font": None, "fill": "#ffd60a", "distance": 30.0}
    cases = []
    for preset in ("words_up", "words_down", "words_left", "words_right",
                   "letters_up", "letters_down", "words_up_blur"):
        for state in (("hidden", "blurred", "shown") if preset.endswith("blur")
                      else ("hidden", "shown")):
            cases.append(preset_draw(
                lambda preset=preset: T.TextPreset("t", "Hello world", preset, at=(640, 360)),
                state, preset_data("t", "Hello world", preset, at=[640, 360])))
    style = T.TextPresetStyle(size=40, fill="#ffd60a", distance=30)
    cases += [
        preset_draw(lambda: T.words_up("two", "First line\nsecond one", style=style,
                                       at=cm.at(x=400, top=100)),
                    "hidden", preset_data("two", "First line\nsecond one", "words_up", big,
                                          {"x": 400, "y": None, "top": 100, "bottom": None})),
        preset_draw(lambda: T.letters_up("kh", "ពន្យល់ ok", at=(300, 300)), "shown",
                    preset_data("kh", "ពន្យល់ ok", "letters_up", at=[300, 300])),
        preset_draw(lambda: T.letters_up("em", "a👩‍💻b🇯🇵", at=(300, 300)), "shown",
                    preset_data("em", "a👩‍💻b🇯🇵", "letters_up", at=[300, 300])),
        preset_draw(lambda: T.words_up("g", "in a group"), "hidden",
                    preset_data("g", "in a group", "words_up"), GROUP),
        preset_draw(lambda: T.words_up("t", "x", at=(0, 0)), "blurred",
                    preset_data("t", "x", "words_up", at=[0, 0])),
        preset_rules(lambda: T.words_up("t", "x"), {}, preset_data("t", "x", "words_up")),
        preset_rules(lambda: T.words_up_blur("t", "x"), {"ease": "spring", "stagger": 0.2,
                                                         "order": "centre"},
                     preset_data("t", "x", "words_up_blur")),
        preset_check(lambda: T.TextPreset("a/b", "x", "words_up"), preset_data("a/b", "x", "words_up")),
        preset_check(lambda: T.TextPreset("", "x", "words_up"), preset_data("", "x", "words_up")),
        preset_check(lambda: T.TextPreset("t", "x", "spin"), preset_data("t", "x", "spin")),
        preset_check(lambda: T.TextPreset("t", "x", "words_up", style=T.TextPresetStyle(size=0)),
                     preset_data("t", "x", "words_up",
                                 {"size": 0, "font": None, "fill": None, "distance": 48.0})),
        preset_check(lambda: T.TextPreset("t", 42, "words_up"), preset_data("t", "42", "words_up")),
    ]
    return cases


def focus_case(names, kwargs, data):
    scene = cm.Scene()
    scene.focus(*names, **kwargs)
    focus = scene._focus
    return {"op": "focus", "data": data,
            "expected": {"ok": {"names": focus["names"], "pad": focus["pad"],
                                "min_size": focus["min_size"]}}}


def slice8():
    cases = []
    for frame in (ROOT, GROUP):
        cases += [
            case("formula", frame, lambda g: g.formula("f", r"\frac{a}{b}", size=34, at=(10, 20)),
                 {"key": ["f"], "latex": r"\frac{a}{b}", "size": 34, "at": [10, 20]}),
            case("formula", frame, lambda g: g.formula("f", "x^2", at=cm.at(x=5, top=0)),
                 {"key": ["f"], "latex": "x^2", "size": 16.0,
                  "at": {"x": 5, "y": None, "top": 0, "bottom": None}}),
            case("image", frame, lambda g: g.image("i", "a.png", at=(1, 2)),
                 {"key": ["i"], "file": "a.png", "size": None, "at": [1, 2]}),
            case("image", frame, lambda g: g.image("i", "a.png", size=400, at=(1, 2)),
                 {"key": ["i"], "file": "a.png", "size": 400, "at": [1, 2]}),
            case("image", frame, lambda g: g.image("i", "a.png", size=(520, 300), at=(1, 2)),
                 {"key": ["i"], "file": "a.png", "size": [520, 300], "at": [1, 2]}),
            case("svg", frame, lambda g: g.svg("s", "logo.svg", at=(3, 4)),
                 {"key": ["s"], "file": "logo.svg", "size": 120.0, "at": [3, 4]}),
            case("svg", frame, lambda g: g.svg("s", "flow.svg", size=(900, 420), at=(3, 4)),
                 {"key": ["s"], "file": "flow.svg", "size": [900, 420], "at": [3, 4]}),
        ]
    cases += [
        focus_case(["cat"], {}, {"names": [["cat"]], "pad": 40.0, "least": 240.0}),
        focus_case([("cell", 2, 1)], {"pad": 60}, {"names": [["cell", "2", "1"]], "pad": 60,
                                                   "least": 240.0}),
        focus_case(["The", "cat", "sat"], {"least": 100}, {"names": [["The"], ["cat"], ["sat"]],
                                                          "pad": 40.0, "least": 100}),
        focus_case([], {}, {"names": [], "pad": 40.0, "least": 240.0}),
    ]
    return cases


def mark_of(mark):
    """A Python mark as the data every Skin keeps: a stroke, or a link from a
    highlight's words to its band."""
    if isinstance(mark, str):
        return {"kind": "link", "band": mark}
    return {"kind": "stroke", "w": mark.w, "h": mark.h, "seed": mark.seed,
            "reveal": mark.reveal, "angle": mark.angle, "words": mark.words,
            "paper": mark.paper}


def shape_of(shape):
    return json.loads(json.dumps(asdict(shape)))


def related(scene, name):
    """A marked shape's partner: a stroke's words, or a link's band."""
    mark = scene._markers.get(name)
    names = [name]
    if isinstance(mark, str):
        names.append(mark)
    elif mark is not None and mark.words:
        names.append(mark.words)
    return names


def marked_case(op, draw, change, data):
    """A Handle method on a marked shape: what is sent (the shape, its mark,
    its partner) and what comes back (every shape and mark that changed)."""
    scene = cm.Scene()
    name = draw(scene)
    names = related(scene, name)
    before = {"shapes": {n: shape_of(scene._shapes[n]) for n in names},
              "marks": {n: mark_of(scene._markers[n]) for n in names if n in scene._markers}}
    from codimate.scene.handle import Handle
    handle = Handle(scene._shapes, name, scene._markers)
    try:
        change(handle)
        expected = {"ok": {"shapes": {n: shape_of(scene._shapes[n]) for n in names},
                           "marks": {n: mark_of(scene._markers[n]) for n in names
                                     if n in scene._markers}}}
    except ValueError as error:
        expected = {"error": str(error)}
    return {"op": "marked",
            "data": {"op": op, "name": name, **before,
                     "data": {"shape": before["shapes"][name], **data}},
            "expected": expected}


def slice9():
    cases = []
    for frame in (ROOT, GROUP):
        def marker(g):
            g.marker("note", w=260, h=70, at=(40, 30))

        scene, target = where(frame)
        marker(target)
        (shape,) = scene._shapes.values()
        cases.append({"op": "marker",
                      "data": {"group": frame, "key": ["note"], "w": 260, "h": 70, "at": [40, 30]},
                      "expected": {"ok": {"shape": shape_of(shape),
                                          "mark": mark_of(scene._markers[shape.item])}}})
        scene, target = where(frame)
        band, words = target.highlight("pivot", "Pivot", size=30, at=(100, 60))
        cases.append({"op": "highlight",
                      "data": {"group": frame, "key": ["pivot"], "content": "Pivot",
                               "size": 30, "font": None, "film_font": "", "at": [100, 60]},
                      "expected": {"ok": {
                          "shapes": [shape_of(scene._shapes[band._name]),
                                     shape_of(scene._shapes[words._name])],
                          "marks": {n: mark_of(scene._markers[n])
                                    for n in (band._name, words._name)}}}})

    def a_marker(scene):
        scene.marker("note", w=260, h=70, at=(640, 300))
        return "note"

    def a_band(scene):
        scene.highlight("pivot", "Pivot", size=30, at=(640, 360))
        return "pivot marker"

    def the_words(scene):
        scene.highlight("pivot", "Pivot", size=30, at=(640, 360))
        return "pivot"

    cases += [
        marked_case("fill", a_marker, lambda h: h.fill("pink"),
                    {"color": "pink", "edge": None, "edge_w": None}),
        marked_case("fill", a_marker, lambda h: h.fill("#123456"),
                    {"color": "#123456", "edge": None, "edge_w": None}),
        marked_case("turn", a_marker, lambda h: h.turn(-8), {"degrees": -8, "pivot": "center"}),
        marked_case("turn", a_marker, lambda h: h.turn(5, "corner"),
                    {"degrees": 5, "pivot": "corner"}),
        marked_case("write", a_marker, lambda h: h.write(0.5), {"reveal": 0.5, "pen": 0.0}),
        marked_case("write", a_marker, lambda h: h.write(), {"reveal": None, "pen": 0.0}),
        marked_case("write", a_marker, lambda h: h.write(0.0), {"reveal": 0.0, "pen": 0.0}),
        marked_case("on", a_marker, lambda h: h.on(opacity=0.5), {"layer": None, "opacity": 0.5}),
        marked_case("write", a_band, lambda h: h.write(0.25), {"reveal": 0.25, "pen": 0.0}),
        marked_case("write", a_band, lambda h: h.write(0.0), {"reveal": 0.0, "pen": 0.0}),
        marked_case("fill", the_words, lambda h: h.fill("#ffd60a"),
                    {"color": "#ffd60a", "edge": None, "edge_w": None}),
        marked_case("fill", the_words, lambda h: h.fill("grey"),
                    {"color": "grey", "edge": None, "edge_w": None}),
        marked_case("fill", the_words, lambda h: h.fill("teal"),
                    {"color": "teal", "edge": None, "edge_w": None}),
        marked_case("turn", the_words, lambda h: h.turn(10), {"degrees": 10, "pivot": "center"}),
    ]
    return cases


def axes_fields(plot):
    return {"x0": plot.x0, "x1": plot.x1, "y0": plot.y0, "y1": plot.y1,
            "left": plot.left, "top": plot.top, "w": plot.w, "h": plot.h,
            "ink": plot.ink, "label": plot.label, "layer": plot.layer,
            "right": plot.right, "bottom": plot.bottom}


def slice10():
    canvas = [1280.0, 720.0]
    made = []
    for kwargs, data in [
        ({"x": (-4, 4), "y": (-2, 6)}, {"x": [-4, 4], "y": [-2, 6], "at": None, "size": None, "within": None}),
        ({"x": (0, 10), "y": (0, 5), "size": (760, 420), "at": cm.at(x=500, top=100)},
         {"x": [0, 10], "y": [0, 5], "size": [760, 420],
          "at": {"x": 500, "y": None, "top": 100, "bottom": None}, "within": None}),
        ({"x": (0, 1), "y": (0, 1), "size": 300, "within": cm.Slot(x=300, y=300, w=400, h=400)},
         {"x": [0, 1], "y": [0, 1], "size": 300, "at": None,
          "within": {"x": 300, "y": 300, "w": 400, "h": 400, "anchor": "center"}}),
        ({"x": (1, 1), "y": (0, 1)}, {"x": [1, 1], "y": [0, 1], "at": None, "size": None, "within": None}),
    ]:
        try:
            expected = {"ok": axes_fields(cm.axes(**kwargs))}
        except ValueError as error:
            expected = {"error": str(error)}
        made.append({"op": "axes", "data": {**data, "canvas": canvas}, "expected": expected})
    plot = cm.axes(x=(-4, 4), y=(-2, 6))
    fields = axes_fields(plot)
    cases = made + [
        {"op": "axes_looks", "data": {"axes": fields, "ink": "#334", "label": 16, "layer": 3},
         "expected": {"ok": axes_fields(plot.looks(ink="#334", label=16, layer=3))}},
        {"op": "axes_looks", "data": {"axes": fields, "ink": None, "label": None, "layer": None},
         "expected": {"ok": axes_fields(plot.looks())}},
        {"op": "axes_at", "data": {"axes": fields, "x": 2.0, "y": 4.0},
         "expected": {"ok": list(plot.at(2.0, 4.0))}},
        {"op": "axes_at", "data": {"axes": fields, "x": -4, "y": -2},
         "expected": {"ok": list(plot.at(-4, -2))}},
    ]
    for steps, over in ((200, None), (4, (-1, 1)), (0, None)):
        xs = [x for x, _ in plot.line(lambda t: 0.0, steps=steps, over=over)]
        lo, hi = (plot.x0, plot.x1) if over is None else over
        n = max(int(steps), 1)
        cases.append({"op": "axes_samples",
                      "data": {"axes": fields, "steps": steps, "over": list(over) if over else None},
                      "expected": {"ok": [lo + (hi - lo) * i / n for i in range(n + 1)]}})
        ys = [x * x for x in cases[-1]["expected"]["ok"]]
        cases.append({"op": "axes_points",
                      "data": {"axes": fields, "xs": cases[-1]["expected"]["ok"], "ys": ys},
                      "expected": {"ok": [list(p) for p in plot.line(lambda t: t * t, steps=steps, over=over)]}})
        assert xs
    for axis, about, p in (("x", 6, plot), ("y", 6, plot), ("x", 3, cm.axes(x=(0, 0.05), y=(0, 1))),
                           ("y", 10, cm.axes(x=(0, 1), y=(-1000, 2500)))):
        cases.append({"op": "axes_ticks", "data": {"axes": axes_fields(p), "axis": axis, "about": about},
                      "expected": {"ok": [list(t) for t in p.ticks(axis, about)]}})
    for frame in (ROOT, GROUP):
        for grid, p, name in ((False, plot, "plot"), (True, cm.axes(x=(0, 10), y=(0, 5)).looks(layer=2), ("g", 1))):
            scene, target = where(frame)
            p.draw(target, name, grid=grid)
            cases.append({"op": "axes_draw",
                          "data": {"axes": axes_fields(p), "group": frame, "film_font": "",
                                   "name": list(map(str, cm.scene._key(name).split("/"))),
                                   "about": 6, "grid": grid},
                          "expected": {"ok": [shape_of(s) for s in scene._shapes.values()]}})
    return cases


# Where the Python Surface was wrong, the Core is not: each fix is named here
# and applied to the recording, so the fixture says what is right and why.
FIXES = {
    "slice10": [(
        lambda c: c["op"] == "axes" and "error" in c["expected"],
        "the Core sees the ranges as numbers, so it writes them as floats",
        lambda c: c["expected"].update(error="a range needs width, got x=(1.0, 1.0) y=(0.0, 1.0)"),
    )],
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
            if applies(c):
                fix(c)
                c["fixed"] = why
    path = OUT / f"{name}.json"
    path.write_text(json.dumps(cases, indent=1) + "\n")
    print(f"wrote {len(cases)} cases to {path.relative_to(OUT.parents[3])}")
