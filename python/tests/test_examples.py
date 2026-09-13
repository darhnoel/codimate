"""Every example still runs, end to end.

The other files stop at the payload — the last thing Python produces. These
run the whole pipeline: view, diff, tween, rasterize, ffmpeg, file on disk.

Examples are discovered, not listed, so a new one is covered the day it is
added and nobody has to remember this file exists.

An example is only re-rendered when something it depends on has changed —
its own sources, the `codimate` package, or the compiled engine. So a run
that changes nothing costs a few ffprobe calls instead of eight renders.

    python python/tests/run.py             # skips this file
python python/tests/run.py --all       # includes it entirely
"""

import subprocess
import sys
from pathlib import Path

import support  # noqa: F401  (puts `codimate` on the import path)

EXAMPLES = sorted(Path(__file__).resolve().parents[1].glob("examples/*/main.py"))
ROOT = Path(__file__).resolve().parents[2]

WIDTH, HEIGHT, FPS = 1920, 1080, 60.0


def _probe(video, fields):
    out = subprocess.run(
        ["ffprobe", "-v", "error", "-select_streams", "v",
         "-show_entries", f"stream={fields}", "-of", "csv=p=0", str(video)],
        capture_output=True, text=True, check=True).stdout.strip()
    return out.split(",")


def _frames(video, count=12):
    """A frame a second, as small greyscale thumbnails.

    Sampled with `fps=`, not ffmpeg's `thumbnail` filter — that one picks a
    single representative frame per batch and hands back the same image every
    time, which makes any two of them look identical.
    """
    w, h = 160, 90
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", str(video),
         "-vf", f"fps=1,scale={w}:{h}", "-frames:v", str(count),
         "-f", "rawvideo", "-pix_fmt", "gray", "-"],
        capture_output=True).stdout
    size = w * h
    return [raw[i * size:(i + 1) * size] for i in range(len(raw) // size)]


PACKAGE = ROOT / "python" / "codimate"


def _sources(main_py):
    """Everything whose change could change the video.

    The compiled engine lives inside the package as a `.so`, so re-running
    `maturin develop` invalidates every example — which is what you want,
    since that is where most rendering behaviour actually lives.
    """
    yield from main_py.parent.rglob("*.py")
    yield from PACKAGE.rglob("*.py")
    yield from PACKAGE.rglob("*.so")


def _fresh(main_py):
    """The existing video, if nothing it depends on is newer than it.

    Conservative: an unexpected output path, a missing file, or any doubt
    means we render. A stale pass is far worse than a slow one.
    """
    video = ROOT / "results" / f"{main_py.parent.name}.mp4"
    if not video.exists():
        return None
    newest = max(p.stat().st_mtime for p in _sources(main_py))
    return video if video.stat().st_mtime > newest else None


# Rendered once per run, whichever test asks first. Without this the checks
# below would depend on each other's side effects — and on the order they
# happen to run in, which is how the first version of this file silently
# graded the *previous* run's videos.
_RENDERED = {}


def _video(main_py):
    if main_py not in _RENDERED:
        _RENDERED[main_py] = _fresh(main_py) or _render(main_py)
    return _RENDERED[main_py]


def _render(main_py):
    """Run one example the way a reader would, and return what it wrote."""
    done = subprocess.run([sys.executable, str(main_py)],
                          cwd=ROOT, capture_output=True, text=True)
    assert done.returncode == 0, f"{main_py.parent.name} failed:\n{done.stderr[-2000:]}"

    written = [line for line in done.stdout.splitlines() if line.startswith("wrote ")]
    assert written, f"{main_py.parent.name} printed no output path:\n{done.stdout}"

    video = ROOT / written[-1][len("wrote "):].strip()
    assert video.exists(), f"{video} was announced but not written"
    return video


def test_every_example_renders_a_video():
    assert EXAMPLES, "no examples found — has the layout changed?"

    for main_py in EXAMPLES:
        name = main_py.parent.name
        video = _video(main_py)

        w, h, rate = _probe(video, "width,height,r_frame_rate")
        num, den = rate.split("/")
        assert (int(w), int(h)) == (WIDTH, HEIGHT), f"{name}: {w}x{h}"
        assert abs(int(num) / int(den) - FPS) < 0.01, f"{name}: {rate} fps"

        seconds = float(_probe(video, "duration")[0])
        assert seconds > 2.0, f"{name}: only {seconds:.1f}s long"


def test_every_example_actually_draws_and_moves():
    """Catches the two ways a render fails while still producing a file: a
    black video, and a frozen one.

    It does not catch a video that moves *wrongly* — bars sliding to the wrong
    slot still change pixels. Only looking at it catches that, or golden
    frames, which are their own maintenance problem.
    """
    for main_py in EXAMPLES:
        name = main_py.parent.name
        frames = _frames(_video(main_py))
        assert len(frames) >= 3, f"{name}: could not sample frames"

        brightest = max(max(f) for f in frames)
        assert brightest > 40, f"{name}: every sampled frame is nearly black"

        # How many pixels changed, not by how much: a mean is dominated by the
        # black background, so an animation of small bright text scores almost
        # nothing however much it is saying.
        #
        # Measured against how much is *drawn*, not against the whole frame. An
        # example with one ball on a plain background moves everything it has
        # and still covers under 1% of the screen, which is indistinguishable
        # from frozen if you count pixels flatly — `moving_ball` and
        # `bouncing_ball` both score 0.94% that way, on either side of an
        # arbitrary line. As a share of the ink the spread across the examples
        # is 0.38 to 1.13, and a frozen video is still 0.
        moved = [
            sum(1 for a, b in zip(x, y) if abs(a - b) > 24) / len(x)
            for x, y in zip(frames, frames[1:])
        ]
        ink = max(sum(1 for p in f if p > 24) / len(f) for f in frames)
        assert ink > 0.0, f"{name}: nothing is drawn at all"

        stirred = max(moved) / ink
        assert stirred > 0.15, (
            f"{name}: nothing moves — at most {100 * stirred:.1f}% of what is "
            f"drawn changed between any two sampled seconds")


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
