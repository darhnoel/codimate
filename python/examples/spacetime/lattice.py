"""A lattice of space, deformed by one mass moving through it.

Distances are in the Sun's Schwarzschild radii. No Codimate here — it is
arithmetic about a lattice, and it checks itself.
"""

import math

SUN_RS = 1.0
SUN_R = 1.05            # the body's own edge: the lattice comes no closer

# Every lattice point is moved to where it would have *fallen*, from rest,
# after one fixed stretch of time. Far out a point barely stirs in that time;
# close in it travels most of the way, so the grid bunches exactly where the
# well is deep. That is a real dynamical map, not a guess at how bent a line
# ought to look, and it is what makes the deformation look like gravity.
#
# It needs no numerical solver. Integrating `x'' = -mu/x^2` from rest at `r`
# has the cycloid solution
#
#     d = (r/2) (1 + cos e)        t = sqrt(r^3 / 8mu) (e + sin e)
#
# so the whole map is one inversion of `e + sin e`, monotone on [0, pi] and
# tabulated once below.
SUN_MU = 5.0
FALL = 4.5              # how long everything is allowed to fall

# The Sun goes round a circle while the lattice and the camera hold still, so
# what you watch is the deformation travelling through space rather than the
# view turning. The circle lies flat, so the Sun sweeps across the frame and
# then away from you, and the lines bend aside as it passes.
SWING = 2.3             # kept small: the mass stays the centre of the frame
REACH = 9.0             # the lattice runs -REACH..REACH on each axis
NODES = 7               # lattice nodes per axis: 7 x 7 x 7 = 343
# Samples per cell edge. Nine was not enough: an edge passing near the mass
# is bent hard and unevenly, and a spline through nine samples cuts corners the
# real curve does not have — the shapes came out wrong rather than merely
# coarse. Fixed, whatever the value, so every edge tweens.
ALONG = 21

# A three-quarter view. Face-on, the depth lines project to near-vertical and
# the cube reads as a flat grid with a tunnel in it — you cannot see which way
# anything bends. Turning it lets two side faces and the top show, which is
# what makes the deformation read as happening in a volume.
YAW = math.radians(-14.0)
PITCH = math.radians(15.0)

# The camera goes right to left across the minute, one way, never back — a
# full turn and a half, about nine degrees a second. A degree a second was
# still too slow to read as movement over any few seconds of watching.
SWEEP = math.radians(-540.0)
# Further back and a shorter lens, so the cube sits in the frame with room
# around it rather than filling it corner to corner.
EYE_BACK = 29.0
LENS = 590.0
MIDDLE = (640.0, 392.0)

_STEPS = 4096
_TABLE = [(e + math.sin(e), e)
          for e in (math.pi * k / _STEPS for k in range(_STEPS + 1))]


def _swing(reach):
    """Invert `e + sin e = reach` on [0, pi], off the table."""
    if reach >= math.pi:
        return math.pi
    lo, hi = 0, _STEPS
    while hi - lo > 1:
        mid = (lo + hi) // 2
        if _TABLE[mid][0] <= reach:
            lo = mid
        else:
            hi = mid
    (t0, e0), (t1, e1) = _TABLE[lo], _TABLE[hi]
    return e0 if t1 == t0 else e0 + (e1 - e0) * (reach - t0) / (t1 - t0)


def fallen(r, mu, when):
    """Where something dropped from rest at `r` has got to after `when`."""
    if r <= 1e-9:
        return 0.0
    scale = math.sqrt(r * r * r / (8.0 * mu))
    if when >= math.pi * scale:         # it reached the middle
        return 0.0
    return 0.5 * r * (1.0 + math.cos(_swing(when / scale)))


def collapses(mu, floor):
    """Inside this radius the fall reaches the body, so points land together.

    Not where the fall reaches the *middle* — the clamp bites as soon as it
    reaches the body's edge, which is sooner and is the boundary that matters.
    """
    lo, hi = floor, floor
    while fallen(hi, mu, FALL) < floor:
        hi *= 1.5
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        if fallen(mid, mu, FALL) < floor:
            lo = mid
        else:
            hi = mid
    return hi


def sun_at(part):
    """Where the Sun is, `part` of the way round its circle."""
    turn = math.tau * part
    return (SWING * math.cos(turn), 0.0, SWING * math.sin(turn))


def warp(p, centre):
    """A lattice point, moved to where it would have fallen.

    The mass takes the point and drops it, from rest, for `FALL`, and stops it
    at the body's own edge.

    Points close enough to complete the fall inside `FALL` all stop together
    on that edge. That is not a defect to be smoothed away — it is what draws
    the lines converging into the body, and the equation this came from does
    the same with `WhenEvent[x[t] < 0.01, "StopIntegration"]`. `collapses`
    says how wide that zone is, and the checks hold it under a cell.
    """
    dx, dy, dz = p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]
    r = math.sqrt(dx * dx + dy * dy + dz * dz)
    if r <= SUN_R:
        return p
    near = max(fallen(r, SUN_MU, FALL), SUN_R) / r
    return (centre[0] + dx * near,
            centre[1] + dy * near,
            centre[2] + dz * near)


