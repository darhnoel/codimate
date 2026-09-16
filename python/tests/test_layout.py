"""Where things sit: the canvas, Slots, row, column, anchors."""

import support  # noqa: F401  (puts `codimate` on the import path)
import codimate as cm


def test_a_row_is_centred_and_evenly_spaced():
    cm.canvas(1280, 720)
    slots = [slot for slot, _ in cm.row([1, 2, 3, 4], gap=40)]
    assert len(slots) == 4

    margin = abs(slots[0].left - (cm.width() - slots[-1].right))
    assert margin < 1e-6, "margins must match"
    for a, b in zip(slots, slots[1:]):
        assert abs((b.left - a.right) - 40) < 1e-6
    assert len({round(s.bottom, 6) for s in slots}) == 1, "a row shares one baseline"


def test_a_column_is_centred_and_evenly_spaced():
    cm.canvas(1280, 720)
    slots = list(cm.column(4, gap=40, size=64, at=cm.at(x=300)))
    assert {s.x for s in slots} == {300.0}, "a column shares one x"
    assert abs(slots[0].top - (cm.height() - slots[-1].bottom)) < 1e-6
    for a, b in zip(slots, slots[1:]):
        assert abs((b.top - a.bottom) - 40) < 1e-6

    high = list(cm.column(4, gap=40, at=cm.at(y=200)))
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

    assert (abs(slots[0].left - box.left)
            == abs(box.right - slots[-1].right)), "centred in the box"
    assert slots[0].left >= box.left and slots[-1].right <= box.right, "stays inside"
    assert all(s.bottom <= box.bottom for s in slots)


def test_a_row_slot_anchors_bottom_centre():
    slot = cm.Slot(x=100.0, y=200.0, w=60.0, h=60.0, anchor="bottom")
    assert slot.point() == (100.0, 230.0)
    middle = cm.Slot(x=1.0, y=2.0, w=4.0, h=6.0).point()
    assert middle == (1.0, 2.0), "centre by default"


def test_anchors_resolve_to_the_centre_the_engine_wants():
    cm.canvas(1280, 720)
    s = cm.Scene()
    s.rect("bar", w=60, h=200, at=cm.at(x=100, bottom=560))
    assert s._payload()[0]["y"] == 460.0

    s = cm.Scene()
    s.rect("bar", w=60, h=200, at=cm.at(x=130, top=50))
    assert (s._payload()[0]["x"], s._payload()[0]["y"]) == (130.0, 150.0)

    s = cm.Scene()
    s.text("label", 3, size=32, at=cm.at(x=100, top=580))
    assert s._payload()[0]["y"] == 596.0, \
        "text is placed by its centre, never a baseline"


def test_an_ambiguous_anchor_is_rejected():
    for kwargs in ({"y": 1, "top": 2, "x": 0}, {"x": 1}, {"y": 1}):
        try:
            cm.Scene().rect("bar", w=10, h=10)
        except ValueError:
            pass
        else:
            raise AssertionError(f"should be rejected: {kwargs}")


def test_measure_is_real_and_not_a_guess():
    """The whole point is that it beats `len(text) * size * k`.

    Two strings of equal length must not measure equal when they are in
    different scripts — that is exactly the case an estimate gets wrong, and
    the reason boxes around Khmer used to be sized by rendering and squinting.
    """
    w, h = cm.measure("cat", 30)
    assert w > 0 and h > 0, (w, h)

    # Scales linearly with size. The tolerance is a quantisation budget, not a
    # slack: harfbuzz reports advances in 1/64ths, and how a build rounds them
    # differs between harfbuzz versions — the copy the wheels vendor is 3/64
    # off doubling where a system harfbuzz is exact. What this is really
    # guarding against is hinting or a bitmap strike, which would be off by a
    # percent or more rather than a hundredth of a pixel.
    w2, h2 = cm.measure("cat", 60)
    assert abs(w2 - 2 * w) < 0.1 and abs(h2 - 2 * h) < 0.1, (w, w2, h, h2)

    # Longer text is wider.
    assert cm.measure("cattle", 30)[0] > w

    # A different script goes through font fallback, so it does not land on
    # the same width a character count would predict.
    khmer = cm.measure("អរិយ", 30)[0]
    assert khmer != cm.measure("abcd", 30)[0], khmer


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))


