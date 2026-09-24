"""The physics and the geometry. No Codimate in this file.

Densities are real, and everything the picture does follows from them: how deep
a thing floats, how far the water rises, how long every arrow is. The only
numbers chosen by eye are the tank and the box — the hull's wall thickness is
solved at every instant so that the steel is conserved, and where the body sits
is solved from that rather than animated by hand.
"""

import math

# ---------------------------------------------------------------- physics

RHO_WATER = 1000.0      # kg/m^3
RHO_ICE = 917.0
RHO_IRON = 7850.0
GRAVITY = 9.81          # cancels out of every comparison; carried for honesty


def submerged_fraction(rho_object, rho_fluid=RHO_WATER):
    """How much of a floating thing sits under the surface.

    At rest an object carries its own weight in displaced fluid, so
    `rho_object V g = rho_fluid V_sub g` and the fraction is just the ratio of
    densities. Anything denser than the fluid cannot balance at all: it sinks,
    and the fraction is the whole of it.
    """
    return min(1.0, rho_object / rho_fluid)


def weight(rho, volume):
    return rho * volume * GRAVITY


def buoyancy(volume_displaced, rho_fluid=RHO_WATER):
    """Archimedes: the fluid pushes up with the weight of what it lost."""
    return rho_fluid * volume_displaced * GRAVITY


ICE_SUBMERGED = submerged_fraction(RHO_ICE)      # 0.917, computed


# ---------------------------------------------------------------- the tank

# One tank, and a small one, because nothing in the film is ever bigger than
# the box. An earlier cut ended with a ship — a hull nine times the box's area
# — and the tank had to be sized for that, which left the box looking lost in
# water it never reached. The ship is gone: the last case is the same box with
# thin walls, so the tank only has to hold a box of volume V.
TANK = (380.0, 150.0, 900.0, 560.0)              # left, top, right, bottom
TANK_W = TANK[2] - TANK[0]
REST_LEVEL = 280.0                               # with nothing in the tank
FLOOR = TANK[3] - 1.5                            # where a sunk thing rests


def water_level(displaced_area):
    """Where the surface sits once something has pushed water aside.

    A tank this wide gains `area / width` of depth for every bit of volume put
    into it, which is the whole of "the water went up because the box went in".
    """
    return REST_LEVEL - displaced_area / TANK_W


def surface(bottom, w, h):
    """Where the water settles, and how much of the object is under it.

    The two answer each other: the object pushes the surface up, and the
    higher surface swallows more of the object. For a straight sided tank
    that is one equation rather than a search —

        L = REST - w (bottom - L) / TANK_W

    solved for L. Driving the level directly and reading the depth off it
    would let the two drift apart the moment anything moved.
    """
    if bottom <= REST_LEVEL:                       # still clear of the water
        return REST_LEVEL, 0.0
    sunk = water_level(w * h)                      # fully under
    if bottom - h >= sunk:
        return sunk, 1.0
    level = (REST_LEVEL - w * bottom / TANK_W) / (1.0 - w / TANK_W)
    return level, max(0.0, min(1.0, (bottom - level) / h))


def displaced(bottom, w, h):
    """The volume pushed aside: the part of the object under the waterline."""
    return w * h * surface(bottom, w, h)[1]


def rise(bottom, w, h):
    """How far the surface has climbed — the displaced volume, made visible."""
    return REST_LEVEL - surface(bottom, w, h)[0]


def floating_bottom(fraction, w, h):
    """Where an object's underside sits when it floats `fraction` submerged."""
    return water_level(w * h * fraction) + fraction * h


def settles(rho, w=None, h=None):
    """Where a box of average density `rho` comes to rest, on its own.

    This is what the last third of the film runs on. The box is never told to
    rise: it is asked where it belongs, and as its walls thin the answer
    changes from "on the floor" to "floating" at the moment its average
    density passes the water's. The lift-off is a consequence, not a cue.

        rho < rho_water   ->  it floats, `rho / rho_water` of it under
        otherwise         ->  the floor
    """
    w = BOX_W if w is None else w
    h = BOX_H if h is None else h
    if rho > RHO_WATER:
        return FLOOR
    # Equality is the neutral case, and it is not a special one: the fraction
    # comes out at 1.0 and the body hangs exactly full under, which is what
    # the box of water does.
    return floating_bottom(rho / RHO_WATER, w, h)


# ----------------------------------------------------------------- the box

BOX_W, BOX_H = 230.0, 170.0
BOX_AREA = BOX_W * BOX_H                 # the "volume V" the film talks about

