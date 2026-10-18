"""Scene: the root Group, the picture at one moment."""

from __future__ import annotations

from dataclasses import asdict
from typing import Any, Hashable

from ..layout import width
from .group import Group
from .marker import Stroke
from .shape import _Shape, _key


class Scene(Group):
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

    def focus(self, *names: Hashable, pad: float = 40.0, least: float = 240.0) -> None:
        """Look at these things. The camera works out where to stand.

            scene.focus("cat")                    # frame this shape
            scene.focus(("cell", 2, 1), pad=60)   # any name, with room around it
            scene.focus("The", "cat", "sat")      # several — all of them in shot
            scene.focus()                         # the whole canvas again

        You never write a camera coordinate. The Engine knows where everything
        is and how big it is, so it derives the framing — and because each
        moment is framed on its own, a camera that moves between two moments is
        just two sets of coordinates that differ, which the usual tween
        animates. Camera movement needs nothing new.

        Naming rather than positioning also means the shot survives a layout
        change: move the shape and the camera follows, because the name is the
        part that is stable.

        ``least`` is the smallest thing the camera will fill the frame with, so
        focusing on a full stop does not magnify it into abstraction.

        Anything on an :meth:`overlay` stays where it is put.
        """
        self._focus = {
            "names": [_key(n) for n in names],
            "pad": float(pad),
            "min_size": float(least),
            "fixed": sorted(self._overlays),
        }

    def overlay(self) -> "Group":
        """A place for things the camera does not move.

            hud = scene.overlay()
            hud.text("title", "Scaled dot-product attention", x=640, y=52)

        The diagram pans and zooms underneath; whatever is drawn here stays
        put. Titles and narration belong here — the moment the picture zooms, a
        caption that zooms with it is wrong, and usually off the edge of the
        frame entirely.

        Content is in the world unless it says otherwise, because in an
        explanation most of the frame is diagram and only a little is
        narration.
        """
        # At the canvas origin, so its children are placed in plain screen
        # coordinates rather than relative to somewhere.
        group = self.group("_overlay", at=(0.0, 0.0))
        self._overlays.add("_overlay")
        return group

    def _payload(self) -> "list[dict[str, Any]]":
        return [asdict(s) for s in self._shapes.values()]

    def _camera(self) -> "dict[str, Any] | None":
        # Recomputed here rather than in `focus`, because an overlay may be
        # created after the camera is aimed.
        if self._focus is not None:
            self._focus["fixed"] = sorted(self._overlays)
        return self._focus
