"""Self-checks for the pure-Python half. No Rust needed — run with:

    python python/tests/test_codimate.py
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import codimate as cm


@cm.trace()
def swap_once(values):
    values[0], values[1] = values[1], values[0]
    cm.emit("swap", items=[values[0], values[1]])


def view(frame):
    scene = cm.Scene()
    for slot, item in cm.row(frame.state, gap=40):
        scene.group(item.id, slot).rect("bar", h=item.value * 40, bottom=0)
    return scene


def trace_and_timing():
    values = cm.items([3, 1])
    t = swap_once(values)

    assert [i.value for i in t.initial] == [3, 1]
    assert len(t.events) == 1
    assert [i.value for i in t.events[0].state] == [1, 3], "emit() snapshots AFTER the mutation"

    e = cm.explain(trace=t, view=view, timing=cm.Timing(default=0.5))

    # The Engine requires exactly one more Scene than duration, including the
    # held opening and the final hold.
    assert len(e.scenes) == len(e.durations) + 1
    assert e.durations == [0.8, 0.5, 1.2], e.durations
    assert abs(e.duration - 2.5) < 1e-6

    # Every shape carries every field — the Engine's diff is uniform.
    shape = e.scenes[0]._payload()[0]
    assert set(shape) == {
        "item", "kind", "x", "y", "w", "h", "r", "color", "text", "size", "layer", "opacity",
    }, sorted(shape)

    # Identity, not order, is what the Engine pairs on.
    first = {s["item"] for s in e.scenes[1]._payload()}
    last = {s["item"] for s in e.scenes[-1]._payload()}
    assert first == last and len(first) == 2


def item_identity():
    a, b = cm.Item(3), cm.Item(3)

    # Two equal values are two different things.
    assert a != b, "items with the same value are still different items"
    assert a.id != b.id
    assert len({a, b}) == 2

    # Ordering follows the value, so an algorithm sorts normally.
    assert cm.Item(1) < cm.Item(2)
    assert cm.Item(5) > cm.Item(2)
    assert sorted(cm.items([3, 1, 2]), key=lambda i: i.value)[0].value == 1

    # THE important one: `frame.state` is deep-copied at every event but
    # `event.data` is not, so a view comparing the two must still match.
    # Equality by id survives the copy; equality by `is` would not.
    @cm.trace()
    def touch(values):
        cm.emit("compare", items=[values[0]])

    t = touch(cm.items([7, 8]))
    ev = t.events[0]
    assert ev.state[0] is not ev.data["items"][0], "the snapshot really is a copy"
    assert ev.state[0] in ev.data["items"], "an Item must survive the snapshot as itself"


def groups():
    slot = cm.Slot(x=100.0, y=200.0, w=60.0, h=60.0, anchor="bottom")
    assert slot.point() == (100.0, 230.0), "a row slot anchors bottom-centre"

    scene = cm.Scene()
    g = scene.group(7, slot)
    g.rect("bar", h=100.0, bottom=0)
    g.text("label", 3, top=20, size=32)

    shapes = {s["item"]: s for s in scene._payload()}

    # A child's name is the group's name plus its own.
    assert set(shapes) == {"7/bar", "7/label"}, sorted(shapes)

    # Inside a group, 0 is the group's anchor point.
    bar = shapes["7/bar"]
    assert bar["y"] == 230.0 - 50.0, bar["y"]      # bottom=0 stands it on the point
    assert bar["x"] == 100.0                       # x defaults to the group's centre
    assert bar["w"] == 60.0                        # w defaults to the group's width

    label = shapes["7/label"]
    assert label["y"] == 230.0 + 20.0 + 16.0, label["y"]   # top=20 puts it 20 below

    # Everything in a group shifts together when the group moves.
    moved = cm.Scene()
    m = moved.group(7, cm.Slot(x=300.0, y=200.0, w=60.0, h=60.0, anchor="bottom"))
    m.rect("bar", h=100.0, bottom=0)
    m.text("label", 3, top=20, size=32)
    after = {s["item"]: s for s in moved._payload()}
    deltas = {after[k]["x"] - shapes[k]["x"] for k in shapes}
    assert deltas == {200.0}, f"a group moves as one thing, got {deltas}"

    # Nesting composes names and origins.
    inner = scene.group("outer", slot).group("inner")
    inner.rect("dot", h=10.0, w=10.0, y=0)
    assert "outer/inner/dot" in {s["item"] for s in scene._payload()}


def layout():
    cm.canvas(1280, 720)

    slots = [slot for slot, _ in cm.row([1, 2, 3, 4], gap=40)]
    assert len(slots) == 4

    # A row is centred on the canvas: the margins match.
    assert abs(slots[0].left - (cm.width() - slots[-1].right)) < 1e-6

    # Slots are evenly spaced by exactly the gap, and share a baseline.
    for a, b in zip(slots, slots[1:]):
        assert abs((b.left - a.right) - 40) < 1e-6
    assert len({round(s.bottom, 6) for s in slots}) == 1

    # Anchors resolve to the centre the Engine wants.
    s = cm.Scene().rect("bar", x=100, bottom=560, w=60, h=200)
    assert s._payload()[0]["y"] == 460.0

    s = cm.Scene().rect("bar", left=100, top=50, w=60, h=200)
    assert (s._payload()[0]["x"], s._payload()[0]["y"]) == (130.0, 150.0)

    s = cm.Scene().text("label", 3, x=100, top=580, size=32)
    assert s._payload()[0]["y"] == 596.0

    # At the top level an anchor is required; two on one axis is an error.
    for kwargs in ({"x": 1, "left": 2, "y": 0}, {"x": 1}, {"y": 1}):
        try:
            cm.Scene().rect("bar", w=10, h=10, **kwargs)
        except ValueError:
            pass
        else:
            raise AssertionError(f"ambiguous anchors should be rejected: {kwargs}")

    # A row on a bigger canvas spreads wider — layout follows the canvas.
    cm.canvas(1920, 1080)
    assert [s for s, _ in cm.row([1, 2, 3, 4], gap=40)][0].w > slots[0].w
    cm.canvas(1280, 720)


def mistakes():
    # A name means exactly one thing.
    try:
        cm.Scene().rect("bar", x=0, y=0, w=1, h=1).rect("bar", x=9, y=9, w=1, h=1)
    except ValueError:
        pass
    else:
        raise AssertionError("a duplicate name should be rejected")

    # An unknown motion path fails at authoring time, not render time.
    try:
        cm.Rule("*", position="teleport")
    except ValueError:
        pass
    else:
        raise AssertionError("an unknown path should be rejected")

    # emit() outside a traced function is a mistake worth naming.
    try:
        cm.emit("stray")
    except RuntimeError:
        pass
    else:
        raise AssertionError("emit() outside @trace should be rejected")

    # A rule may be written as a path tuple or a glob string.
    assert cm.Rule(("item", "*")).pattern == "item/*"
    assert cm.Rule("*").pattern == "*"


def main():
    trace_and_timing()
    item_identity()
    groups()
    layout()
    mistakes()
    print("ok")


if __name__ == "__main__":
    main()
