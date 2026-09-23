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
RHO_STEEL = 7850.0
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

# One tank for the whole film. An earlier cut used a small beaker for the box
# and grew it into a basin for the ship, so that the box would look its size —
# but a container cannot inflate, and watching the glass stretch was worse than
# the problem it solved. One tank costs the box some presence and buys back
# every motion in the film being one that could actually happen.
TANK = (405.0, 96.0, 875.0, 648.0)               # left, top, right, bottom
TANK_W = TANK[2] - TANK[0]
REST_LEVEL = 346.0                               # with nothing in the tank
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


def settles(rho, w, h):
    """Where a body with outside `w` x `h` comes to rest, on its own.

    This is what the last third of the film runs on. The hull is never told to
    rise: it is asked where it belongs, and as it opens out the answer changes
    from "on the floor" to "floating" at the moment its outside grows big
    enough to carry the metal. The lift-off is a consequence, not a cue.

        rho_water * outside  >  rho * V   ->  it can float, and does
        otherwise                         ->  the floor
    """
    most = buoyancy(w * h)                         # fully under, at most
    load = weight(rho, BOX_AREA)                   # the metal never changes
    if most < load:
        return FLOOR
    # Equality is the neutral case, and it is not a special one: the fraction
    # comes out at 1.0 and the body hangs exactly full under, which is what
    # the box of water does.
    return floating_bottom(load / most, w, h)


# ----------------------------------------------------------------- the box

BOX_W, BOX_H = 125.0, 80.0
BOX_AREA = BOX_W * BOX_H                 # the "volume V" the film talks about

POINTS = 240            # per outline, fixed, so the box can morph into a hull
CORNER = 11.0           # every corner is arced, on the box and on the hull


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


# ---------------------------------------------------------------- the ship

HULL_W, HULL_H = 330.0, 277.0    # narrow enough that water shows either side
HULL_OUTER = HULL_W * HULL_H


def thickness(w, h, steel_area=BOX_AREA):
    """Wall thickness that holds exactly `steel_area` inside `w` x `h`.

    Solved at every instant, not once for the finished hull, and that is what
    makes the reshape honest. A solid block cannot simply widen — that would
    multiply the metal ninefold, which is the one thing this section claims
    does not happen. So as the outside grows the walls thin, with

        w h - (w - 2t)(h - t) = steel_area

    holding all the way from the block, where the walls meet in the middle and
    there is no cavity at all, to the hull.
    """
    lo, hi = 0.0, min(w / 2, h)
    for _ in range(80):
        mid = 0.5 * (lo + hi)
        if w * h - (w - 2 * mid) * (h - mid) < steel_area:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


HULL_T = thickness(HULL_W, HULL_H)


def reshaping(along):
    """The outside of the body, `along` the way from block to hull."""
    return (BOX_W + (HULL_W - BOX_W) * along,
            BOX_H + (HULL_H - BOX_H) * along)


def _lift_off():
    """How far into the reshape the hull first carries its own weight.

    It is always near the end: the outside has to reach `rho_steel/rho_water`
    times the metal's area, which is most of the way to the finished hull. The
    film uses it to overlap the last of the opening with the rise, so the open
    hull is never shown at rest under water.
    """
    lo, hi = 0.0, 1.0
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        if settles(RHO_STEEL, *reshaping(mid)) >= FLOOR:
            lo = mid
        else:
            hi = mid
    return max(0.0, lo - 0.06)


def outline(middle, w, h):
    """The body at any point in its reshape, as `POINTS` points.

    One family, block to hull, with the cavity opening as the walls thin.
    There is no separate spreading and hollowing: they are the same motion,
    because they have to be for the metal to stay the same metal.
    """
    x, y = middle[0] - w / 2, middle[1] - h / 2
    t = thickness(w, h)
    if w - 2 * t < 2.0:                            # walls still meeting: solid
        return _walk([(x, y), (x + w, y), (x + w, y + h), (x, y + h)], POINTS)
    return _walk([(x, y), (x + t, y), (x + t, y + h - t),
                  (x + w - t, y + h - t), (x + w - t, y), (x + w, y),
                  (x + w, y + h), (x, y + h)], POINTS)


def cavity(middle, w, h):
    """The air inside the hull: left, top, right, bottom in pixels.

    Drawn over the water, because the Engine has no clipping — without it the
    translucent water runs straight through the hull and the ship looks
    swamped, which is the opposite of what floats it.
    """
    x, y = middle[0] - w / 2, middle[1] - h / 2
    t = thickness(w, h)
    return (x + t, y, x + w - t, y + h - t)


def ship_average_density(steel_area=BOX_AREA, outer=HULL_OUTER):
    """Steel and air together, over the whole outer volume.

    This is the number that decides whether a ship floats — not the density of
    what it is made of.
    """
    return RHO_STEEL * steel_area / outer


