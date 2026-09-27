"""The Previewer: a window onto a film, and a way to ask what is on it.

    python -m codimate.preview python/examples/archimedes/main.py km
    python -m codimate.preview results/archimedes-km.mp4

Given a **script**, it runs it up to its `render` call and keeps the film in
memory — no video is drawn. Each moment you scrub to is drawn on request, and
saving the script (or pressing Reload) runs it again. Given an **mp4**, it
plays the finished film, with its sound, against the index written beside it.

Either way: click what is on screen, and learn its name, the event that drew
it and the State behind it. Attach a note, copy the notes out.

**It never writes.** There is no save, no edit, no path back into your Python.
It produces words about the film; changing the film stays an edit to the one
file, made by you or by an agent acting on the note. See
[ADR 0018](../../docs/adr/0018-a-previewer-that-reads-an-index.md).

The lookup lives here rather than in the page, so the question it answers does
not need a window at all:

    from codimate import preview
    index = preview.load("results/archimedes-km.mp4")
    print(preview.note(preview.at(index, 55.0, (640, 290))))
"""

from __future__ import annotations

import json
import re
import sys
import threading
from pathlib import Path


def load(video: "str | Path") -> dict:
    """The index written beside `video` by `render`."""
    video = Path(video)
    beside = video.with_name(f"{video.stem}.index.json")
    if not beside.exists():
        raise FileNotFoundError(
            f"no index beside {video} — render it with index=True (the default)")
    return json.loads(beside.read_text())


def build(script: "str | Path", args=()):
    """Run `script` as `python script args…` would, up to its render call,
    and hand back the Explanation it was about to render.

    The script's folder is importable, as it is under `python`, and whatever
    the script imported from that folder is forgotten afterwards — so running
    it again picks up an edit to `vocabulary.py` as well as to `main.py`.
    """
    import runpy

    import importlib
    explain = importlib.import_module(f"{__package__}.explain")

    script = Path(script).resolve()
    folder = str(script.parent)
    caught = []
    saved_argv, saved_path, before = sys.argv, list(sys.path), set(sys.modules)
    # No .pyc: Python trusts one whose source has the same size and the same
    # second, which an edit made moments after the last run can easily have.
    saved_bytecode, sys.dont_write_bytecode = sys.dont_write_bytecode, True
    sys.argv = [str(script), *args]
    sys.path.insert(0, folder)
    explain._capture = caught.append
    try:
        runpy.run_path(str(script), run_name="__main__")
    except explain._Captured:
        pass
    finally:
        explain._capture = None
        sys.argv, sys.path[:] = saved_argv, saved_path
        sys.dont_write_bytecode = saved_bytecode
        for name in set(sys.modules) - before:
            where = getattr(sys.modules[name], "__file__", None) or ""
            if where.startswith(folder):
                del sys.modules[name]
    if not caught:
        raise RuntimeError(f"{script.name} finished without calling render()")
    return caught[0]


def index_of(explanation) -> dict:
    """The same index `render` writes beside a video, without the video."""
    from .layout import height, width
    return {"version": 1, "canvas": [width(), height()],
            "beats": explanation.index()}


def at(index: dict, seconds: float, point=None) -> dict:
    """What is on screen at `seconds`, and what put it there.

    Returns the beat — its event and the State behind it — and the shapes on
    screen, **topmost first**, the order the Engine draws in reversed. Give a
    `point` in canvas coordinates to keep only the shapes under it.

    Mid-beat, a shape is between where the last beat left it and where this
    one takes it. The nearer of the two is reported: good enough to identify
    what was clicked, and honest about being an approximation.
    """
    beats = index["beats"]
    before: dict = {}
    for i, beat in enumerate(beats):
        now = dict(before)
        for shape in beat["shapes"]["set"]:
            now[shape["name"]] = shape
        for name in beat["shapes"]["gone"]:
            now.pop(name, None)

        last = i == len(beats) - 1
        if seconds < beat["at"] + beat["secs"] or last:
            early = i > 0 and seconds < beat["at"] + beat["secs"] / 2
            shown = before if early else now
            return {"seconds": round(seconds, 3), "beat": i,
                    "event": beat["event"], "state": beat["state"],
                    "shapes": _under(shown.values(), point)}
        before = now
    raise ValueError("the index has no beats")


def covered(index: dict) -> "list[list]":
    """Every label something is drawn on, as `[seconds, above, below]`, from
    the index — so a rendered film answers it as well as a script does.
    Reported where each collision starts, at the moment its beat settles."""
    found, before = [], set()
    for beat in index["beats"]:
        now = {tuple(pair) for pair in beat.get("covered", [])}
        found += [[round(beat["at"] + beat["secs"], 2), a, b]
                  for a, b in sorted(now - before)]
        before = now
    return found


