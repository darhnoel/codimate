"""What one moment looks like: Groups, Scenes, and the flat shape payload."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any, Hashable

from .layout import Slot, _UNSET, _resolve, width


@dataclass(frozen=True)
class _Shape:
    """One shape in the payload. Flat by design (ADR 0008) — the Engine diffs
    it field by field, so every kind carries every field."""

    item: str
    kind: str
    x: float
    y: float
    x2: float = 0.0
    y2: float = 0.0
    w: float = 0.0
    h: float = 0.0
    r: float = 0.0
    color: str = "white"
    text: str = ""
    size: float = 16.0
    layer: int = 0
    opacity: float = 1.0


def _key(path) -> str:
    """Join a path of keys into the string the Engine pairs shapes by.

    Nested keys flatten, so a name built from other names stays readable:
    ``("edge", (0, 1), (1, 2))`` becomes ``"edge/0/1/1/2"``. That is what lets
    you write ``("edge", src, dst)`` instead of concatenating tuples by hand.
    """
    parts: "list[str]" = []

    def walk(key):
        if isinstance(key, (tuple, list)):
            for part in key:
                walk(part)
        else:
            parts.append(str(key))

    walk(path)
    return "/".join(parts)


def _point(place) -> "tuple[float, float]":
    """A place: a Slot (its centre) or a plain (x, y)."""
    if isinstance(place, Slot):
        return (place.x, place.y)
    return (float(place[0]), float(place[1]))


def _at(at, x, y, edges):
    """Resolve ``at=`` against the per-axis arguments.

    A position is one thing, so it should arrive as one value. Splitting it
    into ``x=`` and ``y=`` forces a caller holding a point to take it apart —
    and in practice that meant calling the function twice and indexing ``[0]``
    and ``[1]``.

    The per-axis arguments stay, because they are not the same question: they
    are for when only one axis is known, or when an edge is what you mean
    (``top=``, ``bottom=``). Mixing the two is always a mistake, so it is an
    error rather than a precedence rule.
    """
    if at is None:
        return x, y
    if any(value is not None for value in (x, y, *edges)):
        raise ValueError("give at=, or x=/y= and edges — not both")
    return _point(at)


class Group:
    """Somewhere to draw, with its own origin and its own name.

    Everything drawn on a Group moves as one thing, because the Engine sees
    each child's name as part of the group's name.

    **Inside a Group, ``0`` is the Group's anchor point.** ``bottom=0`` stands
    a shape on it; ``top=20`` puts one 20 below it.
    """

    def __init__(self, scene: "Scene", path: tuple, ox: float, oy: float, w: float):
        self._scene = scene
        self._path = path
        self._ox = ox
        self._oy = oy
        self._w = w

    # -- the origin a child measures from ------------------------------------

    @property
    def _child_default(self):
        """Inside a Group, a shape that says nothing sits at the Group's point.

        Not inherited from the parent's *anchor* — 0 always means this Group's
        own point, at any depth. That is what keeps nesting rule-free.
        """
        return 0.0 if self._path else _UNSET


    def _place(self, key, kind, *, x, y, x2=0.0, y2=0.0, w=0.0, h=0.0, r=0.0, **rest) -> "Group":
        path = self._path + (key,)
        name = _key(path)
        if name in self._scene._shapes:
            raise ValueError(
                f"two shapes share the name {name!r} — a name means exactly one thing"
            )
        self._scene._shapes[name] = _Shape(
            item=name, kind=kind,
            x=self._ox + x, y=self._oy + y,
            x2=self._ox + x2, y2=self._oy + y2,
            w=w, h=h, r=r, **rest
        )
        return self

    # -- shapes ---------------------------------------------------------------

    def rect(
        self,
        key: Hashable,
        *,
        at: "Slot | tuple[float, float] | None" = None,
        h: float,
        w: float = None,
        x: float = None,
        y: float = None,
        left: float = None,
        right: float = None,
        top: float = None,
        bottom: float = None,
        radius: float = 0.0,
        color: str = "white",
        layer: int = 0,
        opacity: float = 1.0,
    ) -> "Group":
        """A rectangle. Place it by its centre or by any edge.

        ``radius`` rounds the corners. It animates like anything else, so a
        rectangle can square off or soften as the explanation moves.
        """
        if w is None:
            w = _resolve(None, None, None, 0, ("w",), self._w if self._path else _UNSET)
        x, y = _at(at, x, y, (left, right, top, bottom,))
        return self._place(
            key,
            "rect",
            x=_resolve(x, left, right, w / 2, ("x", "left", "right"), self._child_default),
            y=_resolve(y, top, bottom, h / 2, ("y", "top", "bottom"), self._child_default),
            w=w,
            h=h,
            color=color,
            r=radius,
            layer=layer,
            opacity=opacity,
        )

    def circle(
        self,
        key: Hashable,
        *,
        at: "Slot | tuple[float, float] | None" = None,
        r: float,
        x: float = None,
        y: float = None,
        left: float = None,
        right: float = None,
        top: float = None,
        bottom: float = None,
        color: str = "white",
        layer: int = 0,
        opacity: float = 1.0,
    ) -> "Group":
        """A circle. Place it by its centre or by any edge, like a rect."""
        x, y = _at(at, x, y, (left, right, top, bottom,))
        return self._place(
            key,
            "circle",
            x=_resolve(x, left, right, r, ("x", "left", "right"), self._child_default),
            y=_resolve(y, top, bottom, r, ("y", "top", "bottom"), self._child_default),
            r=r,
            color=color,
            layer=layer,
            opacity=opacity,
        )

    def text(
        self,
        key: Hashable,
        content: Any,
        *,
        at: "Slot | tuple[float, float] | None" = None,
        x: float = None,
        y: float = None,
        top: float = None,
        bottom: float = None,
        size: float = 16.0,
        color: str = "white",
        layer: int = 10,
        opacity: float = 1.0,
    ) -> "Group":
        """Text, centred horizontally. You never deal with baselines."""
        x, y = _at(at, x, y, (top, bottom,))
        return self._place(
            key,
            "text",
            x=_resolve(x, None, None, 0.0, ("x",), self._child_default),
            y=_resolve(y, top, bottom, size / 2, ("y", "top", "bottom"), self._child_default),
            text=str(content),
            size=size,
            color=color,
            layer=layer,
            opacity=opacity,
        )

    def formula(
        self,
        key: Hashable,
        latex: str,
        *,
        at: "Slot | tuple[float, float] | None" = None,
        x: float = None,
        y: float = None,
        top: float = None,
        bottom: float = None,
        size: float = 16.0,
        reveal: float = 1.0,
        pen: float = 0.0,
        color: str = "white",
        layer: int = 10,
        opacity: float = 1.0,
    ) -> "Group":
        r"""Real mathematics, written as LaTeX.

            scene.formula("eq", r"\frac{QK^{T}}{\sqrt{d_k}}", size=34)

        Use a raw string, or every backslash needs doubling. ``size`` means
        what it means for :meth:`text`, so a formula and a label at the same
        size look the same weight.

        The result is glyph outlines, not a font — so it moves, fades and
        recolours like any other shape. It is typeset once when the video is
        built, never per frame.

        ``reveal`` is how much of it is showing, left to right: ``0.0`` is
        nothing, ``1.0`` is all of it. Animate it and the equation writes
        itself on, a term at a time::

            scene.formula("eq", EQUATION, reveal=1.0 if frame.is_("shown") else 0.0)

        ``pen`` draws it instead of fading it. Give it a stroke width and
        each glyph's outline is traced by a moving pen, then filled in behind
        it as the pen moves on. ``reveal`` still says how far the pen has got::

            scene.formula("eq", EQUATION, pen=2.0,
                          reveal=1.0 if frame.is_("shown") else 0.0)

        Needs the ``typst`` binary on PATH, the way video export needs
        ``ffmpeg``. You get a clear error naming the install if it is missing.
        """
        x, y = _at(at, x, y, (top, bottom,))
        return self._place(
            key,
            "formula",
            x=_resolve(x, None, None, 0.0, ("x",), self._child_default),
            y=_resolve(y, top, bottom, size / 2, ("y", "top", "bottom"), self._child_default),
            text=latex,
            size=size,
            r=reveal,
            w=pen,
            color=color,
            layer=layer,
            opacity=opacity,
        )

    def line(
        self,
        key: Hashable,
        *,
        start: "Slot | tuple[float, float]",
        end: "Slot | tuple[float, float]",
        w: float = 2.0,
        color: str = "white",
        layer: int = 0,
        opacity: float = 1.0,
    ) -> "Group":
        """A line between two points. ``w`` is its thickness.

        A line has no centre to anchor, so it takes its two ends directly.
        Each end may be a Slot — a line joins the middles of two places —
        or a plain ``(x, y)``:

            scene.line(key, start=at[src], end=at[dst])
        """
        (x, y), (x2, y2) = _point(start), _point(end)
        return self._place(
            key,
            "line",
            x=x,
            y=y,
            x2=x2,
            y2=y2,
            w=w,
            color=color,
            layer=layer,
            opacity=opacity,
        )

    # -- nesting --------------------------------------------------------------

    def group(
        self,
        key: Hashable,
        slot: "Slot | None" = None,
        *,
        anchor: "str | None" = None,
        x: float = None,
        y: float = None,
        left: float = None,
        right: float = None,
        top: float = None,
        bottom: float = None,
        w: float = None,
    ) -> "Group":
        """A place to draw a thing made of several shapes.

            bar = scene.group(item.id, slot)
            bar.rect("bar", h=item.value * 70, bottom=0)
            bar.text("label", item.value, top=20)

        Give it a Slot and it sits where the Slot says. Inside it, ``0`` is
        that point, and everything drawn on it moves as one.
        """
        if slot is not None:
            gx, gy = slot.point(anchor)
            gw = slot.w if w is None else w
        else:
            # A group with no Slot and no anchors sits exactly where its
            # parent does — a group that exists only to name things.
            gw = self._w if w is None else w
            gx = _resolve(x, left, right, gw / 2, ("x", "left", "right"), self._child_default)
            gy = _resolve(y, top, bottom, 0.0, ("y", "top", "bottom"), self._child_default)

        return Group(self._scene, self._path + (key,), self._ox + gx, self._oy + gy, gw)


class Scene(Group):
    """The picture at one moment. No animation, no timing, no memory of the
    frame before.

    Draw on it with the shapes it inherits from `Group` — `rect`, `circle`,
    `text`, `line`, `formula` — and `group` to put several of them together.

    Every shape carries a name — the identity of the thing you are talking
    about. **Motion is implied by identity:** the Engine pairs shapes by name
    between one moment and the next, and whatever changed becomes movement. A
    name that follows a *value* travels; a name that follows a *position*
    stays put and changes shape. Both are useful, and choosing wrong produces a
    confusing video, not an error.
    """

    def __init__(self) -> None:
        self._shapes: dict[str, _Shape] = {}
        super().__init__(self, (), 0.0, 0.0, width())

    def _payload(self) -> "list[dict[str, Any]]":
        return [asdict(s) for s in self._shapes.values()]
