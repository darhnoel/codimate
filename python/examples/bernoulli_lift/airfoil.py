"""A Joukowski airfoil and the exact flow around it.

Not a drawing of a wing — the real potential-flow solution, which has been
known since 1910. A circle in one complex plane maps to an airfoil in another,
and the flow around a circle is something you can write down. The Kutta
condition fixes the one free constant, the circulation, by requiring the flow
to leave the trailing edge smoothly instead of whipping round it.

Everything here is complex arithmetic. No Codimate, no drawing.
"""

import cmath
import math

B = 1.0                          # the map's scale
CENTRE = complex(-0.10, 0.09)    # off-centre: gives the wing thickness and camber
ALPHA = math.radians(7.0)        # angle of attack
U = 1.0                          # free-stream speed

# The circle that maps to the airfoil must pass through +B, or the trailing
# edge is rounded instead of sharp.
RADIUS = abs(B - CENTRE)

# Kutta: the circulation that puts a stagnation point exactly at the trailing
# edge, so the flow leaves it smoothly. This is the whole of where lift comes
# from, and nothing about it mentions path length.
BETA = math.asin(CENTRE.imag / RADIUS)
GAMMA = 4.0 * math.pi * U * RADIUS * math.sin(ALPHA + BETA)


def to_airfoil(zeta):
    """Joukowski: a circle becomes a wing."""
    return zeta + B * B / zeta


def to_circle(z):
    """The inverse, on the branch outside the circle."""
    root = cmath.sqrt(z * z - 4.0 * B * B)
    outer = 0.5 * (z + root)
    return outer if abs(outer - CENTRE) >= abs(0.5 * (z - root) - CENTRE) else 0.5 * (z - root)


def surface(count=240):
    """Points around the wing, trailing edge first."""
    return [to_airfoil(CENTRE + RADIUS * cmath.exp(1j * (theta + math.pi)))
            for theta in (2.0 * math.pi * i / count for i in range(count + 1))]


def velocity(z):
    """Flow velocity at a point outside the wing, as (u, v).

    The complex potential around a circle, differentiated, then divided by the
    map's derivative to carry it into the airfoil plane.
    """
    zeta = to_circle(z)
    xi = zeta - CENTRE
    if abs(xi) < RADIUS * 0.999:
        return None                  # inside the wing

    dw = (U * (cmath.exp(-1j * ALPHA) - RADIUS ** 2 * cmath.exp(1j * ALPHA) / (xi * xi))
          + 1j * GAMMA / (2.0 * math.pi * xi))
    dz = 1.0 - B * B / (zeta * zeta)
    if abs(dz) < 1e-9:
        return None                  # the trailing edge itself
    w = dw / dz
    return (w.real, -w.imag)         # conjugate: dw/dz is u - iv


def fill_columns(count=130):
    """The wing as vertical spans: (x, top y, bottom y).

    A circle can be filled and a rectangle can be filled, but an arbitrary
    outline cannot — the Authoring Surface has no polygon. So the wing is
    filled the way a rasteriser would do it, one column at a time.
    """
    points = surface(600)[:-1]          # closed: drop the repeated point
    nose = min(range(len(points)), key=lambda i: points[i].real)
    tail = max(range(len(points)), key=lambda i: points[i].real)

    # The two chains between nose and tail are the two surfaces. Splitting at
    # the nose alone leaves one chain wrapping round the whole wing.
    def chain(start, end):
        out, i = [points[start]], start
        while i != end:
            i = (i + 1) % len(points)
            out.append(points[i])
        return out

    one, other = chain(nose, tail), chain(tail, nose)
    mean = lambda c: sum(p.imag for p in c) / len(c)
    upper, lower = (one, other) if mean(one) > mean(other) else (other, one)

    def edge_at(x, chain):
        for a, b in zip(chain, chain[1:]):
            lo, hi = min(a.real, b.real), max(a.real, b.real)
            if lo <= x <= hi and hi > lo:
                return a.imag + (b.imag - a.imag) * (x - a.real) / (b.real - a.real)
        return None

    xs = [p.real for p in points]
    left, right = min(xs), max(xs)
    columns = []
    for i in range(count + 1):
        x = left + (right - left) * i / count
        top, bottom = edge_at(x, upper), edge_at(x, lower)
        if top is not None and bottom is not None:
            columns.append((x, max(top, bottom), min(top, bottom)))
    return columns


def turned_by():
    """How far the wing turns the air, in degrees. Negative is downwards.

    The other half of the explanation, and the half the familiar diagram
    leaves out: the wing throws air down, so the air throws the wing up. This
    is Newton's third law and it needs no mention of path length either.
    """
    ahead = velocity(complex(-3.5, 0.25))
    behind = velocity(complex(3.0, 0.25))
    return math.degrees(math.atan2(behind[1], behind[0])
                        - math.atan2(ahead[1], ahead[0]))