def test_axes_map_the_corners_of_their_box():
    cm.canvas(1280, 720)
    plot = cm.axes(x=(-4, 4), y=(-2, 6), size=(760, 420))

    assert plot.at(-4, -2) == (plot.left, plot.bottom)
    assert plot.at(4, 6) == (plot.right, plot.top)
    # Screen y grows downward; data y does not.
    assert plot.at(0, 6)[1] < plot.at(0, -2)[1]

    mid = plot.at(0, 2)
    assert abs(mid[0] - (plot.left + plot.right) / 2) < 1e-9
    assert abs(mid[1] - (plot.top + plot.bottom) / 2) < 1e-9


def test_axes_steps_come_from_the_one_two_five_sequence():
    cm.canvas(1280, 720)
    for span, want in ((8, 2.0), (10, 2.0), (1, 0.2), (37, 10.0), (0.09, 0.02)):
        plot = cm.axes(x=(0, span), y=(0, 1))
        step = {v for _, v, _ in plot.ticks("x")}
        gaps = sorted({round(b - a, 9)
                       for a, b in zip(sorted(step), sorted(step)[1:])})
        assert gaps == [want], (span, gaps, want)


def test_a_tick_that_survives_a_pan_keeps_its_name():
    """The reason ticks are named after the step multiple and not the value.

    A name is an identity: rename a tick and it leaves and a new one enters,
    which is a fade — on a picture that only slid sideways.
    """
    cm.canvas(1280, 720)
    before = cm.axes(x=(0, 10), y=(0, 1)).ticks("x")
    after = cm.axes(x=(1, 11), y=(0, 1)).ticks("x")

    shared = {n for n, _, _ in before} & {n for n, _, _ in after}
    assert len(shared) >= 4, "a pan of one tenth must not rename everything"
    for n in shared:
        kept = next(v for m, v, _ in before if m == n)
        still = next(v for m, v, _ in after if m == n)
        assert kept == still, "the same name must mean the same value"


def test_a_range_that_only_drifts_does_not_renumber_its_ticks():
    cm.canvas(1280, 720)
    steady = cm.axes(x=(0, 10), y=(0, 1)).ticks("x")
    for nudge in (1e-12, 1e-9, 1e-7):
        assert cm.axes(x=(0, 10 + nudge), y=(0, 1)).ticks("x") == steady, nudge


def test_axes_labels_never_read_as_minus_zero():
    cm.canvas(1280, 720)
    words = {w for _, _, w in cm.axes(x=(-1, 1), y=(0, 1)).ticks("x")}
    assert "-0" not in words and "-0.0" not in words
    assert "0" in words


def test_a_plotted_line_keeps_its_point_count_off_the_page():
    """Dropping points that fall outside would be prettier and would stop the
    curve animating: two curves of different lengths do not interpolate."""
    cm.canvas(1280, 720)
    plot = cm.axes(x=(-1, 1), y=(0, 1), size=(400, 200))
    assert len(plot.line(lambda t: t * 1000, steps=50)) == 51
    assert len(plot.line(lambda t: 0.5, steps=50)) == 51


def test_axes_draw_shapes_under_a_name_you_chose():
    cm.canvas(1280, 720)
    scene = cm.Scene()
    plot = cm.axes(x=(0, 4), y=(0, 4), size=(400, 400)).draw(scene, "left")
    cm.axes(x=(0, 4), y=(0, 4), size=(400, 400)).draw(scene, "right", grid=True)

    # Names flatten to slash-joined strings, so a tuple key stays readable.
    names = set(scene._shapes)
    assert "left/x-axis" in names and "right/x-axis" in names
    assert "left/label/x/1" in names
    assert "left/grid/x/1" not in names, "grid is off by default"
    assert "right/grid/x/1" in names
    assert plot.at(2, 2) == (plot.left + 200.0, plot.top + 200.0)


def test_axes_reject_a_range_with_no_width():
    cm.canvas(1280, 720)
    for bad in (dict(x=(1, 1), y=(0, 1)), dict(x=(0, 1), y=(2, 2))):
        try:
            cm.axes(**bad)
        except ValueError:
            continue
        raise AssertionError(f"{bad} should not be allowed")
