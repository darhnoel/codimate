"""Where things sit: the canvas, Slots, row, column, anchors."""

import support  # noqa: F401  (puts `codimate` on the import path)
import codimate as cm


def test_a_row_is_centred_and_evenly_spaced():
    cm.canvas(1280, 720)
    slots = [slot for slot, _ in cm.row([1, 2, 3, 4], gap=40)]
    assert len(slots) == 4

    assert abs(slots[0].left - (cm.width() - slots[-1].right)) < 1e-6, "margins must match"
    for a, b in zip(slots, slots[1:]):
        assert abs((b.left - a.right) - 40) < 1e-6
    assert len({round(s.bottom, 6) for s in slots}) == 1, "a row shares one baseline"


def test_a_column_is_centred_and_evenly_spaced():
    cm.canvas(1280, 720)
    slots = list(cm.column(4, gap=40, w=64, x=300))
    assert {s.x for s in slots} == {300.0}, "a column shares one x"
    assert abs(slots[0].top - (cm.height() - slots[-1].bottom)) < 1e-6
    for a, b in zip(slots, slots[1:]):
        assert abs((b.top - a.bottom) - 40) < 1e-6

    high = list(cm.column(4, gap=40, y=200))
    assert abs(sum(s.y for s in high) / 4 - 200.0) < 1e-6, "y re-centres the column"


def test_layout_follows_the_canvas():
    cm.canvas(1280, 720)
    small = [s for s, _ in cm.row([1, 2, 3, 4], gap=40)][0]
    cm.canvas(1920, 1080)
    assert [s for s, _ in cm.row([1, 2, 3, 4], gap=40)][0].w > small.w
    cm.canvas(1280, 720)


def test_within_divides_a_region_not_the_canvas():
    cm.canvas(1280, 720)
    box = cm.Slot(x=900.0, y=500.0, w=400.0, h=300.0)
    slots = list(cm.row(4, gap=20, within=box))

    assert abs(slots[0].left - box.left) == abs(box.right - slots[-1].right), "centred in the box"
    assert slots[0].left >= box.left and slots[-1].right <= box.right, "stays inside"
    assert all(s.bottom <= box.bottom for s in slots)


def test_a_row_slot_anchors_bottom_centre():
    slot = cm.Slot(x=100.0, y=200.0, w=60.0, h=60.0, anchor="bottom")
    assert slot.point() == (100.0, 230.0)
    assert cm.Slot(x=1.0, y=2.0, w=4.0, h=6.0).point() == (1.0, 2.0), "centre by default"


def test_anchors_resolve_to_the_centre_the_engine_wants():
    cm.canvas(1280, 720)
    s = cm.Scene().rect("bar", x=100, bottom=560, w=60, h=200)
    assert s._payload()[0]["y"] == 460.0

    s = cm.Scene().rect("bar", left=100, top=50, w=60, h=200)
    assert (s._payload()[0]["x"], s._payload()[0]["y"]) == (130.0, 150.0)

    s = cm.Scene().text("label", 3, x=100, top=580, size=32)
    assert s._payload()[0]["y"] == 596.0, "text is placed by its centre, never a baseline"


def test_an_ambiguous_anchor_is_rejected():
    for kwargs in ({"x": 1, "left": 2, "y": 0}, {"x": 1}, {"y": 1}):
        try:
            cm.Scene().rect("bar", w=10, h=10, **kwargs)
        except ValueError:
            pass
        else:
            raise AssertionError(f"should be rejected: {kwargs}")


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
