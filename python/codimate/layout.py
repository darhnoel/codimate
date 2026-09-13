"""Where things sit: the canvas, Slots, and the helpers that divide it up."""

from __future__ import annotations

from dataclasses import dataclass


# ponytail: module-level so a view function can lay out without being handed a
# canvas. `render()` reads the same values, so there is one source of truth.
_CANVAS = [1280.0, 720.0]


def measure(text: str, size: float = 16.0) -> tuple[float, float]:
    """How wide and tall ``text`` will be at ``size``: ``(w, h)``.

    For drawing a box around a label without guessing::

        w, h = cm.measure(label, size=30)
        scene.rect("box", x=x, y=y, w=w + 24, h=h + 12, radius=6)
        scene.text("label", label, x=x, y=y, size=30)

    Measured by the engine with the real fonts, including fallback, so it is
    right for Khmer and anything else that is not plain ASCII — which is why
    estimating ``len(text) * size * k`` is not good enough.

    The height is the line height, the same for "cat" and "Qgy", so a row of
    boxes lines up instead of jittering with whatever letters it holds.
    """
    from . import _codimate  # imported here so the pure Python is testable

    return _codimate.measure(str(text), float(size))


def measure_math(latex: str, size: float = 16.0) -> tuple[float, float]:
    """How wide and tall a LaTeX formula will be at ``size``: ``(w, h)``.

    The counterpart of :func:`measure`, so a formula can be laid out beside
    words — a caption that mixes prose and mathematics needs both.
    """
    from . import _codimate

    return _codimate.measure_formula(str(latex), float(size))


def canvas(w: float, h: float) -> None:
    """Set the size of the video. Defaults to 1280x720."""
    _CANVAS[:] = [float(w), float(h)]


def width() -> float:
    """The canvas width. ``cm.width() / 2`` is the horizontal centre."""
    return _CANVAS[0]


def height() -> float:
    """The canvas height."""
    return _CANVAS[1]


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


def row(
    items,
    *,
    gap: float = 40.0,
    w: "float | None" = None,
    h: "float | None" = None,
    bottom: "float | None" = None,
    y: "float | None" = None,
    within: "Slot | None" = None,
):
    """One Slot per item, evenly spaced and centred on the canvas.

        for slot, item in cm.row(values, gap=40):
            ...

    A row lays things out on a shared baseline, so its Slots anchor at
    bottom-centre — hand one straight to ``scene.group()``.

    Pass ``within=slot`` to divide up part of the canvas instead of all of it,
    so a chart can live in its own corner without any arithmetic of yours.

    Yields ``(slot, item)`` pairs, or bare Slots if you passed a count.
    """
    box = _within(within)
    sequence, xs, w = _spread(items, gap, w, box.w, box.x)
    if h is None:
        h = w
    if y is None:
        y = (box.bottom - box.h * 0.22 if bottom is None else bottom) - h / 2

    for x, item in zip(xs, sequence):
        slot = Slot(x=x, y=y, w=w, h=h, anchor="bottom")
        yield slot if isinstance(items, int) else (slot, item)


def column(
    items,
    *,
    gap: float = 40.0,
    w: "float | None" = None,
    h: "float | None" = None,
    x: "float | None" = None,
    y: "float | None" = None,
    within: "Slot | None" = None,
):
    """One Slot per item, stacked vertically and centred on the canvas.

        for slot in cm.column(4, gap=40, x=640):
            ...

    A column stacks things around a centre line, so its Slots anchor at their
    centre. ``x`` places the column horizontally; it defaults to mid-canvas.

    Pass ``within=slot`` to stack inside part of the canvas instead of all
    of it.

    Yields ``(slot, item)`` pairs, or bare Slots if you passed a count.
    """
    box = _within(within)
    sequence, ys, h = _spread(items, gap, h, box.h, box.y if y is None else y)
    if w is None:
        w = h
    if x is None:
        x = box.x

    for cy, item in zip(ys, sequence):
        slot = Slot(x=x, y=cy, w=w, h=h, anchor="center")
        yield slot if isinstance(items, int) else (slot, item)


_UNSET = object()


def _resolve(centre, low, high, half, names, default=_UNSET):
    """Turn whichever anchor was given into a centre coordinate."""
    given = [v for v in (centre, low, high) if v is not None]
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
