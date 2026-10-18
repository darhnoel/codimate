"""Axes: a graph's frame, its ticks, and the map from data to pixels."""

from __future__ import annotations

from dataclasses import dataclass
from .canvas import measure
from .place import Place, Slot, _resolve, _size, _within, at


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
                # A y label ends a fixed gap before its tick, however long it
                # is — centred at a fixed x, "1000" ran into the tick.
                where = at(x=x, top=y + 10) if along else \
                    at(x=x - 10 - measure(words, self.label)[0] / 2, y=y)
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
