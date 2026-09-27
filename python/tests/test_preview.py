"""The Previewer: running a script without rendering it, and asking what is
on screen. Nothing here encodes a video."""

import json
import sys
import tempfile
import threading
import urllib.error
import urllib.request
from pathlib import Path

import support  # noqa: F401  (puts `codimate` on the import path)
from codimate import preview

# A whole film in one script, the way an example is written: it imports a
# sibling module and reads its own argv. The render call must never happen.
SCRIPT = '''
import sys
import codimate as cm
import sibling

def swap(values, emit):
    values[0], values[1] = values[1], values[0]
    emit("swap")

def view(frame):
    scene = cm.Scene()
    for slot, item in cm.row(frame.state, gap=10):
        scene.rect(item.id, w=sibling.WIDE, h=20, at=slot)
    scene.rect("floor", w=1280, h=4, at=(640, 700)).on(layer=-1)
    return scene

cm.explain(trace=cm.trace(swap, cm.items([3, 1])), view=view,
           timing=cm.Timing(default=float(sys.argv[1]))
           ).render(__file__.replace("film.py", "never.mp4"))
raise SystemExit("render() came back")
'''


def folder(wide=40):
    here = Path(tempfile.mkdtemp())
    (here / "film.py").write_text(SCRIPT)
    (here / "sibling.py").write_text(f"WIDE = {wide}\n")
    return here


def test_a_script_is_caught_at_its_render_call():
    """Run to `render`, hand back the Explanation, draw nothing."""
    here = folder()
    exp = preview.build(here / "film.py", ["2.0"])
    assert exp.durations[1] == 2.0, "the script saw its own argv"
    assert not (here / "never.mp4").exists()
    assert not Path("never.mp4").exists()
    assert "sibling" not in sys.modules, "forgotten, so an edit to it is seen"


def test_an_edit_to_a_sibling_module_is_seen_on_the_next_run():
    here = folder(wide=40)
    first = preview.index_of(preview.build(here / "film.py", ["1.0"]))
    (here / "sibling.py").write_text("WIDE = 90\n")
    second = preview.index_of(preview.build(here / "film.py", ["1.0"]))
    width = lambda index: index["beats"][0]["shapes"]["set"][0]["box"][2]
    assert (width(first), width(second)) == (40, 90)


def test_a_script_that_never_renders_says_so():
    here = Path(tempfile.mkdtemp())
    (here / "quiet.py").write_text("x = 1\n")
    try:
        preview.build(here / "quiet.py")
    except RuntimeError as e:
        assert "without calling render()" in str(e)
    else:
        raise AssertionError("expected a RuntimeError")


def test_a_click_finds_the_topmost_shape_first():
    """The floor is under everything; the bar above it wins the click."""
    exp = preview.build(folder() / "film.py", ["1.0"])
    index = preview.index_of(exp)
    everything = preview.at(index, 0.1)["shapes"]
    bar = next(s for s in everything if s["name"] != "floor")
    x, y, w, h = bar["box"]
    hit = preview.at(index, 0.1, (x + w / 2, y + h / 2))["shapes"]
    assert [s["name"] for s in hit] == [bar["name"]]
    floor = preview.at(index, 0.1, (5, 700))["shapes"]
    assert [s["name"] for s in floor] == ["floor"], "a thin shape is still clickable"
    assert preview.at(index, 0.1, (5, 5))["shapes"] == []


def test_the_note_names_the_shape_the_event_and_the_state():
    exp = preview.build(folder() / "film.py", ["1.0"])
    found = preview.at(preview.index_of(exp), 1.5, (5, 700))
    text = preview.note(found, "  too thin  ")
    assert text.splitlines()[0] == "0:01.50  floor"
    assert "event    swap" in text
    assert text.endswith("> too thin")


