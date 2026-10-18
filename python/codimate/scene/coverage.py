"""Where a shape sits, and which labels something else has drawn over."""

from __future__ import annotations

from typing import TYPE_CHECKING


if TYPE_CHECKING:
    from .root import Scene


def box(shape) -> "tuple[float, float, float, float] | None":
    """Where a shape sits: `(left, top, width, height)`.

    Derivable from the payload, which is flat by design — except text and a
    **formula**, which the Engine measures (a formula through Typst, ADR 0005).
    `None` only for a formula on a machine without `typst`.
    """
    kind, x, y = shape.kind, shape.x, shape.y
    if kind in ("rect", "image", "svg"):
        return (x - shape.w / 2, y - shape.h / 2, shape.w, shape.h)
    if kind in ("circle", "arc"):
        return (x - shape.r, y - shape.r, shape.r * 2, shape.r * 2)
    if kind == "line":
        left, top = min(x, shape.x2), min(y, shape.y2)
        return (left, top, abs(shape.x2 - x), abs(shape.y2 - y))
    if kind in ("polygon", "curve"):
        # Flat by the time it is here — x, y, x, y — because that is the shape
        # the Engine diffs (ADR 0008), not pairs.
        flat = shape.points
        if len(flat) < 2:
            return None
        xs, ys = flat[0::2], flat[1::2]
        return (min(xs), min(ys), max(xs) - min(xs), max(ys) - min(ys))
    if kind == "text":
        from ..layout import measure
        w, h = measure(str(shape.text), shape.size)
        return (x - w / 2, y - h / 2, w, h)
    if kind == "formula":
        # Typst typesets it — once, cached in the Engine, centred on its point.
        from ..layout import measure_math
        try:
            w, h = measure_math(str(shape.text), shape.size)
        except Exception:   # no typst here: say nothing rather than guess
            return None
        return (x - w / 2, y - h / 2, w, h)
    return None


def covered(scene: "Scene") -> "list[tuple[str, str]]":
    """Every label something is drawn on, as `(above, below)` pairs.

    Two rules and no configuration (ADR 0017): nothing visible may be drawn
    above a text or a formula, at any opacity, and no two of them may
    overlap. The plate under a label is drawn *below* it and passes; the arrow
    across it is drawn *above* it and does not.
    """
    # ponytail: boxes ignore rotation, and a circle or a curve counts its whole
    # bounding box — exact enough for labels, which are what this is about.
    shapes = [s for s in scene._shapes.values()
              if s.opacity > 0 and (s.color != "none" or s.edge_w > 0)]
    # The Engine draws in (layer, name) order: later is on top.
    shapes.sort(key=lambda s: (s.layer, s.item))
    boxes = {s.item: box(s) for s in shapes}
    world = scene._focus is not None      # a camera moves the world, not overlays

    found = []
    for i, label in enumerate(shapes):
        if label.kind not in ("text", "formula") or boxes[label.item] is None:
            continue
        # Only what is drawn after the label can hide it. Beneath it is its
        # plate; and two labels are found once, from the lower of the two.
        for other in shapes[i + 1:]:
            if boxes[other.item] is None:
                continue
            if world and _overlay(label) != _overlay(other):
                continue
            if _touches(other, boxes[other.item], boxes[label.item]):
                found.append((other.item, label.item))
    return found


def _overlay(shape) -> bool:
    return shape.item.startswith("_overlay")


def _touches(shape, outer, label, slack: float = 2.0) -> bool:
    """Whether `shape` crosses the box `label`, by more than `slack` pixels.

    A measured box includes the blank side bearing of its first and last
    glyph, so a pixel or two at the edge is never ink on ink: words set edge
    to edge, and an axis label set flush against its tick, are not covered.
    """
    lx, ly, lw, lh = label
    lx, ly, lw, lh = lx + slack, ly + slack, lw - 2 * slack, lh - 2 * slack
    if shape.kind == "line":
        # A diagonal line's box covers a lot the line does not; clip the
        # segment itself against the label, grown by half the stroke.
        grow = shape.w / 2
        return _crosses((shape.x, shape.y), (shape.x2, shape.y2),
                        (lx - grow, ly - grow, lx + lw + grow, ly + lh + grow))
    ox, oy, ow, oh = outer
    return ox < lx + lw and lx < ox + ow and oy < ly + lh and ly < oy + oh


def _crosses(start, end, rect) -> bool:
    """Liang–Barsky: does the segment enter the rectangle
    `(left, top, right, bottom)` at all?"""
    (x0, y0), (x1, y1) = start, end
    left, top, right, bottom = rect
    dx, dy, lo, hi = x1 - x0, y1 - y0, 0.0, 1.0
    for p, q in ((-dx, x0 - left), (dx, right - x0),
                 (-dy, y0 - top), (dy, bottom - y0)):
        if p == 0:
            if q < 0:
                return False
        else:
            t = q / p
            if p < 0:
                lo = max(lo, t)
            else:
                hi = min(hi, t)
            if lo > hi:
                return False
    return True
