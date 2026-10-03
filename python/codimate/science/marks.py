"""Marks for explaining something physical — the flat, 2D half of the kit.

A label on a plate, a glowing particle, a bracket on a measured span, an arrow
whose length is a force, the curve a wave makes. Every science sketch wrote
these again, a little differently each time; this is the version that survived
being fixed.

They all follow `cm.axes` (ADR 0016). Each one works out pixels and hands them
back. Where one draws, it draws ordinary shapes under a name *you* chose the
prefix of, so they tween, `focus` can frame them, and a motion Rule can be
aimed at them.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, replace

from ..layout import Place, Slot, _resolve, measure, measure_math

INK, DIM = "#e8eef7", "#93a0b2"
PLATE = "#0f1520"
PAD = 10.0

SIDES = ("top", "bottom", "left", "right")


def _name(name, *parts):
    """`name` extended by `parts`, so every shape of one mark shares a prefix.

    A name that is already a tuple is extended rather than nested, so
    ``("horizon", "label")`` and ``"horizon"`` both give shapes you can find.
    """
    base = name if isinstance(name, tuple) else (name,)
    return (*base, *parts)


def _centre(at, half: float) -> "tuple[float, float]":
    """A centre from ``at`` — a point, a Slot or `cm.at(...)` — for a box
    whose half-height is ``half``.

    Resolved once, here, for the whole mark. Handing the same ``top=`` to a
    plate and to the text on it resolves to two different centres, because
    the plate is taller than the text by its padding; the text then sits high
    with a gap under it.
    """
    if isinstance(at, Place):
        place = at
    elif isinstance(at, Slot):
        place = Place(x=at.x, y=at.y)
    else:
        place = Place(x=float(at[0]), y=float(at[1]))
    return (_resolve((place.x, None, None), 0.0, ("x",)),
            _resolve((place.y, place.top, place.bottom), half,
                     ("y", "top", "bottom")))


# --------------------------------------------------------------------------
# Tag: a label on its own plate
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class Tag:
    """A label on a small plate, with its box already worked out. Built by
    :func:`tag`.

    The box is measured from the real content plus real padding, never a
    guessed size. It is a map in the sense of `cm.axes`: ``edge()`` and
    ``left``/``right``/``top``/``bottom`` give pixels, so a leader line can be
    aimed at the plate's actual edge instead of a point guessed to be near it.
    """

    content: str
    x: float
    y: float
    w: float
    h: float
    color: str = INK
    size: float = 20.0
    formula: bool = True

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

    def edge(self, side: str) -> "tuple[float, float]":
        """The middle of one edge: ``top``, ``bottom``, ``left`` or ``right``."""
        points = {"top": (self.x, self.top), "bottom": (self.x, self.bottom),
                  "left": (self.left, self.y), "right": (self.right, self.y)}
        if side not in points:
            raise ValueError(f"unknown edge {side!r} — use {', '.join(SIDES)}")
        return points[side]

    def draw(self, scene, name, layer: int = 3) -> "Tag":
        """Draw the plate and its content. Returns the Tag, for chaining.

        Shapes are named ``(name, "plate")`` and ``(name, "text")``. The plate
        sits on ``layer`` and the content one above it.
        """
        scene.rect(_name(name, "plate"), w=self.w, h=self.h, at=(self.x, self.y)) \
            .fill(PLATE, edge=self.color, edge_w=1).round(6) \
            .on(layer=layer, opacity=0.92)
        if self.formula:
            body = scene.formula(_name(name, "text"), self.content,
                                 size=self.size, at=(self.x, self.y))
        else:
            body = scene.text(_name(name, "text"), self.content,
                              size=self.size, at=(self.x, self.y))
        body.fill(self.color).on(layer=layer + 1)
        return self

    def clear_of(self, *others: "Tag", gap: float = 6.0) -> "Tag":
        """This tag, moved the shortest way out of the others' boxes.

            name = science.tag("Moon", at=above_the_moon).clear_of(earth_name)

        For labels that follow bodies. Two bodies line up sooner or later and
        their labels land on each other, and no fixed offset prevents it,
        because the bodies move. A tag that is already clear does not move at
        all; one that is not is pushed straight out from the other's centre to
        the edge of its box plus ``gap``.

        It is a pure function of where the tags are, so a label slides round
        another rather than jumping — the push grows from nothing as they meet.
        It is gentle unless the two pass nearly dead centre: then it hurries
        round, by about (half the heights + gap) over how far off-centre the
        pass is, and only when two centres coincide exactly is it not
        continuous at all (there it goes up). Earlier ``others`` win: pass them
        in the order of who should stay put.
        """
        x, y = self.x, self.y
        for _ in range(2):          # clearing one can crowd another
            for other in others:
                half_w = (self.w + other.w) / 2 + gap
                half_h = (self.h + other.h) / 2 + gap
                dx, dy = x - other.x, y - other.y
                if abs(dx) >= half_w or abs(dy) >= half_h:
                    continue
                if dx == 0 and dy == 0:
                    dy = -1.0
                length = math.hypot(dx, dy)
                ux, uy = dx / length, dy / length
                reach = min(half_w / abs(ux) if ux else math.inf,
                            half_h / abs(uy) if uy else math.inf)
                x, y = other.x + ux * reach, other.y + uy * reach
        return replace(self, x=x, y=y)

    def leader(self, scene, name, start, side: str, layer: int = 2) -> None:
        """A line from ``start`` to the middle of one of this plate's edges.

            box = science.tag(r"\\text{event horizon}", at=(900, 120)).draw(scene, "h")
            box.leader(scene, "h-line", start=(780, 300), side="left")

        Aimed at the edge itself. A leader pointed at a spot guessed to be
        near the plate stops short of it, or runs through whatever is beside
        it.
        """
        scene.line(name, start=start, end=self.edge(side), w=1.5) \
            .fill(self.color).on(layer=layer, opacity=0.7)


def tag(content: str, *, at, color: str = INK, size: float = 20.0,
        formula: bool = True) -> Tag:
    """A label on a plate, sized to its content.

        box = science.tag(r"\\rho = 1000\\ \\text{kg/m}^3", at=cm.at(x=640, top=40))
        box.draw(scene, "density")

    ``content`` is LaTeX, which is what a physical quantity usually is, or
    plain words with ``formula=False`` — use that for Khmer or any prose.
    ``at`` is a point, a Slot or `cm.at(...)`; with ``top=`` or ``bottom=`` the
    *plate's* edge lands there, not the text's.
    """
    wide, high = (measure_math if formula else measure)(content, size)
    w, h = wide + PAD * 2, high + PAD * 1.6
    x, y = _centre(at, h / 2)
    return Tag(content=content, x=x, y=y, w=w, h=h, color=color, size=size,
               formula=formula)


# --------------------------------------------------------------------------
# Glow
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class Glow:
    """A soft halo, made the only way this renderer can: a stack of
    progressively wider, fainter copies under the sharp shape.

    It works on **one shape at a time** — a dot, a ring, a spring coil. Laid
    over many thin lines (a grid, a mesh) the stacked copies wash into mud or
    bands, and no step count fixes that. Use fewer, bolder things.

    ``steps`` is smoothness: two steps read as a target, six to eight as a
    glow. ``strength`` scales the whole effect, so a fading star is
    ``replace(glow, strength=0.5)``.
    """

    color: str = "#ffd23f"
    steps: int = 8
    spread: float = 1.8
    strength: float = 1.0
    rim: str = "#0b0f16"

    def dot(self, scene, name, at, r: float, layer: int = 0) -> None:
        """A glowing disc of radius ``r``: halos on ``layer``, core two above."""
        for i in range(self.steps, 0, -1):
            t = i / self.steps
            scene.circle(_name(name, "halo", i), r=r * (1 + self.spread * t), at=at) \
                .fill(self.color) \
                .on(layer=layer, opacity=(0.05 + 0.10 * (1 - t)) * self.strength)
        scene.circle(_name(name, "core"), r=r, at=at) \
            .fill(self.color, edge=self.rim, edge_w=1.5).on(layer=layer + 2)

    def ring(self, scene, name, at, r: float, layer: int = 0) -> None:
        """A glowing rim at radius ``r`` and nothing inside it — a horizon, a
        threshold, a shell. What fills the middle is yours to draw."""
        reach = 0.19 * r * self.spread / 1.8
        for i in range(self.steps, 0, -1):
            t = i / self.steps
            scene.circle(_name(name, "halo", i), r=r + reach * t, at=at) \
                .fill("none", edge=self.color, edge_w=max(0.5, 0.45 * reach * t)) \
                .on(layer=layer, opacity=(0.03 + 0.05 * (1 - t)) * self.strength)
        scene.circle(_name(name, "rim"), r=r, at=at) \
            .fill("none", edge=self.color, edge_w=2.5) \
            .on(layer=layer + 1, opacity=min(1.0, 0.4 + 0.6 * self.strength))


# --------------------------------------------------------------------------
# Wave
# --------------------------------------------------------------------------


def wave(start, end, *, cycles: float = 2.5, amp: float = 8.0,
         steps: int = 20) -> list:
    """Points along a wave from ``start`` to ``end``, for `curve`.

        scene.curve("photon", science.wave((100, 300), (500, 300)), w=2).fill("gold")

    A sine riding the straight line between the two points, ``cycles`` full
    turns of it, ``amp`` pixels either way. The ends land exactly on the
    line when ``cycles`` is a whole or half number — a quarter-cycle leaves the
    last point off to one side.

    Returns points, like `cm.ngon`, so you can shift them or hand them to a
    `polygon`. The point count is fixed by ``steps``, which is what lets one
    wave tween into another (ADR 0010).
    """
    dx, dy = end[0] - start[0], end[1] - start[1]
    length = math.hypot(dx, dy)
    if length == 0:
        raise ValueError("a wave needs somewhere to go: start and end are the same")
    px, py = -dy / length, dx / length
    steps = max(int(steps), 2)
    points = []
    for i in range(steps + 1):
        t = i / steps
        off = amp * math.sin(2 * math.pi * cycles * t)
        points.append((start[0] + dx * t + px * off, start[1] + dy * t + py * off))
    return points


# --------------------------------------------------------------------------
# Bracket
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class Bracket:
    """A measured span: a line, a tick at each end, and a label beside it.

    ``side`` is ``+1`` or ``-1`` — which side of the span the label sits on.
    Looking along the span from ``start`` to ``end``, ``+1`` is to the right
    on screen for a downward span and above it for a left-to-right one.

    ``label`` takes one string or several, stacked. The bracket makes a span
    *visible*; writing the number in a corner leaves the reader to find what
    it measures.
    """

    color: str = DIM
    tick: float = 10.0
    w: float = 2.0
    side: int = 1
    size: float = 22.0
    formula: bool = False
    layer: int = 0

    def draw(self, scene, name, start, end, label=None) -> None:
        """Draw it. Shapes are named ``(name, "span")``, ``(name, "tick", 0|1)``
        and ``(name, "label", n)``."""
        (x0, y0), (x1, y1) = start, end
        length = math.hypot(x1 - x0, y1 - y0)
        if length == 0:
            raise ValueError("a bracket needs a span: start and end are the same")
        ux, uy = (x1 - x0) / length, (y1 - y0) / length
        nx, ny = uy * self.side, -ux * self.side

        scene.line(_name(name, "span"), start=start, end=end, w=self.w) \
            .fill(self.color).on(layer=self.layer)
        for k, (px, py) in enumerate((start, end)):
            scene.line(_name(name, "tick", k),
                       start=(px - nx * self.tick, py - ny * self.tick),
                       end=(px + nx * self.tick, py + ny * self.tick), w=self.w) \
                .fill(self.color).on(layer=self.layer)
        if label is None:
            return

        lines = [label] if isinstance(label, str) else list(label)
        tags = [tag(line, at=(0.0, 0.0), color=self.color, size=self.size,
                    formula=self.formula) for line in lines]
        # Clear of the span by a fixed gap, however big the label is along the
        # direction it sits in.
        reach = max(abs(nx) * t.w / 2 + abs(ny) * t.h / 2 for t in tags)
        mx = (x0 + x1) / 2 + nx * (26 + reach)
        my = (y0 + y1) / 2 + ny * (26 + reach)
        for k, t in enumerate(tags):
            spot = (mx, my + (k - (len(tags) - 1) / 2) * (t.h + 4))
            replace(t, x=spot[0], y=spot[1]).draw(
                scene, _name(name, "label", k), layer=self.layer + 3)


# --------------------------------------------------------------------------
# ForceScale
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class ForceScale:
    """One scale for every arrow in a film, so arrows can be compared.

        scale = science.ForceScale(px_per_unit=0.5, cap=140)
        scale.arrow(scene, "weight", at=(400, 300), vector=(0, -200), label="W")

    The arrows are usually the argument. One that quietly rescaled between
    scenes would make steel look no heavier than ice, so the scale is a value
    you make once and use everywhere.

    ``cap`` is the longest an arrow may be drawn. An arrow that would be longer
    is cut at the cap and marked with a break, the way a chart axis is, rather
    than shrunk to fit or left to run into the title — and the caller is told
    it was capped.
    """

    px_per_unit: float
    cap: "float | None" = None
    color: str = "#58c4dd"
    w: float = 7.0
    head: float = 19.0
    gap: str = "#0b0f16"
    layer: int = 0

    def length(self, value: float) -> "tuple[float, bool]":
        """``(pixels, capped)`` for a magnitude."""
        px = abs(value) * self.px_per_unit
        if self.cap is not None and px > self.cap:
            return self.cap, True
        return px, False

    def arrow(self, scene, name, at, vector, label=None):
        """Draw a force from ``at`` along ``vector``; returns the tip, or
        ``None`` if it is too short to draw.

        ``vector`` is ``(fx, fy)`` in your units with **y up**, as physics
        writes it, so ``(0, 50)`` points up the screen. ``label`` is LaTeX,
        set above and to the right of the tip — symbols rather than words, so
        it needs no translation.
        """
        fx, fy = vector
        size = math.hypot(fx, fy)
        px, capped = self.length(size)
        if px < 1.0:
            return None
        ux, uy = fx / size, -fy / size
        tip = (at[0] + ux * px, at[1] + uy * px)
        scene.arrow(name, start=at, end=tip, w=self.w, head=self.head) \
            .fill(self.color).on(layer=self.layer)
        if capped:
            # Two slashes across the shaft near the tip: the break an axis
            # gets when it is cut short, rather than the words "off scale",
            # which started on top of the arrow they described.
            across = (-uy, ux)
            for k in (0, 10):
                cx, cy = tip[0] - ux * (34 + k), tip[1] - uy * (34 + k)
                scene.line(_name(name, "cut", k),
                           start=(cx - across[0] * 13 - ux * 7,
                                  cy - across[1] * 13 - uy * 7),
                           end=(cx + across[0] * 13 + ux * 7,
                                cy + across[1] * 13 + uy * 7), w=self.w) \
                    .fill(self.gap).on(layer=self.layer + 1)
        if label:
            wide, _ = measure_math(label, 30)
            scene.formula(_name(name, "label"), label, size=30,
                          at=(tip[0] + 18 + wide / 2, tip[1] - 22)) \
                .fill(self.color).on(layer=self.layer + 2)
        return tip
