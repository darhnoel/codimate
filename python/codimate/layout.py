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


def ngon(sides: int, r: float, at=(0.0, 0.0), turn: float = 0.0) -> list:
    """The corners of a regular polygon, for :meth:`Scene.polygon`.

        scene.polygon("tri", cm.ngon(3, r=60, at=(640, 360)))

    A triangle is three sides, a hexagon six. ``turn`` rotates it in degrees —
    the first corner otherwise points straight up.

    Returns points rather than drawing, so it composes: you can shift them,
    hand them to `polygon`, or measure them yourself.
    """
    import math

    if sides < 3:
        raise ValueError(f"a polygon needs at least 3 sides, got {sides}")
    step = 2 * math.pi / sides
    start = math.radians(turn) - math.pi / 2
    return [
        (at[0] + r * math.cos(start + step * i), at[1] + r * math.sin(start + step * i))
        for i in range(sides)
    ]


def star(points: int, r: float, inner: float = None, at=(0.0, 0.0),
         turn: float = 0.0) -> list:
    """The corners of a star, for :meth:`Scene.polygon`.

        scene.polygon("s", cm.star(5, r=80, at=(640, 360)), color="yellow")

    ``inner`` is the radius of the valleys; it defaults to a proportion that
    looks like a star rather than a gear.
    """
    import math

    inner = r * 0.42 if inner is None else inner
    step = math.pi / points
    start = math.radians(turn) - math.pi / 2
    return [
        (
            at[0] + (r if i % 2 == 0 else inner) * math.cos(start + step * i),
            at[1] + (r if i % 2 == 0 else inner) * math.sin(start + step * i),
        )
        for i in range(points * 2)
    ]


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


# --------------------------------------------------------------------------
# Axes: a coordinate map, not a drawing (ADR 0016)
# --------------------------------------------------------------------------


def _nice_step(span: float, about: int) -> float:
    """A round step near ``span / about``, from the 1-2-5 sequence.

    Round steps are not a nicety. A tick is a named shape, so the set of them
    is an identity that survives from one Scene to the next; a step that
    drifted with every frame would rename every tick every frame, and a still
    picture would flicker. 1-2-5 holds steady until the range really changes.
    """
    import math

    if span <= 0:
        raise ValueError(f"a range needs width, got {span}")
    raw = span / max(int(about), 1)
    power = 10.0 ** math.floor(math.log10(raw))
    for nice in (1.0, 2.0, 5.0):
        if raw <= nice * power:
            return nice * power
    return 10.0 * power


def _places(step: float) -> int:
    """How many decimals a label needs so that two ticks never read alike."""
    import math

    return max(0, -math.floor(math.log10(step) + 1e-9))