POINTS = 240            # per outline, fixed, so the box can morph into a hull
CORNER = 14.0           # every corner is arced, on the box and on the hull


def _round_corners(corners, radius):
    """Replace each corner with a short arc, keeping the shape closed.

    The body has to be a polygon rather than a rounded rect, because only a
    polygon can morph into the hull. So the rounding is put into the outline
    itself: each corner becomes a few points on an arc of `radius`, clamped so
    a short side cannot be eaten from both ends at once.
    """
    n = len(corners)
    out = []
    for i, here in enumerate(corners):
        before, after = corners[i - 1], corners[(i + 1) % n]
        r = radius
        for other in (before, after):
            r = min(r, math.dist(here, other) / 2.0)
        if r < 0.5:                                # too tight to round
            out.append(here)
            continue
        # A quadratic arc from one arm to the other with the corner itself as
        # the control point. Five samples is plenty at this radius, and `_walk`
        # resamples the whole outline afterwards anyway.
        a = (here[0] + (before[0] - here[0]) * r / math.dist(here, before),
             here[1] + (before[1] - here[1]) * r / math.dist(here, before))
        b = (here[0] + (after[0] - here[0]) * r / math.dist(here, after),
             here[1] + (after[1] - here[1]) * r / math.dist(here, after))
        for k in range(5):
            u = k / 4.0
            m = (1 - u) ** 2, 2 * u * (1 - u), u ** 2
            out.append((m[0] * a[0] + m[1] * here[0] + m[2] * b[0],
                        m[0] * a[1] + m[1] * here[1] + m[2] * b[1]))
    return out


def _walk(corners, n, radius=CORNER):
    """`n` points evenly spaced around a closed outline, by distance.

    By distance rather than by corner, so two outlines with different corner
    counts still correspond point for point and can tween into one another.
    Sampling by corner would need the same number of corners, which a
    rectangle and a hull do not have.
    """
    corners = _round_corners(list(corners), radius) if radius else list(corners)
    ring = list(corners) + [corners[0]]
    runs = [math.dist(a, b) for a, b in zip(ring, ring[1:])]
    total = sum(runs)
    out, walked, side = [], 0.0, 0
    for i in range(n):
        want = total * i / n
        while side < len(runs) - 1 and walked + runs[side] < want:
            walked += runs[side]
            side += 1
        along = (want - walked) / runs[side] if runs[side] else 0.0
        a, b = ring[side], ring[side + 1]
        out.append((a[0] + (b[0] - a[0]) * along,
                    a[1] + (b[1] - a[1]) * along))
    return out


# ---------------------------------------------------------- the hollow box

# The one number in this section chosen by eye, and it is a drawing choice:
# how thick to leave the walls. Everything else — how much iron is left, what
# the box then weighs, how deep it floats — follows from it.
WALL = 5.0
SOLID = BOX_H / 2.0             # walls meeting in the middle: no cavity at all


def metal(wall):
    """How much iron is left when the walls are `wall` thick.

    A box is its outside less its inside, and the box is **closed** — walls on
    all four sides, lid included. An open one was drawn first and cost an
    idealisation: a vessel open at the top, held under water, would fill, and
    the film had to take the air inside it for granted. A sealed box of thin
    iron around air simply floats, and there is nothing to excuse.

    At `SOLID` the inside is nothing and the answer is the whole of V; at
    `WALL` it is a few per cent of it.
    """
    inside = max(BOX_W - 2 * wall, 0.0) * max(BOX_H - 2 * wall, 0.0)
    return BOX_AREA - inside


def density(rho, wall):
    """The average density of a box of `rho` with walls `wall` thick.

    One formula for every object in the film. A solid box of water comes out
    at 1,000 and a solid box of ice at 917, because at `SOLID` the metal is
    the whole volume — and the hollow iron box comes out at a few hundred,
    because most of what is inside its outside is air. That is the whole of
    the last section: what floats a thing is the average over its outside,
    not the density of the stuff it is made of.
    """
    return rho * metal(wall) / BOX_AREA


HOLLOW_RHO = density(RHO_IRON, WALL)
HOLLOW_SUBMERGED = submerged_fraction(HOLLOW_RHO)
LEFT_OF_IT = metal(WALL) / BOX_AREA      # the fraction of the iron still there


