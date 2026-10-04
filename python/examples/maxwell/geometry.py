"""Small pieces of geometry the scenes share."""

import codimate as cm


def side_by_side(latexes, size, centre):
    """x-centres that set several formulas end to end, centred on `centre`:
    how one equation is shown in two pieces, so the second can arrive alone."""
    widths = [cm.measure_math(latex, size)[0] for latex in latexes]
    x, centres = centre - sum(widths) / 2, []
    for wide in widths:
        centres.append(x + wide / 2)
        x += wide
    return centres


def bezier(p0, p1, p2, p3, count=40):
    pts = []
    for i in range(count + 1):
        t = i / count
        a, b, c, d = (1 - t) ** 3, 3 * (1 - t) ** 2 * t, 3 * (1 - t) * t**2, t**3
        pts.append(
            (
                a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0],
                a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1],
            )
        )
    return pts


def head_at(point, direction, size=11):
    """A small triangle on a line, pointing along `direction` (a unit vector)."""
    (x, y), (dx, dy) = point, direction
    nx, ny = -dy, dx
    return [
        (x + dx * size, y + dy * size),
        (x - dx * size * 0.6 + nx * size * 0.6, y - dy * size * 0.6 + ny * size * 0.6),
        (x - dx * size * 0.6 - nx * size * 0.6, y - dy * size * 0.6 - ny * size * 0.6),
    ]