def _under(shapes, point):
    """The shapes at `point`, topmost first. A shape with no box cannot be hit."""
    # The Engine draws in (layer, name) order, so the last drawn is on top.
    ordered = sorted(shapes, key=lambda s: (s["layer"], s["name"]),
                     reverse=True)
    if point is None:
        return ordered
    x, y = point
    hit = []
    for shape in ordered:
        box = shape["box"]
        if box is None:
            continue
        left, top, w, h = box
        # A line is a box with no width or no height; give it a few pixels
        # so it can be clicked at all.
        pad = 4.0 if min(w, h) < 4.0 else 0.0
        if left - pad <= x <= left + w + pad and top - pad <= y <= top + h + pad:
            hit.append(shape)
    return hit


def note(found: dict, words: str = "") -> str:
    """The note a click turns into — plain text, for a person or an agent."""
    seconds = found["seconds"]
    stamp = f"{int(seconds // 60)}:{seconds % 60:05.2f}"
    shapes = found["shapes"]
    lines = [f"{stamp}  {shapes[0]['name'] if shapes else '(nothing here)'}",
             f"  event    {found['event']}"]
    if len(shapes) > 1:
        lines.append("  under    " + ", ".join(s["name"] for s in shapes[1:6]))
    lines.append(f"  state    {_brief(found['state'])}")
    if words.strip():
        lines.append(f"> {words.strip()}")
    return "\n".join(lines)


def _brief(state, room: int = 96) -> str:
    """The State on one line, short enough to read."""
    if isinstance(state, dict):
        text = "  ".join(f"{k}={_short(v)}" for k, v in state.items())
    else:
        text = _short(state, 200)
    return text if len(text) <= room else text[:room - 1] + "…"


def _short(value, room: int = 24) -> str:
    text = value if isinstance(value, str) else json.dumps(value, ensure_ascii=False)
    if isinstance(value, float):
        text = f"{value:.4g}"
    return text if len(text) <= room else text[:room - 1] + "…"


# ------------------------------------------------------------------ the film


class Film:
    """What the window is looking at: a script run in memory, or an mp4.

    One lock around everything, because the server answers requests on
    several threads and a rebuild swaps the Explanation out from under them.
    """
    # ponytail: one global lock; frames queue behind a rebuild, fine for one viewer

    def __init__(self, path, args=()):
        self.path = Path(path).resolve()
        self.args = list(args)
        self.video = self.path if self.path.suffix == ".mp4" else None
        self.lock = threading.Lock()
        self.built = 0          # bumped on every successful build
        self.error = None
        self.explanation = None
        if self.video:
            self.index = load(self.video)
        else:
            self.stamp = self._stamp()
            self._build()
            if self.error:
                raise RuntimeError(self.error)

    def _stamp(self):
        # Any .py beside the script, so an edit to a helper module counts.
        return max(p.stat().st_mtime for p in self.path.parent.glob("*.py"))

    def _build(self):
        import traceback
        try:
            explanation = build(self.path, self.args)
            index = index_of(explanation)
        except BaseException as e:   # a broken edit must not take the window down
            if isinstance(e, KeyboardInterrupt):
                raise
            self.error = "".join(traceback.format_exception_only(type(e), e)).strip()
            return
        self.explanation, self.index, self.error = explanation, index, None
        self.built += 1

    def refresh(self, force=False):
        """Run the script again if it changed since the last run."""
        if self.video:
            return
        with self.lock:
            stamp = self._stamp()
            if force or stamp != self.stamp:
                self.stamp = stamp
                self._build()

    def sound(self) -> "Path | None":
        """The film's clips mixed into one file, made once per build."""
        import tempfile
        with self.lock:
            if self.explanation is None or not self.explanation.sounds():
                return None
            if getattr(self, "_mixed", (None, None))[0] != self.built:
                folder = Path(tempfile.mkdtemp(prefix="codimate-preview-"))
                mixed = self.explanation.mix_sound(str(folder / "sound.wav"))
                self._mixed = (self.built, Path(mixed))
            return self._mixed[1]

    def frame(self, seconds: float) -> bytes:
        """The picture at `seconds`, as PNG: drawn from the script, or
        decoded out of the video — the filmstrip needs one either way."""
        import subprocess
        import tempfile
        with self.lock, tempfile.TemporaryDirectory() as folder:
            out = f"{folder}/frame.png"
            if self.video:
                from .explain import _ffmpeg
                subprocess.run([_ffmpeg(), "-v", "error", "-ss", f"{seconds:.3f}",
                                "-i", str(self.video), "-frames:v", "1", out],
                               check=True)
            else:
                self.explanation.frame_at(seconds, out)
            return Path(out).read_bytes()

    def listing(self) -> dict:
        """Everything the page needs, in one answer — the whole index
        included, so pointing at the picture is answered in the browser."""
        beats = self.index["beats"]
        return {"title": " ".join([self.path.name, *self.args]),
                "key": str(self.path) + "|" + " ".join(self.args),
                "video": bool(self.video),
                "canvas": self.index["canvas"], "built": self.built,
                "error": self.error,
                "covered": covered(self.index),
                "chapters": [[b["at"], b["chapter"]] for b in beats if "chapter" in b],
                "sound": bool(self.explanation and self.explanation.sounds()),
                "duration": (beats[-1]["at"] + beats[-1]["secs"]) if beats else 0,
                "beats": beats}


