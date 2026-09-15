"""What one moment looks like: Groups, names, and the shapes you can draw."""

import support  # noqa: F401  (puts `codimate` on the import path)
import codimate as cm

SLOT = cm.Slot(x=100.0, y=200.0, w=60.0, h=60.0, anchor="bottom")


def test_a_child_is_named_beneath_its_group():
    scene = cm.Scene()
    g = scene.group(7, SLOT)
    g.rect("bar", h=100.0, at=cm.at(bottom=0))
    g.text("label", 3, size=32, at=cm.at(top=20))

    assert {s["item"] for s in scene._payload()} == {"7/bar", "7/label"}


def test_zero_is_the_groups_own_point():
    scene = cm.Scene()
    g = scene.group(7, SLOT)
    g.rect("bar", h=100.0, at=cm.at(bottom=0))
    g.text("label", 3, size=32, at=cm.at(top=20))
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
        g.rect("bar", h=100.0, at=cm.at(bottom=0))
        g.text("label", 3, size=32, at=cm.at(top=20))
        return {s["item"]: s for s in scene._payload()}

    before, after = built(100.0), built(300.0)
    deltas = {after[k]["x"] - before[k]["x"] for k in before}
    assert deltas == {200.0}, f"every child shifts by the same amount, got {deltas}"


def test_nesting_composes_names_and_origins():
    scene = cm.Scene()
    scene.group("outer", SLOT).group("inner").rect("dot", h=10.0, w=10.0, at=cm.at(y=0))
    assert "outer/inner/dot" in {s["item"] for s in scene._payload()}


def test_a_nested_key_flattens():
    scene = cm.Scene()
    scene.line(("edge", (0, 1), (1, 2)), start=(0, 0), end=(1, 1))
    assert scene._payload()[0]["item"] == "edge/0/1/1/2"


def test_a_line_carries_both_ends():
    scene = cm.Scene()
    scene.line("wire", start=(10, 20), end=(110, 220), w=3.0).fill("orange")
    shape = scene._payload()[0]
    ends = (shape["x"], shape["y"], shape["x2"], shape["y2"])
    assert ends == (10.0, 20.0, 110.0, 220.0)
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
    scene = cm.Scene()
    scene.rect("bar", w=1, h=1, at=(0, 0))
    shape = scene._payload()[0]
    assert set(shape) == {
        "item", "kind", "x", "y", "x2", "y2", "w", "h", "r",
        "color", "points", "edge", "edge_w", "text", "size", "layer", "opacity",
        "scale_x", "scale_y", "rotate", "pivot",
    }, sorted(shape)


def test_a_name_means_exactly_one_thing():
    try:
        scene = cm.Scene()
        scene.rect("bar", w=1, h=1, at=(0, 0))
        scene.rect("bar", w=1, h=1, at=(9, 9))
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
    scene = cm.Scene()
    scene.formula("eq", latex, size=30, at=(10, 20))
    shape = scene._payload()[0]
    assert shape["kind"] == "formula", shape["kind"]
    assert shape["text"] == latex, shape["text"]
    assert (shape["x"], shape["y"], shape["size"]) == (10, 20, 30), shape


def test_a_formula_carries_how_much_of_it_shows():
    """`reveal` shares the `r` field with radius — the payload is a flat union,
    so one slot reads three ways depending on the kind. Worth pinning, because
    nothing else would notice if formula started sending it somewhere else."""
    scene = cm.Scene()
    scene.formula("eq", r"\frac{a}{b}", at=(0, 0)).write(reveal=0.25)
    shape = scene._payload()[0]
    assert shape["kind"] == "formula" and shape["r"] == 0.25, shape


def test_a_drawn_formula_carries_its_pen():
    scene = cm.Scene()
    scene.formula("eq", "x", at=(0, 0)).write(pen=2.0)
    shape = scene._payload()[0]
    assert shape["w"] == 2.0, shape


