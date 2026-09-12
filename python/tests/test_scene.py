"""What one moment looks like: Groups, names, and the shapes you can draw."""

import support  # noqa: F401  (puts `codimate` on the import path)
import codimate as cm

SLOT = cm.Slot(x=100.0, y=200.0, w=60.0, h=60.0, anchor="bottom")


def test_a_child_is_named_beneath_its_group():
    scene = cm.Scene()
    g = scene.group(7, SLOT)
    g.rect("bar", h=100.0, bottom=0)
    g.text("label", 3, top=20, size=32)

    assert {s["item"] for s in scene._payload()} == {"7/bar", "7/label"}


def test_zero_is_the_groups_own_point():
    scene = cm.Scene()
    g = scene.group(7, SLOT)
    g.rect("bar", h=100.0, bottom=0)
    g.text("label", 3, top=20, size=32)
    shapes = {s["item"]: s for s in scene._payload()}

    bar = shapes["7/bar"]
    assert bar["y"] == 230.0 - 50.0, "bottom=0 stands it on the group's point"
    assert bar["x"] == 100.0, "x defaults to the group's centre"
    assert bar["w"] == 60.0, "w defaults to the group's width"
    assert shapes["7/label"]["y"] == 230.0 + 20.0 + 16.0, "top=20 puts it 20 below"


def test_a_shape_that_says_nothing_sits_at_the_groups_point():
    g = cm.Scene().group("n", cm.Slot(x=100.0, y=200.0, w=60.0, h=60.0))
    g.circle("body", r=30)
    body = g._scene._payload()[0]
    assert (body["x"], body["y"]) == (100.0, 200.0)


def test_a_group_moves_as_one_thing():
    def built(x):
        scene = cm.Scene()
        g = scene.group(7, cm.Slot(x=x, y=200.0, w=60.0, h=60.0, anchor="bottom"))
        g.rect("bar", h=100.0, bottom=0)
        g.text("label", 3, top=20, size=32)
        return {s["item"]: s for s in scene._payload()}

    before, after = built(100.0), built(300.0)
    deltas = {after[k]["x"] - before[k]["x"] for k in before}
    assert deltas == {200.0}, f"every child shifts by the same amount, got {deltas}"


def test_nesting_composes_names_and_origins():
    scene = cm.Scene()
    scene.group("outer", SLOT).group("inner").rect("dot", h=10.0, w=10.0, y=0)
    assert "outer/inner/dot" in {s["item"] for s in scene._payload()}


def test_a_nested_key_flattens():
    scene = cm.Scene()
    scene.line(("edge", (0, 1), (1, 2)), start=(0, 0), end=(1, 1))
    assert scene._payload()[0]["item"] == "edge/0/1/1/2"


def test_a_line_carries_both_ends():
    scene = cm.Scene()
    scene.line("wire", start=(10, 20), end=(110, 220), w=3.0, color="orange")
    shape = scene._payload()[0]
    assert (shape["x"], shape["y"], shape["x2"], shape["y2"]) == (10.0, 20.0, 110.0, 220.0)
    assert shape["kind"] == "line" and shape["w"] == 3.0


def test_a_line_end_may_be_a_slot():
    a = cm.Slot(x=10.0, y=20.0, w=8.0, h=8.0)
    b = cm.Slot(x=110.0, y=220.0, w=8.0, h=8.0)
    from_slots = cm.Scene()
    from_slots.line("wire", start=a, end=b)
    from_points = cm.Scene()
    from_points.line("wire", start=(10, 20), end=(110, 220))
    assert from_slots._payload() == from_points._payload()


def test_both_ends_of_a_line_move_with_its_group():
    def built(x, y):
        g = cm.Scene().group("net", cm.Slot(x=x, y=y, w=10.0, h=10.0))
        g.line("wire", start=(10, 20), end=(110, 220))
        return g._scene._payload()[0]

    before, after = built(0.0, 0.0), built(5.0, 7.0)
    assert (after["x"] - before["x"], after["y"] - before["y"]) == (5.0, 7.0)
    assert (after["x2"] - before["x2"], after["y2"] - before["y2"]) == (5.0, 7.0)


def test_every_shape_carries_every_field():
    shape = cm.Scene().rect("bar", x=0, y=0, w=1, h=1)._payload()[0]
    assert set(shape) == {
        "item", "kind", "x", "y", "x2", "y2", "w", "h", "r",
        "color", "text", "size", "layer", "opacity",
    }, sorted(shape)


def test_a_name_means_exactly_one_thing():
    try:
        cm.Scene().rect("bar", x=0, y=0, w=1, h=1).rect("bar", x=9, y=9, w=1, h=1)
    except ValueError:
        pass
    else:
        raise AssertionError("a duplicate name should be rejected")


def test_a_formula_carries_its_latex_untouched():
    """The LaTeX has to survive the payload verbatim.

    It rides in the same `text` field a label uses, so anything that tried to
    be clever about text — stringifying, escaping, stripping — would quietly
    corrupt a formula into a typesetting error.
    """
    latex = r"\frac{QK^{T}}{\sqrt{d_k}}"
    shape = cm.Scene().formula("eq", latex, x=10, y=20, size=30)._payload()[0]
    assert shape["kind"] == "formula", shape["kind"]
    assert shape["text"] == latex, shape["text"]
    assert (shape["x"], shape["y"], shape["size"]) == (10, 20, 30), shape


def test_a_formula_carries_how_much_of_it_shows():
    """`reveal` shares the `r` field with radius — the payload is a flat union,
    so one slot reads three ways depending on the kind. Worth pinning, because
    nothing else would notice if formula started sending it somewhere else."""
    shape = cm.Scene().formula("eq", r"\frac{a}{b}", x=0, y=0, reveal=0.25)._payload()[0]
    assert shape["kind"] == "formula" and shape["r"] == 0.25, shape


def test_a_drawn_formula_carries_its_pen():
    shape = cm.Scene().formula("eq", "x", x=0, y=0, pen=2.0)._payload()[0]
    assert shape["w"] == 2.0, shape


def test_a_rounded_rect_carries_its_radius():
    shape = cm.Scene().rect("b", x=0, y=0, w=10, h=10, radius=4)._payload()[0]
    assert shape["kind"] == "rect" and shape["r"] == 4, shape


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