@dataclass(frozen=True)
class Axes:
    """A map from your numbers to pixels. Built by :func:`axes`.

    It draws nothing you did not ask it to and owns none of your names — see
    :meth:`at`. Its own shapes (frame, ticks, labels) are drawn by
    :meth:`draw`, under names you choose the prefix of.
    """

    x0: float
    x1: float
    y0: float
    y1: float
    left: float
    top: float
    w: float
    h: float
    ink: str = "#8b96a8"
    label: float = 20.0
    layer: int = 0

    def looks(self, ink: str = None, label: float = None,
              layer: int = None) -> "Axes":
        """Restyle the frame, ticks and labels. Returns a new Axes.

            plot = cm.axes(x=(0, 10), y=(0, 5)).looks(ink="#334", label=16)

        A separate call rather than more arguments on :meth:`draw`, which the
        shapes made the same choice about: no call in this library takes more
        than five things.
        """
        import dataclasses

        return dataclasses.replace(
            self,
            ink=self.ink if ink is None else str(ink),
            label=self.label if label is None else float(label),
            layer=self.layer if layer is None else int(layer))

    @property
    def right(self) -> float:
        return self.left + self.w

    @property
    def bottom(self) -> float:
        return self.top + self.h

    def at(self, x: float, y: float) -> tuple[float, float]:
        """One data point as a pixel pair, for any shape's ``at=``.

            scene.circle("dot", r=8, at=plot.at(2.0, 4.0))

        Handing back pixels rather than drawing is the whole design: what you
        do with them is an ordinary shape with a name of yours, so it tweens,
        `focus` frames it, and a motion Rule can be aimed at it.
        """
        across = (float(x) - self.x0) / (self.x1 - self.x0)
        up = (float(y) - self.y0) / (self.y1 - self.y0)
        return (self.left + across * self.w, self.bottom - up * self.h)

    def line(self, f, steps: int = 200, over=None) -> list:
        """``steps + 1`` points along ``y = f(x)``, as pixels.

            scene.curve("f", plot.line(lambda x: x * x), w=4).fill("red")

        ``over=(lo, hi)`` samples part of the range instead of all of it.

        An open `curve` is *drawn* rather than filled, so its colour is
        ``fill()`` and its thickness is ``w=`` — ``fill("none", edge=...)``
        draws nothing at all.

        Every sample is returned, including any that fall outside the box.
        Dropping them would be prettier and is wrong: two curves with different
        point counts do not interpolate (ADR 0010), so a curve that shed a
        point as it left the frame would stop animating. Keep the count and
        clamp the range with ``over=`` if you need it inside.
        """
        lo, hi = (self.x0, self.x1) if over is None else (float(over[0]),
                                                          float(over[1]))
        steps = max(int(steps), 1)
        return [self.at(x, f(x))
                for x in (lo + (hi - lo) * i / steps for i in range(steps + 1))]

    def ticks(self, axis: str = "x", about: int = 6) -> list:
        """``(n, value, label)`` per tick, where ``n`` is the step multiple.

        ``n`` is what a tick is named after, not the value. A float that drifts
        by one part in a billion is a different name, and a renamed shape
        leaves and re-enters — which is a fade, on a picture that did not move.
        An integer count of steps cannot drift.
        """
        import math

        lo, hi = (self.x0, self.x1) if axis == "x" else (self.y0, self.y1)
        step = _nice_step(hi - lo, about)
        digits = _places(step)
        first = math.ceil(lo / step - 1e-9)
        last = math.floor(hi / step + 1e-9)
        out = []
        for n in range(int(first), int(last) + 1):
            value = n * step
            label = f"{value:.{digits}f}"
            out.append((n, value, "0" if label.lstrip("-").strip("0.") == ""
                        else label))
        return out

    def draw(self, scene, name="plot", *, about: int = 6,
             grid: bool = False) -> "Axes":
        """Draw the frame, ticks and labels. Returns the Axes, for chaining.

            plot = cm.axes(x=(-4, 4), y=(-2, 6)).draw(scene)

        Every shape is named ``(name, ...)``, so two plots on one canvas do not
        collide and you can restyle or omit any of them by drawing your own.
        Colours and sizes come from :meth:`looks`.
        """
        ink, layer = self.ink, self.layer
        zero_y = self.y0 <= 0.0 <= self.y1
        zero_x = self.x0 <= 0.0 <= self.x1
        base = self.at(self.x0, 0.0)[1] if zero_y else self.bottom
        spine = self.at(0.0, self.y0)[0] if zero_x else self.left

        scene.line((name, "x-axis"), start=(self.left, base),
                   end=(self.right, base), w=1.6).fill(ink).on(layer=layer)
        scene.line((name, "y-axis"), start=(spine, self.top),
                   end=(spine, self.bottom), w=1.6).fill(ink).on(layer=layer)

        for axis, along in (("x", True), ("y", False)):
            for n, value, words in self.ticks(axis, about):
                if n == 0 and (zero_x if along else zero_y):
                    continue            # the other axis already draws through it
                x, y = (self.at(value, 0.0)[0], base) if along else \
                    (spine, self.at(0.0, value)[1])
                if grid:
                    ends = ((x, self.top), (x, self.bottom)) if along else \
                        ((self.left, y), (self.right, y))
                    scene.line((name, "grid", axis, n), start=ends[0],
                               end=ends[1], w=1.0).fill(ink) \
                         .on(layer=layer - 1, opacity=0.18)
                mark = ((x, y - 5), (x, y + 5)) if along else \
                    ((x - 5, y), (x + 5, y))
                scene.line((name, "tick", axis, n), start=mark[0], end=mark[1],
                           w=1.6).fill(ink).on(layer=layer)
                where = at(x=x, top=y + 10) if along else at(x=x - 18, y=y)
                scene.text((name, "label", axis, n), words, size=self.label,
                           at=where).fill(ink).on(layer=layer)
        return self


def axes(*, x, y, at=None, size=None, within: "Slot | None" = None) -> Axes:
    """A coordinate map from your numbers to pixels.

        plot = cm.axes(x=(-4, 4), y=(-2, 6), size=(760, 420)).draw(scene)
        scene.curve("f", plot.line(lambda t: t * t), w=4).fill("orange")
        scene.circle("dot", r=8, at=plot.at(t, t * t))

    ``x`` and ``y`` are ``(low, high)``. ``size`` is ``(w, h)`` in pixels, or
    one number for a square; it defaults to most of the canvas. ``at`` places
    the box's centre, and ``within=slot`` fits it to part of the canvas.

    It returns points rather than drawing, so what you plot is a shape with a
    name of yours — which is what lets it tween, be framed by `focus`, and be
    aimed at by a motion Rule. See ADR 0016.
    """
    box = _within(within)
    w, h = _size(size)
    if w is None:
        w = box.w * 0.72
    if h is None:
        h = box.h * 0.66
    place = at or Place()
    cx = box.x if place.x is None else place.x
    cy = _resolve((place.y, place.top, place.bottom), h / 2.0,
                  ("y", "top", "bottom"), box.y)
    x0, x1 = (float(v) for v in x)
    y0, y1 = (float(v) for v in y)
    if x0 == x1 or y0 == y1:
        raise ValueError(f"a range needs width, got x={x!r} y={y!r}")
    return Axes(x0=x0, x1=x1, y0=y0, y1=y1,
                left=cx - w / 2.0, top=cy - h / 2.0, w=float(w), h=float(h))
