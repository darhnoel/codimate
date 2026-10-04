"""The index: the compact file reads back what was written, any beat can be asked
for in any order, and a film in memory answers the same as the file it would write.
Nothing here renders a video."""

import json
import random
import tempfile
from pathlib import Path

import support  # noqa: F401  (puts `codimate` on the import path)
import codimate as cm
from codimate import index as ix
from codimate import preview


def film(beats=40):
    """Bars that slide, appear and leave, over many beats, so chunks matter."""
    def algorithm(state, emit):
        for i in range(beats):
            state["x"] = i * 7.3
            state["n"] = i
            emit("step", chapter="Second half" if i == beats // 2 else "")

    def view(frame):
        scene = cm.Scene()
        x, n = frame.state["x"], frame.state["n"]
        scene.rect("slider", w=30.5, h=12.25, at=(100 + x, 200 + x / 9))
        scene.circle(("dot", n % 5), r=7 + n % 3, at=(50 * (n % 5) + 20, 300)) \
            .on(layer=n % 4)
        if n % 6 < 3:
            scene.text("blink", f"n={n}", size=18, at=(640, 100))
        return scene

    return cm.explain(trace=cm.trace(algorithm, {"x": 0.0, "n": 0}), view=view,
                      timing=cm.Timing(default=0.2))


def same(a, b, places=1 / ix.QUANT):
    """Two shape lists name the same things, a box within the file's resolution."""
    a, b = {s["name"]: s for s in a}, {s["name"]: s for s in b}
    assert a.keys() == b.keys()
    for name in a:
        assert (a[name]["kind"], a[name]["layer"]) == \
            (b[name]["kind"], b[name]["layer"])
        assert (a[name]["box"] is None) == (b[name]["box"] is None), name
        if a[name]["box"]:
            for u, v in zip(a[name]["box"], b[name]["box"]):
                assert abs(u - v) <= places, (name, u, v)


def test_a_film_in_memory_answers_as_the_index_it_would_write():
    """`LiveIndex` reads shapes off the scenes; the readable index replays
    differences. They must agree on every beat — it is why the Previewer can skip
    building the second."""
    exp = film()
    live, readable = ix.LiveIndex(exp), ix.DictIndex(
        {"canvas": live_canvas(), "beats": exp.index()})
    for beat in range(len(exp.timeline())):
        same(live.shapes_at(beat), readable.shapes_at(beat), places=0)


def live_canvas():
    return [cm.width(), cm.height()]


def test_the_compact_file_reads_back_what_was_written():
    exp = film()
    path = Path(tempfile.mkdtemp()) / "x.index.bin"
    ix.write(path, live_canvas(), exp._index_beats(), chunk=7)   # many small chunks
    compact, live = ix.CompactIndex(path), ix.LiveIndex(exp)
    assert compact.canvas == live.canvas and len(compact.beats) == len(live.beats)
    assert compact.beats == live.beats, "the per-beat table is exact"
    for beat in range(len(live.beats)):
        same(compact.shapes_at(beat), live.shapes_at(beat))


def test_any_beat_can_be_asked_for_in_any_order():
    """A chunk begins with everything on screen, so no beat depends on the ones
    before it having been read — a click is not a replay."""
    exp = film(beats=60)
    path = Path(tempfile.mkdtemp()) / "x.index.bin"
    ix.write(path, live_canvas(), exp._index_beats(), chunk=8)
    live = ix.LiveIndex(exp)
    asked = list(range(len(live.beats)))
    random.Random(7).shuffle(asked)
    compact = ix.CompactIndex(path)               # a fresh reader, nothing cached
    for beat in asked:
        same(compact.shapes_at(beat), live.shapes_at(beat))


def test_the_compact_file_changes_back_into_the_readable_index():
    """`changes()` is how a compact index is read by eye: the same beats the JSON
    file holds, differences and all (to the file's resolution)."""
    exp = film(beats=20)
    path = Path(tempfile.mkdtemp()) / "x.index.bin"
    ix.write(path, live_canvas(), exp._index_beats(), chunk=6)
    got = list(ix.CompactIndex(path).changes())
    want = exp.index()
    assert [b["event"] for b in got] == [b["event"] for b in want]
    assert [b.get("chapter") for b in got] == [b.get("chapter") for b in want]
    for g, w in zip(got, want):
        assert sorted(g["shapes"]["gone"]) == sorted(w["shapes"]["gone"])
        same(g["shapes"]["set"], w["shapes"]["set"])
    # a slice does not need the beats before it, and replays to the same screen
    middle = list(ix.CompactIndex(path).changes(9, 13))
    assert [b["at"] for b in middle] == [b["at"] for b in want[9:13]]
    json.dumps(middle)                             # and is plain, writable JSON


def test_a_shape_with_no_box_survives_the_file():
    """A formula on a machine without Typst has no box (ADR 0005); it must come
    back as having none, not as a box of zeros."""
    beats = [{"at": 0.0, "secs": 1.0, "event": "e", "state": None, "covered": [],
              "shapes": {"set": [{"name": "eq", "kind": "formula", "layer": 3,
                                  "box": None},
                                 {"name": "r", "kind": "rect", "layer": 0,
                                  "box": [1.5, 2.25, 10.0, 4.0]}], "gone": []}},
             {"at": 1.0, "secs": 1.0, "event": "e", "state": None, "covered": [],
              "shapes": {"set": [{"name": "r", "kind": "rect", "layer": 0,
                                  "box": [2.0, 2.25, 10.0, 4.0]}], "gone": ["eq"]}}]
    path = Path(tempfile.mkdtemp()) / "x.index.bin"
    ix.write(path, [1280, 720], iter(beats), chunk=1)
    compact = ix.CompactIndex(path)
    first = {s["name"]: s for s in compact.shapes_at(0)}
    assert first["eq"]["box"] is None and first["r"]["box"] == [1.5, 2.25, 10.0, 4.0]
    assert [s["name"] for s in compact.shapes_at(1)] == ["r"]


def test_an_empty_film_has_an_empty_index():
    path = Path(tempfile.mkdtemp()) / "x.index.bin"
    ix.write(path, [1280, 720], iter(()))
    compact = ix.CompactIndex(path)
    assert compact.beats == [] and compact.canvas == [1280, 720]
    assert list(compact.changes()) == []


def test_a_file_that_is_not_an_index_says_so():
    path = Path(tempfile.mkdtemp()) / "x.index.bin"
    path.write_bytes(b"this is not an index at all")
    try:
        ix.CompactIndex(path)
    except ValueError as wrong:
        assert "not a codimate index" in str(wrong)
    else:
        raise AssertionError("a foreign file was accepted")


def test_the_compact_file_is_far_smaller_than_the_readable_one():
    exp = film(beats=120)
    folder = Path(tempfile.mkdtemp())
    exp.write_compact_index(str(folder / "x.mp4"))
    exp.write_index(str(folder / "x.mp4"))
    compact = (folder / "x.index.bin").stat().st_size
    readable = (folder / "x.index.json").stat().st_size
    assert compact * 10 < readable, (compact, readable)


def test_the_previewer_finds_the_compact_index_beside_a_video():
    """`load` prefers `.index.bin`, falls back to `.index.json`, and says what to
    do when there is neither."""
    folder = Path(tempfile.mkdtemp())
    exp = film(beats=12)
    (folder / "x.mp4").write_bytes(b"not really a video")
    try:
        preview.load(folder / "x.mp4")
    except FileNotFoundError as missing:
        assert "no index beside" in str(missing)
    else:
        raise AssertionError("found an index that is not there")
    exp.write_index(str(folder / "x.mp4"))
    assert isinstance(preview.load(folder / "x.mp4"), ix.DictIndex)
    exp.write_compact_index(str(folder / "x.mp4"))
    assert isinstance(preview.load(folder / "x.mp4"), ix.CompactIndex), \
        "the compact one wins when both are there"
    found = preview.at(preview.load(folder / "x.mp4"), 1.1)   # after the opening hold
    assert found["event"] == "step" and found["shapes"]


def test_a_film_in_the_previewer_builds_no_index_of_its_shapes():
    """The window answers from the scenes it already holds."""
    script = Path(tempfile.mkdtemp()) / "film.py"
    script.write_text(
        "import codimate as cm\n"
        "def go(s, emit):\n    s['i'] = 1\n    emit('a')\n"
        "def view(f):\n    sc = cm.Scene()\n    sc.rect('r', w=20, h=20, at=(50, 50))\n"
        "    return sc\n"
        "cm.explain(trace=cm.trace(go, {'i': 0}), view=view).render('x.mp4')\n")
    film_ = preview.Film(script)
    assert isinstance(film_.index, ix.LiveIndex)
    assert all("shapes" not in beat for beat in film_.index.beats), \
        "only the per-beat table is built"
    assert [s["name"] for s in preview.on_screen(film_.index, 1)] == ["r"]


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