LIFTS_AT = _lift_off()
SHIP_RHO = ship_average_density()
SHIP_SUBMERGED = submerged_fraction(SHIP_RHO)
SPREAD = HULL_OUTER / BOX_AREA           # how much bigger the outside got


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
    assert submerged_fraction(RHO_STEEL) == 1.0, "steel cannot float as a block"
    assert submerged_fraction(RHO_WATER) == 1.0, "a water box is neutral"

    # At the floating depth, up equals down. That is the whole principle.
    up = buoyancy(BOX_AREA * ICE_SUBMERGED)
    down = weight(RHO_ICE, BOX_AREA)
    assert abs(up - down) / down < 1e-12, (up, down)

    # Held under, the same ice is pushed up harder than it weighs; steel is not.
    assert buoyancy(BOX_AREA) > weight(RHO_ICE, BOX_AREA)
    assert buoyancy(BOX_AREA) < weight(RHO_STEEL, BOX_AREA)

    _the_reshape_conserves_steel()
    _the_body_finds_its_own_level()
    _the_surface_and_the_depth_agree()

    # Spread that wide it floats, and the depth is computed, not chosen.
    assert RHO_WATER * 0.2 < SHIP_RHO < RHO_WATER * 0.95, SHIP_RHO

    # Every shape in the reshape has the same point count, which is what lets
    # the block become a hull one sampled step at a time instead of cutting.
    assert len({len(outline((0, 0), *reshaping(i / 8))) for i in range(9)}) == 1

    # Rounding does not move the outline off its own box: every point of a
    # rounded rectangle is still inside it, and the sides still reach the edge.
    pts = outline((0.0, 0.0), BOX_W, BOX_H)
    assert max(abs(x) for x, _ in pts) <= BOX_W / 2 + 1e-9
    assert max(abs(y) for _, y in pts) <= BOX_H / 2 + 1e-9
    assert max(abs(x) for x, _ in pts) > BOX_W / 2 - 1e-9, "not rounded at all"

    # The ship fits the glass with water either side, and the water never goes
    # over the rim — not even at the deepest moment of the whole film, which is
    # the fully opened hull still sitting on the floor.
    assert HULL_W < TANK_W - 120, "no water either side of the ship"
    assert water_level(HULL_OUTER) > TANK[1] + 20, "water over the rim"
    ship = settles(RHO_STEEL, HULL_W, HULL_H)
    assert FLOOR - ship > 40, "the ship should float clear of the floor"
    assert abs(displaced(ship, HULL_W, HULL_H)
               - BOX_AREA * RHO_STEEL / RHO_WATER) < 1e-6, "displaces its steel"

    # The longest arrow lands inside the tank rather than in the caption, and
    # the ship's push arrow stops short of the title.
    deepest = floating_bottom(1.0, BOX_W, BOX_H)
    assert deepest - BOX_H / 2 + ARROW_CAP < TANK[3] + 8, "weight arrow escapes"
    assert ship - HULL_H / 2 - ARROW_CAP > 100.0, "push arrow reaches the title"
    return True


def _the_reshape_conserves_steel():
    """The metal is the same metal at every step of the reshape."""
    for i in range(41):
        w, h = reshaping(i / 40)
        t = thickness(w, h)
        held = w * h - max(w - 2 * t, 0.0) * max(h - t, 0.0)
        assert abs(held - BOX_AREA) < 1e-6, (i / 40, held, BOX_AREA)

    # It starts solid — walls meeting in the middle — and ends thin-walled.
    assert BOX_W - 2 * thickness(BOX_W, BOX_H) < 1e-6, "the block has a hole"
    assert HULL_T < 0.06 * HULL_W, ("hull walls too thick", HULL_T)


def _the_body_finds_its_own_level():
    """Nothing is told to rise. It lifts off when the outside is big enough."""
    assert settles(RHO_STEEL, BOX_W, BOX_H) == FLOOR, "a block must sink"
    assert settles(RHO_STEEL, HULL_W, HULL_H) < FLOOR, "a hull must float"
    assert abs(settles(RHO_ICE, BOX_W, BOX_H)
               - floating_bottom(ICE_SUBMERGED, BOX_W, BOX_H)) < 1e-9
    assert abs(settles(RHO_WATER, BOX_W, BOX_H)
               - floating_bottom(1.0, BOX_W, BOX_H)) < 1e-9

    # It leaves the floor exactly once and never settles back: the outside
    # only grows, so the moment it can carry itself it keeps carrying itself.
    was, lifts = FLOOR, 0
    for i in range(1, 121):
        where = settles(RHO_STEEL, *reshaping(i / 120))
        if was >= FLOOR > where:
            lifts += 1
        assert where <= was + 1e-9, "it sank back down"
        was = where
    assert lifts == 1, ("it should lift off exactly once", lifts)


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

    # A box fully under raises it by its own volume over the width — small,
    # but the thing the bracket has to be able to show.
    assert abs(rise(FLOOR, BOX_W, BOX_H) - BOX_AREA / TANK_W) < 1e-9

    # At the floating depth the push up equals the weight down.
    bottom = floating_bottom(ICE_SUBMERGED, BOX_W, BOX_H)
    _, sub = surface(bottom, BOX_W, BOX_H)
    assert abs(sub - ICE_SUBMERGED) < 1e-9, sub
    assert abs(buoyancy(BOX_AREA * sub) - weight(RHO_ICE, BOX_AREA)) < 1e-6

    # A sunk box still has water over the top of it.
    assert FLOOR - BOX_H > water_level(BOX_AREA) + 8, "too shallow"


assert _the_physics_holds()
