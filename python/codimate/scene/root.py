"""Scene: the root Group, the picture at one moment."""

from __future__ import annotations

from dataclasses import asdict
from typing import Any

from ..layout import width
from .._surface import _SceneSurface
from .group import Group
from .marker import Stroke
from .shape import _Shape


class Scene(_SceneSurface, Group):
    """The picture at one moment. No animation, no timing, no memory of the
    frame before.

    Draw on it with the shapes it inherits from `Group`: `Group.rect`,
    `Group.circle`, `Group.text`, `Group.line`, and `Group.formula`.
    Use `Group.group` to put several shapes together.

    Every shape carries a name — the identity of the thing you are talking
    about. **Motion is implied by identity:** the Engine pairs shapes by name
    between one moment and the next, and whatever changed becomes movement. A
    name that follows a *value* travels; a name that follows a *position*
    stays put and changes shape. Both are useful, and choosing wrong produces a
    confusing video, not an error.
    """

    def __init__(self) -> None:
        self._shapes: dict[str, _Shape] = {}
        self._markers: dict[str, Stroke] = {}
        self._overlays: set = set()
        self._focus: "dict[str, Any] | None" = None
        super().__init__(self, (), 0.0, 0.0, width())

    def _aim(self, aim: dict) -> None:
        """Keep where the camera looks (the Core worked it out)."""
        self._focus = dict(aim, fixed=sorted(self._overlays))

    def _open_overlay(self, frame: dict) -> "Group":
        """The overlay the Core placed, remembered so the camera leaves it."""
        self._overlays.add(frame["path"][-1])
        return self._nest(frame)

    def _payload(self) -> "list[dict[str, Any]]":
        return [asdict(s) for s in self._shapes.values()]

    def _camera(self) -> "dict[str, Any] | None":
        # Recomputed here rather than in `focus`, because an overlay may be
        # created after the camera is aimed.
        if self._focus is not None:
            self._focus["fixed"] = sorted(self._overlays)
        return self._focus
