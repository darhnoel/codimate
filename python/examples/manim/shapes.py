"""The two outlines, walked by distance rather than by angle.

No Codimate in this file. It is arithmetic about squares and circles, which is
the seam worth splitting on — it can be checked without rendering anything.
"""

import math

MIDDLE = (640.0, 360.0)
# Every outline has this many points, so each one tweens into the next. The
# count is not arbitrary: `POINTS - 1` is 768, which divides by 4 and by 3, so
# a square's corners and a triangle's land exactly on samples instead of being
# chopped off by a chord between the two either side of them. At 96 points the
# far corner of the square was cut by five and a half pixels, under a stroke
# four pixels wide. The density is the other half — while a shape is only part
# drawn the samples fall elsewhere again, and 768 keeps that miss to half of
# one pixel, which is a third of a device pixel at the scale these render at.
POINTS = 769
HALF, R = 130.0, 150.0
START = math.radians(45.0)  # both outlines begin at the same corner


def corners(tilt=0.0, centre=MIDDLE):
    """The square's four corners, with the first repeated to close the walk."""
    turn = START + math.radians(tilt)
    return [(centre[0] + HALF * math.sqrt(2) * math.cos(turn + math.tau * k / 4),
             centre[1] + HALF * math.sqrt(2) * math.sin(turn + math.tau * k / 4))
            for k in range(5)]


def ngon(sides, radius=HALF * math.sqrt(2)):
    """A walker for a regular polygon, by *distance* round its perimeter.

    Describing a square as `r(a) = HALF / max(|cos a|, |sin a|)` and stepping
    `a` evenly is the obvious thing and it is not smooth: the radius stretches
    toward the corners, so equal angles cover more perimeter there and the pen
    hurries through the corners and dawdles along the flats. Walking the sides
    at a constant rate makes the tip speed exactly even.
    """
    def walk(f, tilt=0.0, centre=MIDDLE):
        turn = START + math.radians(tilt)
        corner = [(centre[0] + radius * math.cos(turn + math.tau * k / sides),
                   centre[1] + radius * math.sin(turn + math.tau * k / sides))
                  for k in range(sides + 1)]
        walked = min(f, 1.0) * sides
        side = min(int(walked), sides - 1)   # the last point is the first corner
        along = walked - side
        a, b = corner[side], corner[side + 1]
        return (a[0] + (b[0] - a[0]) * along, a[1] + (b[1] - a[1]) * along)
    return walk


square = ngon(4)
triangle = ngon(3, R)


def circle(f, tilt=0.0, centre=MIDDLE):
    a = START + math.radians(tilt) + math.tau * f
    return (centre[0] + R * math.cos(a), centre[1] + R * math.sin(a))


def outline(shape, part=1.0, tilt=0.0, centre=MIDDLE):
    """`POINTS` points along the first `part` of an outline.

    Resampling to a *fixed* count is what lets a shape be drawn on. Two
    polygons with different point counts do not interpolate — the later one
    stands for the whole beat (ADR 0010) — so a path that grows by adding
    points cannot animate. One that keeps ninety-six and spreads them over a
    longer and longer run can, and at `part` of 1 the last point lands back on
    the first.
    """
    return [shape(part * i / (POINTS - 1), tilt, centre)
            for i in range(POINTS)]


def _corners_are_never_cut():
    """A sample lands on every corner of a finished shape, and while one is
    being drawn no corner is missed by as much as a pixel."""
    assert (POINTS - 1) % 12 == 0, POINTS
    for sides, walk in ((4, square), (3, triangle)):
        for part in [1.0] + [k / 48 for k in range(1, 48)]:
            drawn = outline(walk, part)
            reach = walk(0.0)
            radius = math.dist(reach, MIDDLE)
            here = [(MIDDLE[0] + radius * math.cos(START + math.tau * k / sides),
                     MIDDLE[1] + radius * math.sin(START + math.tau * k / sides))
                    for k in range(sides)
                    if k / sides <= part + 1e-9]
            miss = max((min(math.dist(c, p) for p in drawn) for c in here),
                       default=0.0)
            assert miss < (1e-9 if part == 1.0 else 0.6), (sides, part, miss)
    return True


def _the_pen_moves_at_one_speed():
    """What makes the drawing smooth, and the reason this file exists."""
    for tilt in (0.0, 45.0):
        tip = [outline(square, k / 48, tilt)[-1] for k in range(49)]
        gaps = [math.dist(a, b) for a, b in zip(tip, tip[1:])]
        assert max(gaps) - min(gaps) < 1e-9, (tilt, min(gaps), max(gaps))

    # Every outline starts at the same bearing and closes on itself, so one
    # morphs into another without twisting or unzipping.
    for shape in (square, circle, triangle):
        assert math.dist(outline(shape)[0], outline(shape)[-1]) < 1e-9
        assert abs(math.atan2(shape(0.0)[1] - MIDDLE[1],
                              shape(0.0)[0] - MIDDLE[0]) - START) < 1e-9
    return True


assert _corners_are_never_cut()
assert _the_pen_moves_at_one_speed()
