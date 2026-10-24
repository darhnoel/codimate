"""Generated from api/surface.toml by tools/generate.py. Do not edit.

The Python Skin's side of the Interface Manifest: each method sends its
arguments across the Waist and keeps what comes back."""

from __future__ import annotations

from typing import TYPE_CHECKING

from ._waist import _as_points, _as_size, _at, _bool, _canvas, _film_font, _int, _items, _key, _number, _pair, _paired, _point, _points, _radius, _slot, _string, _text, call

if TYPE_CHECKING:
    from .scene import Group, Handle


class _GroupSurface:
    def rect(self, key, *, h, w=None, at=None) -> "Handle":
        """A rectangle. Say where with a point, a Slot, or `at(...)`. Round the corners with `.round(12)`."""
        return self._build("rect", {"group": self._frame(), "key": _key(key), "h": _number(h), "w": _number(w), "at": _at(at)})

    def circle(self, key, *, r, at=None) -> "Handle":
        """A filled circle. `fill("none", edge=...)` draws a ring."""
        return self._build("circle", {"group": self._frame(), "key": _key(key), "r": _number(r), "at": _at(at)})

    def text(self, key, content, *, size=16, font=None, at=None) -> "Handle":
        """Text, centred horizontally. `font` is a font file or an installed font's name, for this text only; unset, the film's font."""
        return self._build("text", {"group": self._frame(), "film_font": _film_font(), "key": _key(key), "content": _text(content), "size": _number(size), "font": _string(font), "at": _at(at)})

    def line(self, key, *, start, end, w=2.0) -> "Handle":
        """A line between two points. `w` is its thickness. Each end may be a Slot (a line joins the middles of two places) or a point."""
        return self._build("line", {"group": self._frame(), "key": _key(key), "start": _point(start), "end": _point(end), "w": _number(w)})

    def arrow(self, key, *, start, end, w=4.0, head=16.0) -> "Handle":
        """An arrow, as a single filled shape: one name, travelling as one thing. `w` is the shaft, `head` the point."""
        return self._build("arrow", {"group": self._frame(), "key": _key(key), "start": _point(start), "end": _point(end), "w": _number(w), "head": _number(head)})

    def polygon(self, key, points, *, closed=True) -> "Handle":
        """A shape with corners. `points` is a sequence of points. Closed and filled by default; `closed=False` leaves an open outline. Two polygons tween only if they have the same number of corners."""
        return self._build("polygon", {"group": self._frame(), "key": _key(key), "points": _points(points), "closed": _bool(closed)})

    def curve(self, key, points, *, w=2.0, closed=False) -> "Handle":
        """A smooth line through every one of `points`: samples, not control points. `w` is the stroke width. Open by default; `closed=True` makes a fillable shape. Two curves tween only if they have the same number of points."""
        return self._build("curve", {"group": self._frame(), "key": _key(key), "points": _points(points), "w": _number(w), "closed": _bool(closed)})

    def arc(self, key, *, r, sweep, at=None) -> "Handle":
        """A slice of a circle: an angle mark, a pie, a dial, an orbit. `sweep` is (start, end) in degrees, clockwise from twelve o'clock; `r` is the radius, or (rx, ry) for an ellipse. Open by default; `.round(1)` closes it into a pie slice."""
        return self._build("arc", {"group": self._frame(), "key": _key(key), "r": _radius(r), "sweep": _pair(sweep), "at": _at(at)})

    def group(self, key, slot=None, *, at=None, anchor=None, w=None) -> "Group":
        """A place to draw a thing made of several shapes. Give it a Slot and it sits where the Slot says (at `anchor`: center, top or bottom), or place it with `at` like any shape. Inside it, 0 is that point, and everything drawn on it moves as one."""
        return self._nest(call("group", {"group": self._frame(), "key": _key(key), "slot": _slot(slot), "at": _at(at), "anchor": _string(anchor), "w": _number(w)}))


class _HandleSurface:
    def fill(self, color="white", *, edge=None, edge_w=None) -> "Handle":
        """Colour it. `edge` outlines it; `color="none"` leaves it unfilled. Saying nothing about the outline leaves it alone."""
        self._change("fill", {"shape": self._shape(), "color": _string(color), "edge": _string(edge), "edge_w": _number(edge_w)})
        return self


class Rule:
    """How things matching `pattern` change; the first matching rule wins. `position` is the path: straight, linear, fall, lift_carry_drop or arc. `ease` is linear, in, out, in_out, back, spring or bounce. `delay` and `stagger` are fractions of the beat; `order` is screen, reverse, name or centre."""

    def __init__(self, pattern, position="straight", *, ease=None, order="screen", **options) -> None:
        made = call("rule", {"pattern": _key(pattern), "position": _string(position), "ease": _string(ease), "order": _string(order), "options": options})
        self._made = tuple(made["payload"])
        self.pattern = made["pattern"]
        self.position = made["position"]
        self.ease = made["ease"]
        self.order = made["order"]
        self.options = made["options"]

    def _payload(self) -> tuple:
        """What the Engine takes."""
        return self._made

    def __repr__(self) -> str:
        return "Rule(" + f"pattern={self.pattern!r}" + ", " + f"position={self.position!r}" + ", " + f"ease={self.ease!r}" + ", " + f"order={self.order!r}" + ", " + f"options={self.options!r}" + ")"


def ngon(sides, r, *, at=(0.0, 0.0), turn=0.0):
    """The corners of a regular polygon, for `polygon`. A triangle is three sides, a hexagon six; `turn` rotates it in degrees, the first corner otherwise pointing straight up."""
    return _as_points(call("ngon", {"sides": _int(sides), "r": _number(r), "at": _point(at), "turn": _number(turn)}))


def star(points, r, *, inner=None, at=(0.0, 0.0), turn=0.0):
    """The corners of a star, for `polygon`. `inner` is the radius of the valleys; it defaults to a proportion that looks like a star rather than a gear."""
    return _as_points(call("star", {"points": _int(points), "r": _number(r), "inner": _number(inner), "at": _point(at), "turn": _number(turn)}))


def row(items, *, gap=40.0, size=None, at=None, within=None):
    """One Slot per item, evenly spaced on a shared baseline, anchored at bottom-centre. `size` is the Slot's width (its height follows) or (w, h). `within` divides part of the canvas instead of all of it. Gives (slot, item) pairs, or bare Slots for a count."""
    items = items if isinstance(items, int) else list(items)
    return _paired(items, call("row", {"canvas": _canvas(), "items": _items(items), "gap": _number(gap), "size": _radius(size), "at": _at(at), "within": _slot(within)}))


def column(items, *, gap=40.0, size=None, at=None, within=None):
    """One Slot per item, stacked vertically around a centre line, anchored at their centres. `x` places the column; it defaults to mid-canvas. Gives (slot, item) pairs, or bare Slots for a count."""
    items = items if isinstance(items, int) else list(items)
    return _paired(items, call("column", {"canvas": _canvas(), "items": _items(items), "gap": _number(gap), "size": _radius(size), "at": _at(at), "within": _slot(within)}))


def measure(text, size=16.0, font=None):
    """How wide and tall `text` will be at `size`: (w, h). Measured by the Engine with the real fonts, fallback included, so it is right for Khmer and anything else that is not plain ASCII. The height is the line height, the same for every word."""
    return _as_size(call("measure", {"film_font": _film_font(), "text": _text(text), "size": _number(size), "font": _string(font)}))


def ease(t):
    """The easing curve the Engine applies between two moments: the same curve your animation uses, not a copy of it."""
    return call("ease", {"t": _number(t)})
