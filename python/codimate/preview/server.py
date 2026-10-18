"""The Previewer's window: a film held open, and the local page that shows it."""

from __future__ import annotations

import json
import re
import threading
from pathlib import Path
from ..index import LiveIndex
from .lookup import build, covered, load, on_screen


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
            self.index = load(self.video)       # lazy: nothing but its table is read
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
            # Not `index_of`: that builds every shape of every beat, seconds and
            # hundreds of megabytes on a dense film, to answer for one beat at a
            # time. The scenes are already here; ask them.
            index = LiveIndex(explanation)
        except BaseException as e:   # a broken edit must not take the window down
            if isinstance(e, KeyboardInterrupt):
                raise
            self.error = "".join(traceback.format_exception_only(type(e), e)).strip()
            return
        self.explanation, self.index, self.error = explanation, index, None
        self.built += 1
        # The Engine builds the film on the first frame asked for — seconds,
        # on a big one. Ask now, in the background, rather than on first scrub.
        threading.Thread(target=self.frame, args=(0.0,), daemon=True).start()

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
                from ..explain import _ffmpeg
                subprocess.run([_ffmpeg(), "-v", "error", "-ss", f"{seconds:.3f}",
                                "-i", str(self.video), "-frames:v", "1", out],
                               check=True)
            else:
                self.explanation.frame_at(seconds, out)
            return Path(out).read_bytes()

    def listing(self) -> dict:
        """Everything the page needs, in one answer — the whole index
        included, so pointing at the picture is answered in the browser."""
        beats = self.index.beats
        # Shapes stay here and are asked for a beat at a time: a film where
        # everything moves every beat has an index of hundreds of megabytes.
        light = [{k: v for k, v in b.items() if k != "shapes"} for b in beats]
        return {"title": " ".join([self.path.name, *self.args]),
                "key": str(self.path) + "|" + " ".join(self.args),
                "video": bool(self.video),
                "canvas": self.index.canvas, "built": self.built,
                "error": self.error,
                "covered": covered(self.index),
                "chapters": [[b["at"], b["chapter"]] for b in beats if "chapter" in b],
                "sound": bool(self.explanation and self.explanation.sounds()),
                "duration": (beats[-1]["at"] + beats[-1]["secs"]) if beats else 0,
                "beats": light}


def serve(path, args=(), port: int = 0, show: bool = True) -> None:
    """Open the Previewer on a script or an mp4 and wait until interrupted.

    Bound to 127.0.0.1 and serving a handful of fixed routes and the page's own
    folder, so nothing else on the disk is reachable.
    """
    import webbrowser
    from http.server import ThreadingHTTPServer

    print(f"running {Path(path).name} up to its render call…", flush=True)
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

    viewer = Path(__file__).resolve().parent.parent / "viewer"
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
                # Asked with the build it has, it gets a few bytes back until
                # there is a new one.
                film.refresh(force="force" in query)
                since = query.get("since", [None])[0]
                if since == str(film.built) and "force" not in query:
                    self._json({"same": True, "error": film.error})
                else:
                    self._json(film.listing())
            elif url.path == "/shapes":
                try:
                    beat = int(query["beat"][0])
                    shapes = on_screen(film.index, beat)
                except (KeyError, ValueError, IndexError):
                    self._send(400, "text/plain", b"give a beat in the index")
                    return
                self._json({"built": film.built, "beat": beat, "shapes": shapes})
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
