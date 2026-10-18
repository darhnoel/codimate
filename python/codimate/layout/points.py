"""Corner points of regular shapes: polygons and stars."""

from __future__ import annotations



def ngon(sides: int, r: float, at=(0.0, 0.0), turn: float = 0.0) -> list:
    """The corners of a regular polygon, for `Group.polygon`.

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
    """The corners of a star, for `Group.polygon`.

        scene.polygon("s", cm.star(5, r=80, at=(640, 360))).fill("yellow")

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
