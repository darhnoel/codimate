"""What happened: recording a Trace, and Items that survive it."""

import support  # noqa: F401  (puts `codimate` on the import path)
import codimate as cm


@cm.trace()
def swap_once(values):
    values[0], values[1] = values[1], values[0]
    cm.emit("swap", items=[values[0], values[1]])


def test_emit_snapshots_after_the_mutation():
    t = swap_once(cm.items([3, 1]))
    assert [i.value for i in t.initial] == [3, 1], "the initial state is captured first"
    assert len(t.events) == 1
    assert [i.value for i in t.events[0].state] == [1, 3]
    assert t.events[0].name == "swap"


def test_two_equal_values_are_two_different_things():
    a, b = cm.Item(3), cm.Item(3)
    assert a != b
    assert a.id != b.id
    assert len({a, b}) == 2


def test_items_order_by_value_so_an_algorithm_reads_normally():
    assert cm.Item(1) < cm.Item(2)
    assert cm.Item(5) > cm.Item(2)
    assert cm.Item(3) <= cm.Item(3), "equal values compare equal by ORDER"
    assert cm.Item(3) >= cm.Item(3)
    assert sorted(cm.items([3, 1, 2]), key=lambda i: i.value)[0].value == 1


def test_an_item_survives_the_snapshot_as_itself():
    """`frame.state` is deep-copied at every event but `event.data` is not, so
    a view comparing the two must still match. Equality by id survives the
    copy; equality by `is` would not."""

    @cm.trace()
    def touch(values):
        cm.emit("compare", items=[values[0]])

    ev = touch(cm.items([7, 8])).events[0]
    assert ev.state[0] is not ev.data["items"][0], "the snapshot really is a copy"
    assert ev.state[0] in ev.data["items"]


def test_a_frame_reports_what_just_happened():
    ev = swap_once(cm.items([3, 1])).events[0]
    frame = cm.Frame(state=ev.state, event=ev)
    assert frame.is_("swap") and not frame.is_("compare")
    assert len(frame.items()) == 2

    opening = cm.Frame(state=[], event=None)
    assert not opening.is_("swap"), "the opening moment came from no event"
    assert opening.items() == []


def test_emit_outside_a_traced_function_is_rejected():
    try:
        cm.emit("stray")
    except RuntimeError:
        pass
    else:
        raise AssertionError("emit() outside @trace should be rejected")


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