def test_every_drawable_shape_says_more_the_same_way():
    """`scale`, `rotate` and `pivot` are the rest of the engine's Transform.

    They are said on the handle rather than passed in, which is what keeps the
    shape functions inside the five-argument limit `ruff --select PLR0913`
    enforces — they used to carry eighteen.
    """
    for draw in (lambda s: s.rect("k", h=4, w=4, at=(0, 0)),
                 lambda s: s.circle("k", r=4, at=(0, 0)),
                 lambda s: s.text("k", "hi", at=(0, 0)),
                 lambda s: s.polygon("k", [(0, 0), (4, 0), (2, 3)])):
        scene = cm.Scene()
        draw(scene).grow(2.0).turn(30, "top")
        shape = scene._payload()[0]
        spin = (shape["scale_x"], shape["rotate"], shape["pivot"])
        assert spin == (2.0, 30.0, "top"), shape

    # a number scales both axes; a pair stretches
    scene = cm.Scene()
    scene.rect("k", h=4, w=4, at=(0, 0)).grow((3.0, 0.5))
    got = scene._payload()[0]
    assert (got["scale_x"], got["scale_y"]) == (3.0, 0.5), got

    try:
        cm.Scene().rect("k", h=4, w=4, at=(0, 0)).turn(10, "middle")
    except ValueError:
        pass
    else:
        raise AssertionError("an unknown pivot should not reach the Engine")


def test_an_image_is_placed_by_a_fit_box_like_an_svg():
    """Two imports, one sizing rule (ADR 0013). `size` is a box the picture
    fits inside with its aspect kept; leaving it out draws the file at its own
    pixel size, which `w`/`h` of zero says."""
    scene = cm.Scene()
    scene.image("shot", "screen.png", size=(520, 300), at=(100, 200))
    shape = scene._payload()[0]
    assert shape["kind"] == "image", shape["kind"]
    assert shape["text"] == "screen.png", "the path travels in `text`"
    assert (shape["w"], shape["h"]) == (520.0, 300.0)

    square = cm.Scene()
    square.image("logo", "logo.png", size=90, at=(0, 0))
    assert square._payload()[0]["w"] == 90.0, "a scalar is a square box"

    natural = cm.Scene()
    natural.image("as_is", "logo.png", at=(0, 0))
    got = natural._payload()[0]
    assert (got["w"], got["h"]) == (0.0, 0.0), "no box means the file's own size"


def test_an_svg_keeps_its_own_colours_until_told_otherwise():
    """`color` is empty for an import, meaning "as authored" (ADR 0014).

    `"white"` could not say that: it is the default every other shape carries,
    so there would be no way to tell a deliberate white silhouette from a
    caller who said nothing at all.
    """
    scene = cm.Scene()
    scene.svg("logo", "brand.svg", size=90, at=(100, 200))
    shape = scene._payload()[0]
    assert shape["kind"] == "svg", shape["kind"]
    assert shape["text"] == "brand.svg", "the path travels in `text`"
    assert shape["color"] == "", "empty means as authored"
    assert (shape["w"], shape["h"]) == (90.0, 90.0), "a scalar size is a square box"
    assert shape["r"] == 1.0, "fully revealed unless .write says otherwise"
    assert shape["size"] == 0.0, "no pen unless .write says otherwise"

    scene.svg("wide", "flow.svg", size=(900, 420), at=(640, 360))
    wide = scene._payload()[1]
    assert (wide["w"], wide["h"]) == (900.0, 420.0), "a pair is the box"

    flat = cm.Scene()
    flat.svg("logo", "brand.svg", at=(0, 0)).fill("red")
    assert flat._payload()[0]["color"] == "red", "fill overrides every path"


def test_the_pen_rides_in_a_different_field_for_an_svg():
    """`w` is a formula's pen and an SVG's fit box, so `write` has to know
    which it is holding. This is the one place the payload's per-kind field
    reuse is visible from Python."""
    formula = cm.Scene()
    formula.formula("eq", r"x^2", size=30, at=(0, 0)).write(reveal=0.5, pen=2.0)
    assert formula._payload()[0]["w"] == 2.0, "a formula's pen is `w`"

    svg = cm.Scene()
    svg.svg("logo", "brand.svg", size=90, at=(0, 0)).write(reveal=0.5, pen=2.0)
    shape = svg._payload()[0]
    assert shape["size"] == 2.0, "an SVG's pen is `size`"
    assert shape["w"] == 90.0, "and its box survives"
    assert shape["r"] == 0.5


