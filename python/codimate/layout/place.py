"""Placing things: points, edges, Slots, and rows and columns of them."""

from __future__ import annotations

from dataclasses import dataclass
from .canvas import height, width


@dataclass(frozen=True)
class Place:
    """Where a shape goes, as one value.

    Built by :func:`at`. Exists so a shape takes one placement argument
    instead of six — the six were one idea wearing a disguise.
    """

    x: "float | None" = None
    y: "float | None" = None
    top: "float | None" = None
    bottom: "float | None" = None


def at(x=None, y=None, top=None, bottom=None) -> Place:
    """A place, for a shape's ``at=``.

        scene.rect("bar", h=40, at=cm.at(bottom=0))
        scene.text("l", "hi", at=cm.at(x=col, top=y + 22))

    Give one value per axis: ``x`` or nothing for horizontal, and ``y``,
    ``top`` or ``bottom`` for vertical. A plain ``(x, y)`` or a `Slot` works
    wherever a Place does, so you only need this for edges.
    """
    if y is not None and (top is not None or bottom is not None):
        raise ValueError("give one of y=, top= or bottom=")
    return Place(x=x, y=y, top=top, bottom=bottom)


@dataclass(frozen=True)
class Slot:
    """A place to put something. Not a shape — nothing draws a Slot.

    `row` and `column` hand you these; you rarely build one.

    - `x`, `y` — its centre
    - `w`, `h` — its size
    - `left`, `right`, `top`, `bottom` — its edges
    - `anchor` — which edge things placed here line up on
    """

    x: float
    y: float
    w: float
    h: float
    anchor: str = "center"

    @property
    def left(self) -> float:
        return self.x - self.w / 2

    @property
    def right(self) -> float:
        return self.x + self.w / 2

    @property
    def top(self) -> float:
        return self.y - self.h / 2

    @property
    def bottom(self) -> float:
        return self.y + self.h / 2

    def point(self, anchor: "str | None" = None) -> "tuple[float, float]":
        """The single point this Slot anchors things at."""
        anchor = anchor or self.anchor
        if anchor == "center":
            return (self.x, self.y)
        if anchor == "bottom":
            return (self.x, self.bottom)
        if anchor == "top":
            return (self.x, self.top)
        raise ValueError(f"unknown anchor {anchor!r} — use center, top or bottom")


def _within(box) -> "Slot":
    """The region a run of Slots divides up — the whole canvas by default."""
    if box is not None:
        return box
    return Slot(x=width() / 2, y=height() / 2, w=width(), h=height())


def _spread(items, gap, size, extent, centre):
    """Evenly space things of `size` along one axis, centred on `centre`.

    The shared half of `row` and `column`: normalise the input, pick a default
    size that fills most of `extent`, and lay out the positions.
    """
    sequence = list(range(items)) if isinstance(items, int) else list(items)
    count = len(sequence)
    if count == 0:
        return [], [], 0.0

    if size is None:
        # Fill 70% of the canvas by default, so a run always looks deliberate.
        size = max((extent * 0.7 - gap * (count - 1)) / count, 1.0)

    span = count * size + (count - 1) * gap
    first = centre - span / 2 + size / 2
    return sequence, [first + i * (size + gap) for i in range(count)], size


def _size(size):
    """A Slot size: one number for both sides, or `(w, h)`."""
    if size is None:
        return None, None
    if isinstance(size, (int, float)):
        return float(size), None
    return size


def row(
    items,
    *,
    gap: float = 40.0,
    size=None,
    at: "Place | None" = None,
    within: "Slot | None" = None,
):
    """One Slot per item, evenly spaced and centred on the canvas.

        for slot, item in cm.row(values, gap=40, size=190):
            ...

    ``size`` is one number for the Slot's width (its height follows), or
    ``(w, h)`` for both.

    A row lays things out on a shared baseline, so its Slots anchor at
    bottom-centre — hand one straight to ``scene.group()``.

    Pass ``within=slot`` to divide up part of the canvas instead of all of it,
    so a chart can live in its own corner without any arithmetic of yours.

    Yields ``(slot, item)`` pairs, or bare Slots if you passed a count.
    """
    box = _within(within)
    place = at or Place()
    w, h = _size(size)
    sequence, xs, w = _spread(items, gap, w, box.w, box.x)
    if h is None:
        h = w
    # A row sits on a baseline near the bottom of its box unless told otherwise.
    floor = box.bottom - box.h * 0.22 if place.bottom is None else place.bottom
    y = place.y if place.y is not None else floor - h / 2

    for x, item in zip(xs, sequence):
        slot = Slot(x=x, y=y, w=w, h=h, anchor="bottom")
        yield slot if isinstance(items, int) else (slot, item)


def column(
    items,
    *,
    gap: float = 40.0,
    size=None,
    at: "Place | None" = None,
    within: "Slot | None" = None,
):
    """One Slot per item, stacked vertically and centred on the canvas.

        for slot in cm.column(4, gap=40, at=cm.at(x=640)):
            ...

    A column stacks things around a centre line, so its Slots anchor at their
    centre. ``x`` places the column horizontally; it defaults to mid-canvas.

    Pass ``within=slot`` to stack inside part of the canvas instead of all
    of it.

    Yields ``(slot, item)`` pairs, or bare Slots if you passed a count.
    """
    box = _within(within)
    place = at or Place()
    w, h = _size(size)
    centre = box.y if place.y is None else place.y
    sequence, ys, h = _spread(items, gap, h, box.h, centre)
    if w is None:
        w = h
    x = box.x if place.x is None else place.x

    for cy, item in zip(ys, sequence):
        slot = Slot(x=x, y=cy, w=w, h=h, anchor="center")
        yield slot if isinstance(items, int) else (slot, item)


_UNSET = object()


def _resolve(given, half, names, default=_UNSET):
    """Turn whichever anchor was given into a centre coordinate."""
    centre, low, high = given
    given = [v for v in given if v is not None]
    if len(given) > 1:
        raise ValueError(f"give only one of {', '.join(f'{n}=' for n in names)}")
    if not given:
        if default is _UNSET:
            raise ValueError(f"give one of {', '.join(f'{n}=' for n in names)}")
        return float(default)
    if centre is not None:
        return float(centre)
    if low is not None:
        return float(low) + half
    return float(high) - half
