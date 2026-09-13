"""Putting it together: motion rules, timing, and the Engine's easing."""

import support  # noqa: F401  (puts `codimate` on the import path)
import codimate as cm


@cm.trace()
def swap_once(values):
    values[0], values[1] = values[1], values[0]
    cm.emit("swap", items=[values[0], values[1]])


def view(frame):
    scene = cm.Scene()
    for slot, item in cm.row(frame.state, gap=40):
        scene.group(item.id, slot).rect("bar", h=item.value * 40, at=cm.at(bottom=0))
    return scene


def built():
    return cm.explain(trace=swap_once(cm.items([3, 1])), view=view,
                      timing=cm.Timing(default=0.5))


def test_there_is_one_more_scene_than_duration():
    """What the Engine requires, counting the held opening and final hold."""
    e = built()
    assert len(e.scenes) == len(e.durations) + 1


def test_the_opening_and_ending_are_held():
    e = built()
    assert e.durations == [0.8, 0.5, 1.2], e.durations
    assert abs(e.duration - 2.5) < 1e-6
    assert e.scenes[0]._payload() == e.scenes[1]._payload(), "the opening goes nowhere"
    assert e.scenes[-1]._payload() == e.scenes[-2]._payload(), "so does the ending"


def test_a_named_event_can_take_its_own_duration():
    e = cm.explain(trace=swap_once(cm.items([3, 1])), view=view,
                   timing=cm.Timing(default=0.5, events={"swap": 1.25}))
    assert e.durations == [0.8, 1.25, 1.2], e.durations


def test_identity_not_order_is_what_pairs_scenes():
    e = built()
    first = {s["item"] for s in e.scenes[1]._payload()}
    last = {s["item"] for s in e.scenes[-1]._payload()}
    assert first == last and len(first) == 2


def test_a_rule_may_be_a_path_tuple_or_a_glob():
    assert cm.Rule(("item", "*")).pattern == "item/*"
    assert cm.Rule("*").pattern == "*"
    assert cm.Rule("*", position="linear").position == "linear"
    assert cm.Rule("*", clearance=90).options == {"clearance": 90.0}


def test_an_unknown_path_fails_at_authoring_time():
    try:
        cm.Rule("*", position="teleport")
    except ValueError:
        pass
    else:
        raise AssertionError("an unknown path should be rejected")


def test_the_engines_easing_is_a_usable_motion_curve():
    """cm.ease calls into the Engine, so a diagram of it cannot drift.

    Skipped when the extension is not built — the rest of these are pure
    Python on purpose.
    """
    try:
        cm.ease(0.5)
    except ImportError:
        print("      (skipped cm.ease — extension not built)")
        return

    assert cm.ease(0.0) == 0.0 and cm.ease(1.0) == 1.0
    assert cm.ease(0.5) == 0.5, "symmetric about its midpoint"
    assert cm.ease(0.25) < 0.25 < cm.ease(0.75), "starts slow and ends slow"
    assert cm.ease(-1.0) == 0.0 and cm.ease(2.0) == 1.0, "t is clamped"

    seen = [cm.ease(i / 40) for i in range(41)]
    assert all(b >= a for a, b in zip(seen, seen[1:])), "never travels backwards"


def test_the_bundled_ffmpeg_is_only_a_fallback():
    """A system ffmpeg is usually newer and hardware-accelerated, and it is the
    one the author expects to be used. The copy that ships with the wheel is
    there so a bare `pip install` works at all — not to take over."""
    import importlib
    import os
    import shutil

    module = importlib.import_module("codimate.explain")
    before = os.environ.pop("CODIMATE_FFMPEG", None)
    try:
        if shutil.which("ffmpeg"):
            module._find_encoder()
            assert "CODIMATE_FFMPEG" not in os.environ, (
                "a system ffmpeg was on PATH but the bundled one was chosen anyway")

        # An explicit choice is never second-guessed.
        os.environ["CODIMATE_FFMPEG"] = "/somewhere/of/my/own"
        module._find_encoder()
        assert os.environ["CODIMATE_FFMPEG"] == "/somewhere/of/my/own"
    finally:
        os.environ.pop("CODIMATE_FFMPEG", None)
        if before is not None:
            os.environ["CODIMATE_FFMPEG"] = before


def test_the_timeline_accounts_for_the_whole_video():
    """Every second of the render belongs to some beat.

    The timeline is what you read when a video feels wrong, so it has to agree
    with the video exactly — a second copy of the duration arithmetic would
    drift and send you looking at the wrong moment.
    """
    @cm.trace()
    def run(state):
        cm.emit("one")
        cm.emit("two")

    exp = cm.explain(trace=run({}), view=lambda f: cm.Scene(),
                     timing=cm.Timing(default=1.5, opening=1.0, final_hold=2.0))
    beats = exp.timeline()

    assert [n for _, _, n in beats] == ["(opening)", "one", "two", "(final hold)"]
    assert abs(sum(d for _, d, _ in beats) - sum(exp.durations)) < 1e-6

    # each beat starts where the previous one ended
    for (start, length, _), (next_start, _, _) in zip(beats, beats[1:]):
        assert abs(start + length - next_start) < 1e-6, beats


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