def thinning(along):
    """Wall thickness `along` the way from a solid box to a hollow one.

    Walked by the *cavity*, not by the thickness. Linear in thickness, almost
    the whole animation is spent in the thick-walled range where the average
    density hardly moves; linear in the hole it is cutting, the number falls
    at a readable rate.
    """
    inside = (BOX_W - 2 * WALL) * (BOX_H - 2 * WALL) * along
    lo, hi = WALL, SOLID
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        if max(BOX_W - 2 * mid, 0.0) * max(BOX_H - 2 * mid, 0.0) > inside:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def _lift_off():
    """How far into the thinning the box first carries its own weight.

    Always near the end: iron is 7.85 times water, so all but an eighth of it
    has to be gone before the average drops under 1,000. The film uses this to
    overlap the last of the thinning with the rise, so a box that can float is
    never shown sitting on the bottom.
    """
    lo, hi = 0.0, 1.0
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        if density(RHO_IRON, thinning(mid)) > RHO_WATER:
            lo = mid
        else:
            hi = mid
    return max(0.0, lo - 0.05)


LIFTS_AT = _lift_off()


def outline(middle):
    """The body, as `POINTS` points. The same box in every scene of the film.

    It never changes — not its size and not its shape. The box is closed, so
    hollowing it cuts no notch in this outline: the cavity is a second shape
    drawn inside it, and the walls are what is left showing between the two.
    A polygon cannot have a hole in it, and it does not need one.
    """
    w, h = BOX_W, BOX_H
    x, y = middle[0] - w / 2, middle[1] - h / 2
    return _walk([(x, y), (x + w, y), (x + w, y + h), (x, y + h)], POINTS)


def cavity(middle, wall=SOLID):
    """The air inside the box: left, top, right, bottom in pixels.

    Inset on all four sides, because the box has a lid. Drawn over the water,
    because the Engine has no clipping — without it the translucent water runs
    straight through the box and the inside looks flooded, which is the
    opposite of what floats it.
    """
    x, y = middle[0] - BOX_W / 2, middle[1] - BOX_H / 2
    return (x + wall, y + wall, x + BOX_W - wall, y + BOX_H - wall)


# ------------------------------------------------------------- the arrows

FORCE_PX = 110.0        # what a fully submerged box of water is drawn as
ARROW_CAP = 140.0       # beyond this an arrow reaches the title


def arrow_length(force):
    """Force to pixels, on one scale for the whole film.

    One scale matters more than it sounds: the arrows are the argument, and an
    arrow that quietly rescaled between scenes would make steel look no heavier
    than ice. Steel does run off the end, so it is capped — and the caller is
    told it was, rather than the picture pretending it fits.
    """
    px = FORCE_PX * force / buoyancy(BOX_AREA)
    return min(px, ARROW_CAP), px > ARROW_CAP


def _the_physics_holds():
    # 0.917 is not typed in anywhere. It falls out of 917 over 1000.
    assert abs(ICE_SUBMERGED - 0.917) < 1e-9
    assert submerged_fraction(RHO_IRON) == 1.0, "solid iron cannot float"
    assert submerged_fraction(RHO_WATER) == 1.0, "a water box is neutral"

    # At the floating depth, up equals down. That is the whole principle.
    up = buoyancy(BOX_AREA * ICE_SUBMERGED)
    down = weight(RHO_ICE, BOX_AREA)
    assert abs(up - down) / down < 1e-12, (up, down)

    # Held under, the same ice is pushed up harder than it weighs; iron is not.
    assert buoyancy(BOX_AREA) > weight(RHO_ICE, BOX_AREA)
    assert buoyancy(BOX_AREA) < weight(RHO_IRON, BOX_AREA)

    _one_formula_for_every_object()
    _the_body_finds_its_own_level()
    _the_surface_and_the_depth_agree()

    # Every shape in the hollowing has the same point count, which is what
    # lets the box hollow itself a sampled step at a time instead of cutting.
    assert len(outline((0, 0))) == POINTS

    # Rounding does not move the outline off its own box: every point of a
    # rounded rectangle is still inside it, and the sides still reach the edge.
    pts = outline((0.0, 0.0))
    assert max(abs(x) for x, _ in pts) <= BOX_W / 2 + 1e-9
    assert max(abs(y) for _, y in pts) <= BOX_H / 2 + 1e-9
    assert max(abs(x) for x, _ in pts) > BOX_W / 2 - 1e-9, "not rounded at all"

    # The box fits the tank with water either side, the water never goes over
    # the rim, and a sunk box still has water over the top of it.
    assert BOX_W < TANK_W - 200, "no water either side of the box"
    assert BOX_W > 0.35 * TANK_W, "the box is lost in the tank"
    assert water_level(BOX_AREA) > TANK[1] + 20, "water over the rim"
    assert FLOOR - BOX_H > water_level(BOX_AREA) + 20, "too shallow to sink in"

    # The hollow box floats, and where it floats is what its density asks for.
    afloat = settles(HOLLOW_RHO)
    assert FLOOR - afloat > 40, "the hollow box should float clear of the floor"
    assert abs(displaced(afloat, BOX_W, BOX_H) * RHO_WATER
               - metal(WALL) * RHO_IRON) < 1e-6, "displaces the iron it holds"

    # The walls have to be worth drawing, and the iron worth calling gone.
    assert 4.0 <= WALL <= 12.0, ("walls you cannot see", WALL)
    assert LEFT_OF_IT < 0.13, ("still too much iron to float", LEFT_OF_IT)

    # The longest arrow lands inside the tank, and the push arrow on the
    # floating box stops short of the title.
    deepest = floating_bottom(1.0, BOX_W, BOX_H)
    assert deepest - BOX_H / 2 + ARROW_CAP < TANK[3] + 8, "weight arrow escapes"
    assert afloat - BOX_H / 2 - ARROW_CAP > 100.0, "push arrow reaches the title"
    return True


