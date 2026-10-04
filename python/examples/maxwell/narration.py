"""The narration, if there is any, and the caption that carries it.

`speak.py` records each caption and works out when every word is said. Without its
files, or with `--silent`, the film runs at reading pace and says nothing: the voice
is an addition, never a dependency.
"""

import json
import sys
from pathlib import Path

import lines  # Khmer, word boundaries marked

# The caption lives in `year` for now (ADR 0019 leaves it out of the kit).
sys.path.append(str(Path(__file__).resolve().parent.parent / "year"))
import caption  # noqa: E402

AUDIO = Path(__file__).resolve().parent / "audio"
SILENT = "--silent" in sys.argv
LEAD, TAIL = 0.4, 1.4  # the caption's start, and a rest after the last word
LAY_CAPTION = 900


def _load(name):
    path = AUDIO / name
    return json.loads(path.read_text()) if path.exists() and not SILENT else None


VOICE = {entry["text"]: entry for entry in _load("narration.json") or []}
# per caption: when each word is said, counted from the start of its own clip
WHEN = _load("timing.json") or {}


def clip(key):
    """The recording of this scene's line, if there is one."""
    return VOICE.get(lines.SPEAK[key])


def reading(key):
    """Seconds each word of the line holds the mark, as the reader needs them."""
    return [caption.pace(piece) for piece, _ in caption.chunks(lines.SAY[key])]


def starts(key):
    """When each word of the line begins, in seconds from the scene's start.

    From the voice if the clip has been heard word by word: the mark is then on
    the word being said. Otherwise the reading pace, stretched to the clip's
    length if there is a clip, or as it stands if there is none."""
    heard, when = clip(key), WHEN.get(lines.SPEAK[key])
    if heard and when and len(when) == len(reading(key)):
        return [LEAD + moment for moment in when]
    steps = reading(key)
    if heard:
        steps = [step * heard["seconds"] / sum(steps) for step in steps]
    out, at = [], LEAD
    for step in steps:
        out.append(at)
        at += step
    return out


def said(key, t):
    """Which word is being said at `t` (1-based); 0 before the first, and the
    last stays lit once the line is done."""
    return sum(start <= t for start in starts(key))


def seconds(key):
    """How long the line takes to say: the recording's length, or the reading pace."""
    heard = clip(key)
    return heard["seconds"] if heard else sum(reading(key))


def draw(scene, key, count, y):
    """The caption for scene line `key`, the mark on word number `count`."""
    caption.draw(scene, lines.SAY.get(key, ""), count, y=y, layer=LAY_CAPTION)