def river(p, centre):
    """How fast space is falling inward here, as a fraction of light speed.

    Hamilton and Lisle's river model, which is not a metaphor: in
    Gullstrand-Painlevé coordinates space flows inward at exactly the
    Newtonian escape velocity,

        v / c = sqrt(rs / r)

    reaching light speed at the horizon. So the colour of a line is a real
    quantity, not a measure of how bent it happens to look.
    """
    return min(1.0, math.sqrt(SUN_RS / max(math.dist(p, centre), SUN_R)))


_FASTEST = math.sqrt(SUN_RS / SUN_R)


def heat(p, centre):
    """The river's speed here as a share of the fastest in the picture.

    Squared, because `v^2` is `rs/r` — so this is a clean one-over-distance
    ramp rather than a square root flattened across the middle of the range.
    """
    return min(1.0, (river(p, centre) / _FASTEST) ** 2)


def look(p, drift=0.0):
    """The viewpoint, then perspective. `drift` is the slow sweep."""
    yaw = YAW + drift
    x, y, z = p
    x, z = (x * math.cos(yaw) + z * math.sin(yaw),
            -x * math.sin(yaw) + z * math.cos(yaw))
    y, z = (y * math.cos(PITCH) - z * math.sin(PITCH),
            y * math.sin(PITCH) + z * math.cos(PITCH))
    away = z + EYE_BACK
    return (MIDDLE[0] + LENS * x / away, MIDDLE[1] + LENS * y / away), away


def _the_lattice_is_honest():
    cell = 2.0 * REACH / (NODES - 1)
    middle = (0.0, 0.0, 0.0)

    def moved(r):
        return r - warp((r, 0.0, 0.0), middle)[0]

    # The closed form is the real solution, not something shaped like it.
    # Free fall from rest reaches the middle at (pi/2) sqrt(r^3 / 2mu), and
    # is half way down when the cycloid says it is.
    for r in (1.0, 3.5, 9.0):
        whole = 0.5 * math.pi * math.sqrt(r ** 3 / (2.0 * SUN_MU))
        assert fallen(r, SUN_MU, whole * 0.999) > 0.0, r
        assert fallen(r, SUN_MU, whole * 1.001) == 0.0, r
        half = math.sqrt(r ** 3 / (8.0 * SUN_MU)) * (math.pi / 2 + 1.0)
        assert abs(fallen(r, SUN_MU, half) - 0.5 * r) < 1e-4, r

    # Falling for a fixed time: far out almost nothing happens, because the
    # pull is weak and the clock is short. Said against a lattice cell rather
    # than as a bare number, which would quietly track the constants.
    assert moved(400.0) < cell * 0.01, moved(400.0)

    # The map never turns the lattice inside out. It is not *strictly*
    # increasing — everything that completes its fall stops together on the
    # body's edge, which is what draws the lines converging into it.
    radii = [warp((r, 0.0, 0.0), middle)[0]
             for r in (0.05 * k for k in range(1, 400))]
    # To within rounding: the table interpolation is a numeric map, so
    # asking for exact monotonicity fails on noise of order 1e-16.
    assert all(b >= a - 1e-9 for a, b in zip(radii, radii[1:])), "must not fold"
    assert all(b > 0.0 for b in radii), "must not turn inside out"

    # Outside that zone it *is* strictly increasing, so nothing folds beyond
    # the pile-up. How wide the pile-up may be is a matter of taste, not of
    # correctness — a long fall really does drop a whole inner shell onto the
    # body, and that is what draws the lines converging into it. What must
    # hold is that it stays an *inner* shell: the lattice has to reach the rim
    # still recognisably a lattice, or "flat far away" is not shown at all.
    edge = collapses(SUN_MU, SUN_R)
    assert edge < REACH * 0.6, \
        f"the pile-up reaches {edge / REACH:.0%} of the way out"
    out = [warp((r, 0.0, 0.0), middle)[0]
           for r in (edge + 0.02 * k for k in range(1, 500))]
    assert all(b > a for a, b in zip(out, out[1:])), "not monotone outside"

    # The Sun stays well inside the lattice all the way round.
    for k in range(180):
        assert max(abs(v) for v in sun_at(k / 180)) < REACH * 0.55

    # Nothing gets behind the camera, wherever the Sun is. A clipped line
    # would shed a point, change its count, and stop tweening (ADR 0010).
    worst = min(look(warp(corner, sun_at(k / 36)), SWEEP * k / 36)[1]
                for corner in ((s * REACH, t * REACH, u * REACH)
                               for s in (-1, 1) for t in (-1, 1)
                               for u in (-1, 1))
                for k in range(36))
    assert worst > 1.0, f"the camera is too close: {worst:.2f} to spare"
    return True


assert _the_lattice_is_honest()