def _one_formula_for_every_object():
    """Average density, over the outside, for every body in the film."""
    # Solid is solid: at `SOLID` the metal is the whole volume, so the formula
    # gives back exactly the density of the stuff.
    assert abs(density(RHO_WATER, SOLID) - RHO_WATER) < 1e-9
    assert abs(density(RHO_ICE, SOLID) - RHO_ICE) < 1e-9
    assert abs(density(RHO_IRON, SOLID) - RHO_IRON) < 1e-9

    # And hollowing only ever takes metal away.
    was = None
    for i in range(41):
        wall = thinning(i / 40)
        rho = density(RHO_IRON, wall)
        assert was is None or rho <= was + 1e-6, "hollowing added metal"
        was = rho
    assert abs(density(RHO_IRON, WALL) - HOLLOW_RHO) < 1e-9
    assert HOLLOW_RHO < RHO_WATER, ("it has to float", HOLLOW_RHO)


def _the_body_finds_its_own_level():
    """Nothing is told to rise. It lifts off when enough iron has gone."""
    assert settles(RHO_IRON) == FLOOR, "solid iron must sink"
    assert settles(HOLLOW_RHO) < FLOOR, "the hollow box must float"
    assert abs(settles(RHO_ICE)
               - floating_bottom(ICE_SUBMERGED, BOX_W, BOX_H)) < 1e-9
    assert abs(settles(RHO_WATER)
               - floating_bottom(1.0, BOX_W, BOX_H)) < 1e-9

    # It leaves the floor exactly once and never settles back: the walls only
    # thin, so the moment it can carry itself it keeps carrying itself.
    was, lifts = FLOOR, 0
    for i in range(1, 121):
        where = settles(density(RHO_IRON, thinning(i / 120)))
        if was >= FLOOR > where:
            lifts += 1
        assert where <= was + 1e-9, "it sank back down"
        was = where
    assert lifts == 1, ("it should lift off exactly once", lifts)

    # And the overlap point is before the crossing, not after it.
    assert density(RHO_IRON, thinning(LIFTS_AT)) > RHO_WATER, "overlap too late"


def _the_surface_and_the_depth_agree():
    """The waterline and how deep things sit answer each other, everywhere."""
    for bottom in (REST_LEVEL - 40, REST_LEVEL + BOX_H / 2,
                   REST_LEVEL + BOX_H, FLOOR):
        level, sub = surface(bottom, BOX_W, BOX_H)
        assert abs(level - water_level(BOX_AREA * sub)) < 1e-9, bottom
        assert abs(rise(bottom, BOX_W, BOX_H)
                   - BOX_AREA * sub / TANK_W) < 1e-9, bottom
        if 0.0 < sub < 1.0:
            assert abs((bottom - level) / BOX_H - sub) < 1e-9, bottom

    # A box fully under raises it by its own volume over the width.
    assert abs(rise(FLOOR, BOX_W, BOX_H) - BOX_AREA / TANK_W) < 1e-9

    # At the floating depth the push up equals the weight down.
    bottom = floating_bottom(ICE_SUBMERGED, BOX_W, BOX_H)
    _, sub = surface(bottom, BOX_W, BOX_H)
    assert abs(sub - ICE_SUBMERGED) < 1e-9, sub
    assert abs(buoyancy(BOX_AREA * sub) - weight(RHO_ICE, BOX_AREA)) < 1e-6


assert _the_physics_holds()
