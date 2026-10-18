"""Asking a film what is on it: at a time, at a point, and why."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from ..index import Index, as_index, beside


def load(video: "str | Path") -> Index:
    """The index written beside `video` by `render`: the compact `.index.bin` if
    there is one, else the readable `.index.json`. It is read lazily, so a film
    with an index of hundreds of megabytes as JSON opens at once."""
    return beside(video)


def build(script: "str | Path", args=()):
    """Run `script` as `python script args…` would, up to its render call,
    and hand back the Explanation it was about to render.

    The script's folder is importable, as it is under `python`, and whatever
    the script imported from that folder is forgotten afterwards — so running
    it again picks up an edit to `vocabulary.py` as well as to `main.py`.
    """
    import runpy

    import importlib

    # By module name: `codimate.explain` the attribute is the explain() function.
    explain = importlib.import_module(f"{__package__.rpartition('.')[0]}.explain")

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
    from ..layout import height, width
    return {"version": 1, "canvas": [width(), height()],
            "beats": explanation.index()}


def on_screen(index, beat: int) -> "list[dict]":
    """Every shape on screen once `beat` has arrived, topmost first.

    How it is found depends on the index: a film in memory is read off its
    scene, a compact file decodes one chunk, and the readable JSON replays
    differences from the nearest checkpoint (see :mod:`codimate.index`).
    """
    return _under(as_index(index).shapes_at(beat), None)


def at(index, seconds: float, point=None) -> dict:
    """What is on screen at `seconds`, and what put it there.

    Returns the beat — its event and the State behind it — and the shapes on
    screen, **topmost first**, the order the Engine draws in reversed. Give a
    `point` in canvas coordinates to keep only the shapes under it.

    Mid-beat, a shape is between where the last beat left it and where this
    one takes it. The nearer of the two is reported: good enough to identify
    what was clicked, and honest about being an approximation.
    """
    index = as_index(index)
    beats = index.beats
    if not beats:
        raise ValueError("the index has no beats")
    i = shown_beat = beat_at(index, seconds)
    beat = beats[i]
    if i > 0 and seconds < beat["at"] + beat["secs"] / 2:
        shown_beat = i - 1
    return {"seconds": round(seconds, 3), "beat": i,
            "event": beat["event"], "state": beat["state"],
            "shapes": _under(on_screen(index, shown_beat), point)}


def beat_at(index, seconds: float) -> int:
    """The beat `seconds` falls in: the last one to have started."""
    import bisect
    return max(bisect.bisect_right(as_index(index).starts(), seconds) - 1, 0)


def covered(index) -> "list[list]":
    """Every label something is drawn on, as `[seconds, above, below]`, from
    the index — so a rendered film answers it as well as a script does.
    Reported where each collision starts, at the moment its beat settles."""
    found, before = [], set()
    for beat in as_index(index).beats:
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
