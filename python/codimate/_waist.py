"""The Python Skin's crossing of the Waist (ADR 0032): send one operation and
its data to the Authoring Core, get the answer back.

Also the converters the generated methods (`_surface.py`) use to turn Python
values into the plain data every Skin sends: the same shapes of data, whatever
the language.
"""

from __future__ import annotations

import json


_ENGINE = []  # the extension, imported on first use so the pure Python is testable


def call(op: str, data: dict):
    """Run `op` in the Authoring Core. A refusal is a ValueError, raised here,
    on the line of the author's call."""
    if not _ENGINE:
        from . import _codimate

        _ENGINE.append(_codimate.call)
    answer = json.loads(_ENGINE[0](op, json.dumps(data)))
    if "error" in answer:
        raise ValueError(answer["error"])
    return answer["ok"]


def _key(key) -> "list[str]":
    """A name or a path of names, flattened: ``("edge", (0, 1))`` is
    ``["edge", "0", "1"]``."""
    parts: "list[str]" = []

    def walk(part):
        if isinstance(part, (tuple, list)):
            for inner in part:
                walk(inner)
        else:
            parts.append(str(part))

    walk(key)
    return parts


def _number(value):
    return None if value is None else float(value)


def _text(value) -> str:
    return str(value)


def _string(value):
    return None if value is None else str(value)


def _at(at):
    """A placement as data: nothing, a point ``[x, y]`` (a Slot is its centre),
    or a Place ``{x, y, top, bottom}``."""
    from .layout import Place, Slot

    if at is None:
        return None
    if isinstance(at, Place):
        return {"x": at.x, "y": at.y, "top": at.top, "bottom": at.bottom}
    if isinstance(at, Slot):
        return [at.x, at.y]
    return [float(at[0]), float(at[1])]


def _film_font() -> str:
    from .layout import _FONT

    return _FONT[0]