def test_a_curve_is_drawn_through_its_points_not_filled():
    """`curve` hands the Engine samples, not control points (ADR 0012).

    An open one is stroked the way a line is, because it encloses nothing —
    so its width rides on `edge_w`, `w` being the closed flag a polygon uses.
    """
    scene = cm.Scene()
    scene.curve("wave", [(0, 0), (10, 20), (20, 0)], w=3).fill("cyan")
    shape = scene._payload()[0]
    assert shape["kind"] == "curve", shape["kind"]
    assert shape["points"] == (0.0, 0.0, 10.0, 20.0, 20.0, 0.0), shape["points"]
    assert shape["w"] == 0.0, "open"
    assert shape["edge_w"] == 3.0, "the stroke width travels as edge_w"

    closed = cm.Scene()
    closed.curve("ring", [(0, 0), (10, 20), (20, 0)], closed=True)
    assert closed._payload()[0]["w"] == 1.0, "closed"

    # Anchored at the middle of its samples, so it travels as one thing.
    assert (shape["x"], shape["y"]) == (10.0, 10.0), (shape["x"], shape["y"])

    try:
        cm.Scene().curve("k", [(0, 0)])
    except ValueError:
        pass
    else:
        raise AssertionError("one point is not a curve")


def test_a_polygon_carries_its_corners_flat():
    """`points` is the one payload field that is not a single number (ADR 0010),
    so it is worth pinning that it stays flat and in order."""
    scene = cm.Scene()
    scene.polygon("tri", [(0, 0), (10, 0), (5, 8)])
    shape = scene._payload()[0]
    assert shape["kind"] == "polygon"
    assert shape["points"] == (0.0, 0.0, 10.0, 0.0, 5.0, 8.0), shape["points"]
    # anchored at the middle of its corners, so the whole shape travels as one
    assert (shape["x"], shape["y"]) == (5.0, 4.0), shape


def test_an_arrow_is_one_shape_not_two():
    """A line plus a separate head would be two names, and they could drift
    apart. One polygon travels as one thing."""
    scene = cm.Scene()
    scene.arrow("a", start=(0, 0), end=(100, 0))
    payload = scene._payload()
    assert len(payload) == 1 and payload[0]["kind"] == "polygon", payload


def test_ngon_and_star_return_points_rather_than_drawing():
    """They compose: the corners can be shifted, measured, or handed on."""
    assert len(cm.ngon(3, r=10)) == 3
    assert len(cm.ngon(6, r=10)) == 6
    assert len(cm.star(5, r=10)) == 10          # a point and a valley each
    try:
        cm.ngon(2, r=10)
    except ValueError:
        pass
    else:
        raise AssertionError("two sides is not a polygon")


def test_a_shape_can_be_filled_and_outlined_at_once():
    """Two separate colours, which the engine always supported and the surface
    used to collapse into one. Without it a bordered box is two stacked
    rectangles, and a banded ring is four concentric discs."""
    scene = cm.Scene()
    scene.rect("b", w=10, h=10, at=(0, 0)).fill("#1b2332", edge="#4ade80", edge_w=3)
    shape = scene._payload()[0]
    look = (shape["color"], shape["edge"], shape["edge_w"])
    assert look == ("#1b2332", "#4ade80", 3.0)

    # and a shape that asks for no edge keeps exactly the old behaviour
    scene = cm.Scene()
    scene.rect("b", w=10, h=10, at=(0, 0)).fill("blue")
    plain = scene._payload()[0]
    assert plain["edge_w"] == 0.0


def test_a_rounded_rect_carries_its_radius():
    scene = cm.Scene()
    scene.rect("b", w=10, h=10, at=(0, 0)).round(4)
    shape = scene._payload()[0]
    assert shape["kind"] == "rect" and shape["r"] == 4, shape


def test_focus_names_things_rather_than_coordinates():
    """The camera is aimed by name, so a layout change cannot leave it pointing
    at whitespace — the name is the part that stays stable."""
    scene = cm.Scene()
    scene.rect("box", w=10, h=10, at=(100, 100))
    assert scene._camera() is None, "no camera unless one is asked for"

    scene.focus("box", pad=12, least=200)
    camera = scene._camera()
    assert camera["names"] == ["box"], camera
    assert (camera["pad"], camera["min_size"]) == (12.0, 200.0), camera


def test_an_overlay_is_listed_as_fixed():
    """Whatever is on the overlay must reach the Engine as camera-exempt, or the
    first zoom pushes the narration off the frame."""
    scene = cm.Scene()
    scene.rect("box", w=10, h=10, at=(100, 100))
    scene.overlay().text("title", "hello", at=(640, 52))
    scene.focus("box")
    assert scene._camera()["fixed"] == ["_overlay"], scene._camera()


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
