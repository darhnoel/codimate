"""The physics and the geometry. No Codimate in this file.

Densities are real, and everything the picture does follows from them: how deep
a thing floats, how long each force arrow is, where the waterline sits. The
only numbers chosen by eye are the sizes of the tank and the box — and even the
ship's hull is solved, so that it holds exactly as much steel as the block it
was made from.
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

# One container, centred on the frame, with the notes column to its right.
TANK = (330.0, 170.0, 950.0, 620.0)              # left, top, right, bottom
TANK_W = TANK[2] - TANK[0]
REST_LEVEL = 392.0                               # with nothing in the tank


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
    """How far the surface has climbed — the displaced volume, made visible.

    This is the whole of the second scene in one number: the water goes up by
    exactly the volume that went in, spread over the width of the tank.
    """
    return REST_LEVEL - surface(bottom, w, h)[0]


def floating_bottom(fraction, w, h):
    """Where an object's underside sits when it floats `fraction` submerged."""
    return water_level(w * h * fraction) + fraction * h


# ---------------------------------------------------------------- the box

BOX_W, BOX_H = 130.0, 100.0
BOX_AREA = BOX_W * BOX_H                 # the "volume V" the film talks about

POINTS = 240            # per outline, fixed, so the box can morph into a hull


def _walk(corners, n):
    """`n` points evenly spaced around a closed outline, by distance.

    By distance rather than by corner, so two outlines with different corner
    counts still correspond point for point and can tween into one another.
    Sampling by corner would need the same number of corners, which a
    rectangle and a hull do not have.
    """
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


def box_outline(middle, w=BOX_W, h=BOX_H):
    """The block, as `POINTS` points clockwise from its top left corner."""
    x, y = middle[0] - w / 2, middle[1] - h / 2
    return _walk([(x, y), (x + w, y), (x + w, y + h), (x, y + h)], POINTS)


# ---------------------------------------------------------------- the ship

HULL_W, HULL_H = 520.0, 300.0    # narrower than the tank, so it fits in it
HULL_OUTER = HULL_W * HULL_H


def hull_thickness(steel_area=BOX_AREA, w=HULL_W, h=HULL_H):
    """Wall thickness for a hull holding exactly `steel_area` of steel.

    The point of the ship is that none of the steel goes away, so `t` solves

        w h - (w - 2t)(h - t) = steel_area

    Solved rather than chosen, so the hull cannot quietly gain or lose metal
    on its way to floating.
    """
    lo, hi = 0.0, min(w / 2, h)
    for _ in range(80):
        mid = 0.5 * (lo + hi)
        if w * h - (w - 2 * mid) * (h - mid) < steel_area:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


HULL_T = hull_thickness()


def hull_outline(middle, w=HULL_W, h=HULL_H, t=HULL_T):
    """The same steel, spread into an open vessel. Same point count as a box."""
    x, y = middle[0] - w / 2, middle[1] - h / 2
    return _walk([(x, y), (x + t, y), (x + t, y + h - t),
                  (x + w - t, y + h - t), (x + w - t, y), (x + w, y),
                  (x + w, y + h), (x, y + h)], POINTS)


def ship_average_density(steel_area=BOX_AREA, outer=HULL_OUTER):
    """Steel and air together, over the whole outer volume.

    This is the number that decides whether a ship floats — not the density of
    what it is made of. The hull is drawn to scale, so the ratio is the one on
    screen rather than a figure picked to make the point come out.
    """
    return RHO_STEEL * steel_area / outer


SHIP_RHO = ship_average_density()
SHIP_SUBMERGED = submerged_fraction(SHIP_RHO)
SPREAD = HULL_OUTER / BOX_AREA           # how much bigger the outside got


# ------------------------------------------------------------- the arrows

FORCE_PX = 120.0        # what a fully submerged box of water is drawn as
ARROW_CAP = 170.0       # beyond this an arrow runs out of the tank


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

    # The hull holds the block's steel, to a fraction of a pixel.
    held = HULL_W * HULL_H - (HULL_W - 2 * HULL_T) * (HULL_H - HULL_T)
    assert abs(held - BOX_AREA) < 1e-6, (held, BOX_AREA)

    # Spread that wide it floats, and the depth is computed, not chosen.
    assert RHO_WATER * 0.2 < SHIP_RHO < RHO_WATER * 0.9, SHIP_RHO
    assert 0.2 < SHIP_SUBMERGED < 0.9, SHIP_SUBMERGED

    # Every outline has the same point count, which is what lets the box
    # become a hull instead of cutting to one.
    assert len(box_outline((0, 0))) == len(hull_outline((0, 0))) == POINTS

    # The ship has to fit in the tank, and float in the water that is in it.
    assert HULL_W < TANK_W, (HULL_W, TANK_W)
    draft = SHIP_SUBMERGED * HULL_H
    assert draft < TANK[3] - REST_LEVEL, (draft, TANK[3] - REST_LEVEL)

    # The surface climbs by exactly the volume put in, over the tank's width,
    # and the depth it settles at is consistent with that same rise.
    for bottom in (REST_LEVEL - 40, REST_LEVEL + BOX_H / 2,
                   REST_LEVEL + BOX_H, REST_LEVEL + 300):
        level, sub = surface(bottom, BOX_W, BOX_H)
        assert abs(level - water_level(BOX_AREA * sub)) < 1e-9, bottom
        assert abs(rise(bottom, BOX_W, BOX_H)
                   - BOX_AREA * sub / TANK_W) < 1e-9, bottom
        if 0.0 < sub < 1.0:
            assert abs((bottom - level) / BOX_H - sub) < 1e-9, bottom

    # A box fully under raises it by its own volume over the width — small,
    # but the thing the bracket has to be able to show.
    assert abs(rise(REST_LEVEL + 300, BOX_W, BOX_H)
               - BOX_AREA / TANK_W) < 1e-9

    # The floating depth is the one the densities ask for, and there the push
    # up equals the weight down.
    bottom = floating_bottom(ICE_SUBMERGED, BOX_W, BOX_H)
    _, sub = surface(bottom, BOX_W, BOX_H)
    assert abs(sub - ICE_SUBMERGED) < 1e-9, sub
    assert abs(buoyancy(BOX_AREA * sub) - weight(RHO_ICE, BOX_AREA)) < 1e-6

    # The ship floats, fits the glass, and does not push the water over the
    # top of the tank on its way.
    ship = floating_bottom(SHIP_SUBMERGED, HULL_W, HULL_H)
    assert ship < TANK[3], (ship, TANK[3])
    assert surface(ship, HULL_W, HULL_H)[0] > TANK[1] + 20, "water over the rim"

    # The longest arrow still lands inside the tank rather than in the caption.
    deepest = floating_bottom(1.0, BOX_W, BOX_H)
    assert deepest - BOX_H / 2 + ARROW_CAP < TANK[3] + 8, "weight arrow escapes"
    return True


assert _the_physics_holds()
