"""Putting it together: motion rules, timing, and the Engine's easing."""

import support  # noqa: F401  (puts `codimate` on the import path)
import codimate as cm


def swap_once(values, emit):
    values[0], values[1] = values[1], values[0]
    emit("swap", items=[values[0], values[1]])


def view(frame):
    scene = cm.Scene()
    for slot, item in cm.row(frame.state, gap=40):
        scene.group(item.id, slot).rect("bar", h=item.value * 40, at=cm.at(bottom=0))
    return scene


def built():
    return cm.explain(trace=cm.trace(swap_once, cm.items([3, 1])), view=view,
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
    e = cm.explain(trace=cm.trace(swap_once, cm.items([3, 1])), view=view,
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
    def run(state, emit):
        emit("one")
        emit("two")

    exp = cm.explain(trace=cm.trace(run, {}), view=lambda f: cm.Scene(),
                     timing=cm.Timing(default=1.5, opening=1.0, final_hold=2.0))
    beats = exp.timeline()

    assert [n for _, _, n in beats] == ["(opening)", "one", "two", "(final hold)"]
    assert abs(sum(d for _, d, _ in beats) - sum(exp.durations)) < 1e-6

    # each beat starts where the previous one ended
    for (start, length, _), (next_start, _, _) in zip(beats, beats[1:]):
        assert abs(start + length - next_start) < 1e-6, beats


def test_the_index_answers_what_was_on_screen_and_why():
    """One entry per beat, carrying the State and every shape's box.

    The question a film provokes is "what is at 0:55, and what put it there?".
    Answering it by rendering a frame and squinting is how a whole session
    goes by; the index answers it as data (ADR 0018).
    """
    def swap(values, emit):
        values[0], values[1] = values[1], values[0]
        emit("swap")

    def view(frame):
        scene = cm.Scene()
        for slot, item in cm.row(frame.state, gap=10):
            scene.rect(item.id, w=40, h=20, at=slot)
        return scene

    values = cm.items([3, 1])
    names = {str(item.id) for item in values}
    exp = cm.explain(trace=cm.trace(swap, values), view=view,
                     timing=cm.Timing(default=1.0))
    index = exp.index()

    assert len(index) == len(exp.timeline()), "one entry per beat"
    assert [b["event"] for b in index] == ["(opening)", "swap", "(final hold)"]

    beat = index[1]
    assert beat["at"] == exp.timeline()[1][0], "the clock agrees with timeline"
    assert [i["value"] for i in beat["state"]] == [1, 3], "the state after the swap"
    assert {s["name"] for s in index[0]["shapes"]["set"]} == names, \
        "named by Item, not by slot"
    assert all(s["box"] for s in index[0]["shapes"]["set"])

    # Only what changed. The two bars swap places, so both move and both are
    # written; nothing is "gone", because neither left.
    assert {s["name"] for s in beat["shapes"]["set"]} == names
    assert beat["shapes"]["gone"] == []
    assert index[2]["shapes"]["set"] == [], "the final hold changes nothing"


def test_a_formula_has_a_box_centred_on_its_point():
    """Typst measures it — the same cached glyphs the Engine draws — so a
    formula can be clicked and checked like any label (ADR 0005)."""
    def once(state, emit):
        emit("one")

    def view(frame):
        scene = cm.Scene()
        scene.formula("maths", r"a^2", size=20, at=(200, 100))
        return scene

    shapes = {s["name"]: s for s in cm.explain(
        trace=cm.trace(once, {}), view=view).index()[0]["shapes"]["set"]}
    if shapes["maths"]["box"] is None:
        # Nothing to measure it with — no box rather than a guessed one — and
        # only then: with typst here, or its answer cached, there is a box.
        import shutil
        assert not shutil.which("typst")
        return
    left, top, w, h = shapes["maths"]["box"]
    assert (left + w / 2, top + h / 2) == (200, 100)
    assert (w, h) == cm.measure_math(r"a^2", 20)


def the_covered(*draw):
    """What the check reports for one picture drawn by `draw`."""
    def once(state, emit):
        emit("one")

    def view(frame):
        scene = cm.Scene()
        for step in draw:
            step(scene)
        return scene

    return [(a, b) for _, a, b in cm.explain(
        trace=cm.trace(once, {}), view=view).covered()]


def test_an_arrow_over_a_label_is_reported_and_its_plate_is_not():
    """ADR 0017: what is drawn above a label hides it; below it holds it up."""
    def label(s):
        s.text("name", "iron", size=24, at=(200, 100))
    def plate(s):
        s.rect("plate", w=120, h=40, at=(200, 100)).on(layer=-1)
    def arrow(s):
        s.line("arrow", start=(200, 40), end=(200, 160), w=4).on(layer=11)
    assert the_covered(plate, label) == []
    assert the_covered(plate, label, arrow) == [("arrow", "name")]


def test_any_opacity_counts_and_invisible_does_not():
    """No threshold: 46% water over a caption is still unreadable."""
    def label(s):
        s.text("name", "iron", size=24, at=(200, 100))
    def water(s):
        s.rect("water", w=300, h=300, at=(200, 100)).on(layer=11, opacity=0.46)
    def gone(s):
        s.rect("water", w=300, h=300, at=(200, 100)).on(layer=11, opacity=0)
    assert the_covered(label, water) == [("water", "name")]
    assert the_covered(label, gone) == []


def test_two_labels_in_one_place_are_reported_once():
    def a(s):
        s.text("a", "force", size=24, at=(200, 100))
    def b(s):
        s.text("b", "weight", size=24, at=(210, 104))
    assert the_covered(a, b) == [("b", "a")]


def test_words_set_edge_to_edge_do_not_collide():
    """A running subtitle is words laid side by side, touching exactly."""
    w, _ = cm.measure("one", 24)
    def a(s):
        s.text("a", "one", size=24, at=(200, 100))
    def b(s):
        s.text("b", "one", size=24, at=(200 + w, 100))
    assert the_covered(a, b) == []


def test_a_diagonal_line_near_a_label_misses_it():
    """Its box covers the label; the line itself does not."""
    def label(s):
        s.text("name", "iron", size=24, at=(200, 100))
    def line(s):
        s.line("edge", start=(150, 180), end=(300, 60), w=2).on(layer=11)
    assert the_covered(label, line) == []


def test_the_index_survives_a_state_it_does_not_understand():
    """An index that refused to be written for one odd object in the State
    would be no use at all, so anything unrecognised becomes its repr."""
    class Odd:
        def __repr__(self):
            return "<odd>"

    def once(state, emit):
        emit("one")

    written = cm.explain(trace=cm.trace(once, {"thing": Odd(), "n": 2}),
                         view=lambda f: cm.Scene()).index()[1]["state"]
    assert written["n"] == 2
    assert written["thing"] == {} or written["thing"] == "<odd>"


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))