def test_the_window_serves_frames_and_its_own_page():
    """The server end to end: a frame is a PNG drawn on request, and the page
    gets the whole index so pointing is answered in the browser."""
    from http.server import ThreadingHTTPServer

    film = preview.Film(folder() / "film.py", ["1.0"])
    server = ThreadingHTTPServer(("127.0.0.1", 0), preview._handler(film))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_address[1]}"
    try:
        listing = json.load(urllib.request.urlopen(f"{base}/index"))
        assert listing["video"] is False and listing["error"] is None
        assert listing["duration"] == film.explanation.duration

        png = urllib.request.urlopen(f"{base}/frame?t=0.5").read()
        assert png[:8] == b"\x89PNG\r\n\x1a\n"

        assert listing["beats"] == json.loads(json.dumps(film.index["beats"])), "the whole index"
        assert listing["title"] == "film.py 1.0"

        assert urllib.request.urlopen(f"{base}/").read().startswith(b"<!doctype html>")
        for page in ("app.js", "state.js", "style.css", "vendor/preact.js"):
            assert urllib.request.urlopen(f"{base}/viewer/{page}").status == 200

        # Nothing outside the page's own folder, however it is asked for.
        for sneaky in ("/viewer/../preview.py", "/viewer/../../codimate/preview.py",
                       "/viewer/vendor/README.md", "/viewer/"):
            try:
                urllib.request.urlopen(base + sneaky)
            except urllib.error.HTTPError as e:
                assert e.code == 404, sneaky
            else:
                raise AssertionError(f"served {sneaky}")
    finally:
        server.shutdown()
        server.server_close()


def test_the_window_lists_where_a_label_is_covered():
    """Reported once, where it starts, and read from the index — so an mp4
    with its index answers it as well as a script does."""
    here = Path(tempfile.mkdtemp())
    (here / "film.py").write_text('''
import codimate as cm

def grow(state, emit):
    for n in range(3):
        state["n"] = n
        emit("step")

def view(frame):
    scene = cm.Scene()
    scene.text("label", "iron", size=24, at=(200, 100))
    if frame.state.get("n", -1) >= 1:
        scene.line("arrow", start=(200, 40), end=(200, 160), w=4).on(layer=11)
    return scene

cm.explain(trace=cm.trace(grow, {}), view=view,
           timing=cm.Timing(default=1.0)).render("never.mp4")
''')
    film = preview.Film(here / "film.py")
    assert film.listing()["covered"] == [[2.8, "arrow", "label"]]
    assert film.explanation.covered() == [(2.8, "arrow", "label")]


def test_chapters_reach_the_window_with_their_start():
    here = Path(tempfile.mkdtemp())
    (here / "film.py").write_text('''
import codimate as cm

def story(state, emit):
    emit("open", chapter="Water")
    emit("more")
    emit("next", chapter="Ice")

cm.explain(trace=cm.trace(story, {}), view=lambda f: cm.Scene(),
           timing=cm.Timing(default=1.0, opening=0.5)).render("never.mp4")
''')
    film = preview.Film(here / "film.py")
    assert film.listing()["chapters"] == [[0.5, "Water"], [2.5, "Ice"]]
    assert film.explanation.chapters() == [(0.5, "Water"), (2.5, "Ice")]


def test_a_broken_edit_keeps_the_last_good_film():
    here = folder()
    film = preview.Film(here / "film.py", ["1.0"])
    (here / "sibling.py").write_text("WIDE = (\n")
    film.refresh(force=True)
    assert film.error and "SyntaxError" in film.error
    assert film.explanation is not None and film.built == 1


def test_an_mp4_is_served_in_ranges():
    """Browsers seek an mp4 by asking for pieces of it; Safari insists."""
    from http.server import ThreadingHTTPServer

    here = Path(tempfile.mkdtemp())
    (here / "x.mp4").write_bytes(bytes(range(256)) * 4)
    (here / "x.index.json").write_text(json.dumps(
        {"version": 1, "canvas": [1280, 720], "beats": []}))
    film = preview.Film(here / "x.mp4")
    server = ThreadingHTTPServer(("127.0.0.1", 0), preview._handler(film))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_address[1]}"
    try:
        asked = urllib.request.Request(f"{base}/video", headers={"Range": "bytes=10-19"})
        got = urllib.request.urlopen(asked)
        assert got.status == 206
        assert got.headers["Content-Range"] == "bytes 10-19/1024"
        assert got.read() == bytes(range(10, 20))
        assert json.load(urllib.request.urlopen(f"{base}/index"))["duration"] == 0
    finally:
        server.shutdown()
        server.server_close()


if __name__ == "__main__":
    for name, test in list(globals().items()):
        if name.startswith("test_"):
            test()
    print("ok")
