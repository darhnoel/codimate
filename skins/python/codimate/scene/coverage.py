"""Where a shape sits, and which labels something else has drawn over
(ADR 0017). Both worked out by the Authoring Core, so a film's check is the
same in every language."""

from __future__ import annotations

from dataclasses import asdict
from typing import TYPE_CHECKING

from .._waist import _film_font, call

if TYPE_CHECKING:
    from .root import Scene


def box(shape) -> "tuple[float, float, float, float] | None":
    """Where a shape sits: `(left, top, width, height)`, or `None` for a
    formula on a machine without `typst`."""
    found = call("bounds", {"shape": asdict(shape), "film_font": _film_font()})
    return None if found is None else tuple(found)


def covered(scene: "Scene") -> "list[tuple[str, str]]":
    """Every label something is drawn on, as `(above, below)` pairs: nothing
    visible may be drawn above a text or a formula, at any opacity, and no two
    of them may overlap."""
    return [tuple(pair) for pair in call("covered_scene", {
        "scene": scene._payload(), "camera": scene._camera(),
        "film_font": _film_font()})]