# ------------------------------------------------------------------ serving


def serve(path, args=(), port: int = 0, show: bool = True) -> None:
    """Open the Previewer on a script or an mp4 and wait until interrupted.

    Bound to 127.0.0.1 and serving a handful of fixed routes and the page's own
    folder, so nothing else on the disk is reachable.
    """
    import webbrowser
    from http.server import ThreadingHTTPServer

    film = Film(path, args)
    server = ThreadingHTTPServer(("127.0.0.1", port), _handler(film))
    address = f"http://127.0.0.1:{server.server_address[1]}/"
    print(f"previewing {film.path.name} at {address}  (ctrl-c to stop)")
    if show:
        webbrowser.open(address)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


def _handler(film: Film):
    from http.server import BaseHTTPRequestHandler
    from urllib.parse import parse_qs, urlparse

    viewer = Path(__file__).with_name("viewer").resolve()
    kinds = {".html": "text/html; charset=utf-8", ".js": "text/javascript",
             ".css": "text/css"}

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):          # quiet: this is a local tool
            pass

        def do_GET(self):
            url = urlparse(self.path)
            query = parse_qs(url.query)
            if url.path == "/" or url.path.startswith("/viewer/"):
                # The page's own files, and nothing outside their folder.
                asked = url.path[len("/viewer/"):] if url.path != "/" else "index.html"
                path = (viewer / asked).resolve()
                if viewer not in path.parents or not path.is_file() \
                        or path.suffix not in kinds:
                    self._send(404, "text/plain", b"not here")
                    return
                self._send(200, kinds[path.suffix], path.read_bytes())
            elif url.path == "/index":
                # Polled by the page: this is also how a saved edit arrives.
                film.refresh(force="force" in query)
                self._json(film.listing())
            elif url.path == "/frame" and (film.explanation or film.video):
                try:
                    seconds = float(query["t"][0])
                except (KeyError, ValueError):
                    self._send(400, "text/plain", b"give t")
                    return
                self._send(200, "image/png", film.frame(seconds))
            elif url.path == "/video" and film.video:
                self._file(film.video, "video/mp4")
            elif url.path == "/sound" and film.sound():
                self._file(film.sound(), "audio/wav")
            else:
                self._send(404, "text/plain", b"not here")

        def _file(self, video, kind):
            # Range requests, because a browser will not seek media it was
            # not allowed to fetch in pieces — Safari will not even play it.
            size = video.stat().st_size
            start, end = 0, size - 1
            asked = re.match(r"bytes=(\d*)-(\d*)", self.headers.get("Range", ""))
            if asked and (asked[1] or asked[2]):
                if asked[1]:
                    start = int(asked[1])
                    end = int(asked[2]) if asked[2] else size - 1
                else:                            # the last N bytes
                    start = max(size - int(asked[2]), 0)
                end = min(end, size - 1)
                self.send_response(206)
                self.send_header("Content-Range", f"bytes {start}-{end}/{size}")
            else:
                self.send_response(200)
            self.send_header("Content-Type", kind)
            self.send_header("Accept-Ranges", "bytes")
            self.send_header("Content-Length", str(end - start + 1))
            self.end_headers()
            with video.open("rb") as f:
                f.seek(start)
                left = end - start + 1
                while left > 0:
                    chunk = f.read(min(1 << 16, left))
                    if not chunk:
                        break
                    try:
                        self.wfile.write(chunk)
                    except (BrokenPipeError, ConnectionResetError):
                        return                   # the browser moved on
                    left -= len(chunk)

        def _json(self, value):
            self._send(200, "application/json",
                       json.dumps(value, ensure_ascii=False).encode())

        def _send(self, status, kind, body):
            self.send_response(status)
            self.send_header("Content-Type", kind)
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            try:
                self.wfile.write(body)
            except (BrokenPipeError, ConnectionResetError):
                pass                             # the page asked for a newer frame

    return Handler


def main(argv=None) -> int:
    import argparse

    parser = argparse.ArgumentParser(
        prog="python -m codimate.preview",
        description="Look at a film — a script, or a rendered mp4 — and ask "
                    "what is on screen.")
    parser.add_argument("path", help="a script that calls render(), or an mp4 "
                                     "rendered with its index beside it")
    parser.add_argument("args", nargs=argparse.REMAINDER,
                        help="passed to the script, as `python script args` would")
    parser.add_argument("--port", type=int, default=0)
    parser.add_argument("--no-open", action="store_true",
                        help="print the address instead of opening a browser")
    options = parser.parse_args(argv)
    serve(options.path, options.args, port=options.port,
          show=not options.no_open)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