def tone(path, seconds=0.2):
    """A short clip to lay down, made by the ffmpeg the render uses."""
    import subprocess
    from codimate.explain import _ffmpeg
    subprocess.run([_ffmpeg(), "-y", "-v", "error", "-f", "lavfi", "-i",
                    f"sine=frequency=440:duration={seconds}", str(path)], check=True)
    return str(path)


def length(path):
    """How long a media file is, as ffmpeg reads it."""
    import re
    import subprocess
    from codimate.explain import _ffmpeg
    said = subprocess.run([_ffmpeg(), "-i", str(path)],
                          capture_output=True, text=True).stderr
    h, m, s = re.search(r"Duration: (\d+):(\d+):([\d.]+)", said).groups()
    return int(h) * 3600 + int(m) * 60 + float(s)


def test_a_sound_starts_as_the_beat_of_its_event_begins():
    """ADR 0007: sound rides on the event, and the timeline says when."""
    import tempfile
    from pathlib import Path
    here = Path(tempfile.mkdtemp()).resolve()
    hello, bye = tone(here / "hello.wav"), tone(here / "bye.wav")

    def talk(state, emit):
        emit("quiet")
        emit("hello", sound=hello)
        emit("bye", sound=bye)

    exp = cm.explain(trace=cm.trace(talk, {}), view=lambda f: cm.Scene(),
                     timing=cm.Timing(default=1.0, opening=0.5))
    starts = {name: at for at, _, name in exp.timeline()}
    assert exp.sounds() == [(starts["hello"], hello), (starts["bye"], bye)]
    assert exp.sounds()[0][0] == 1.5, "after the opening and one quiet beat"


def test_a_missing_sound_fails_before_the_picture_is_drawn():
    def talk(state, emit):
        emit("hello", sound="/nowhere/hello.wav")

    import tempfile
    exp = cm.explain(trace=cm.trace(talk, {}), view=lambda f: cm.Scene())
    out = f"{tempfile.mkdtemp()}/never.mp4"
    try:
        exp.render(out)
    except FileNotFoundError as e:
        assert "hello.wav" in str(e) and "'hello'" in str(e)
    else:
        raise AssertionError("expected FileNotFoundError")
    from pathlib import Path
    assert not Path(out).exists()


def test_the_mix_lays_every_clip_at_its_start():
    """The last clip starts at 2.5s and lasts 0.2s: the mix is 2.7s long."""
    import tempfile
    from pathlib import Path
    here = Path(tempfile.mkdtemp())
    clip = tone(here / "a.wav")

    def talk(state, emit):
        emit("one", sound=clip)
        emit("two", sound=clip)

    exp = cm.explain(trace=cm.trace(talk, {}), view=lambda f: cm.Scene(),
                     timing=cm.Timing(default=2.0, opening=0.5))
    out = exp.mix_sound(str(here / "mix.wav"))
    assert abs(length(out) - 2.7) < 0.05


def test_sound_is_laid_under_a_video_without_redrawing_it():
    import subprocess
    import tempfile
    from pathlib import Path
    from codimate.explain import _ffmpeg
    here = Path(tempfile.mkdtemp())
    video = here / "v.mp4"
    subprocess.run([_ffmpeg(), "-y", "-v", "error", "-f", "lavfi", "-i",
                    "color=c=black:s=64x36:d=1", "-pix_fmt", "yuv420p", str(video)],
                   check=True)

    def talk(state, emit):
        emit("one", sound=tone(here / "a.wav"))

    exp = cm.explain(trace=cm.trace(talk, {}), view=lambda f: cm.Scene())
    exp._lay_sound(str(video), exp.sounds())
    streams = subprocess.run([_ffmpeg(), "-i", str(video)],
                             capture_output=True, text=True).stderr
    assert "Video:" in streams and "Audio:" in streams
    assert not list(here.glob(".*tmp*")), "the temporary is gone"
