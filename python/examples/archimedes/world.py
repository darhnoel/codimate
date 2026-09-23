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


# --------------------------------------------------------------- the pools

# Two containers, not one, and the reason is arithmetic rather than taste.
#
# A hull has to enclose about nine times the block's area before steel and air
# together come out lighter than water, and the tank has to be wider than the
# hull and deeper than its draft. So a tank sized for the ship is roughly five
# times the box in every direction — which leaves the box looking lost in a
# column of water it never reaches.
#
# Nothing in Archimedes cares what the water is held in. So the first eight
# sections use a beaker the box actually fills, and the tank *grows* into a
# basin when the ship is built. The growth is sampled like everything else, so
# it reads as the setup being scaled up rather than as a cut.
#
# A pool is `(left, top, right, bottom, rest)` — the glass, and where the
# surface sits with nothing in it.
BEAKER = (510.0, 210.0, 770.0, 560.0, 380.0)
BASIN = (405.0, 150.0, 875.0, 600.0, 344.0)

POOLS = (BEAKER, BASIN)


def width(pool):
    return pool[2] - pool[0]


def floor(pool):
    """Where a sunk thing comes to rest — on the glass, not above it."""
    return pool[3] - 1.5


def water_level(displaced_area, pool):
    """Where the surface sits once something has pushed water aside.

    A tank this wide gains `area / width` of depth for every bit of volume put
    into it, which is the whole of "the water went up because the box went in".
    """
    return pool[4] - displaced_area / width(pool)


def surface(bottom, w, h, pool):
    """Where the water settles, and how much of the object is under it.

    The two answer each other: the object pushes the surface up, and the
    higher surface swallows more of the object. For a straight sided tank
    that is one equation rather than a search —

        L = REST - w (bottom - L) / WIDTH

    solved for L. Driving the level directly and reading the depth off it
    would let the two drift apart the moment anything moved.
    """
    rest, span = pool[4], width(pool)
    if bottom <= rest:                             # still clear of the water
        return rest, 0.0
    sunk = water_level(w * h, pool)                # fully under
    if bottom - h >= sunk:
        return sunk, 1.0
    level = (rest - w * bottom / span) / (1.0 - w / span)
    return level, max(0.0, min(1.0, (bottom - level) / h))


def displaced(bottom, w, h, pool):
    """The volume pushed aside: the part of the object under the waterline."""
    return w * h * surface(bottom, w, h, pool)[1]


def rise(bottom, w, h, pool):
    """How far the surface has climbed — the displaced volume, made visible.

    This is the whole of the second scene in one number: the water goes up by
    exactly the volume that went in, spread over the width of the tank.
    """
    return pool[4] - surface(bottom, w, h, pool)[0]


def floating_bottom(fraction, w, h, pool):
    """Where an object's underside sits when it floats `fraction` submerged."""
    return water_level(w * h * fraction, pool) + fraction * h


# ---------------------------------------------------------------- the box

BOX_W, BOX_H = 120.0, 80.0
BOX_AREA = BOX_W * BOX_H                 # the "volume V" the film talks about

POINTS = 240            # per outline, fixed, so the box can morph into a hull
CORNER = 12.0           # every corner is arced, on the box and on the hull


def _round_corners(corners, radius):
    """Replace each corner with a short arc, keeping the shape closed.

    The body has to be a polygon rather than a rounded rect, because only a
    polygon can morph into the hull. So the rounding is put into the outline
    itself: each corner becomes a few points on a circle of `radius`, clamped
    so a short side cannot be eaten from both ends at once.
    """
    n = len(corners)
    out = []
    for i, here in enumerate(corners):
        before, after = corners[i - 1], corners[(i + 1) % n]
        arms = []
        for other in (before, after):
            run = math.dist(here, other)
            r = min(radius, run / 2.0)
            arms.append(((here[0] + (other[0] - here[0]) * r / run,
                          here[1] + (other[1] - here[1]) * r / run), r))
        (a, ra), (b, rb) = arms
        r = min(ra, rb)
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

HULL_W, HULL_H = 340.0, 270.0    # narrow enough that water shows either side
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


def outline(middle, w, h, notch=0.0):
    """The body at any point in its reshape, as `POINTS` points.

    `notch` is how far the cavity has been excavated down from the top: zero
    is a solid block, `h - HULL_T` is the finished hull. One family rather
    than two shapes, because the morph is *sampled* — handed over a step at a
    time — and a step needs a shape in between to be a step towards.
    """
    x, y = middle[0] - w / 2, middle[1] - h / 2
    t = HULL_T
    if notch < 1.0 or w <= 2 * t:
        return _walk([(x, y), (x + w, y), (x + w, y + h), (x, y + h)], POINTS)
    notch = min(notch, h - t)
    return _walk([(x, y), (x + t, y), (x + t, y + notch),
                  (x + w - t, y + notch), (x + w - t, y), (x + w, y),
                  (x + w, y + h), (x, y + h)], POINTS)


def cavity(middle, w, h, notch):
    """The dry air inside the hull: left, top, right, bottom in pixels.

    Drawn over the water, because the Engine has no clipping — without it the
    translucent water runs straight through the hull and the ship looks
    swamped, which is the opposite of what floats it.
    """
    x, y = middle[0] - w / 2, middle[1] - h / 2
    return (x + HULL_T, y, x + w - HULL_T, y + min(notch, h - HULL_T))


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

    # The hull holds the block's steel, to a fraction of a pixel.
    held = HULL_W * HULL_H - (HULL_W - 2 * HULL_T) * (HULL_H - HULL_T)
    assert abs(held - BOX_AREA) < 1e-6, (held, BOX_AREA)

    # Spread that wide it floats, and the depth is computed, not chosen.
    assert RHO_WATER * 0.2 < SHIP_RHO < RHO_WATER * 0.9, SHIP_RHO

    # Every shape in the reshape has the same point count, which is what lets
    # the box become a hull one sampled step at a time instead of cutting.
    counts = {len(outline((0, 0), BOX_W, BOX_H)),
              len(outline((0, 0), HULL_W, HULL_H)),
              len(outline((0, 0), HULL_W, HULL_H, 40.0)),
              len(outline((0, 0), HULL_W, HULL_H, HULL_H - HULL_T))}
    assert counts == {POINTS}, counts

    # Rounding does not move the outline off its own box: every point of a
    # rounded rectangle is still inside it, and its corners are pulled in by
    # no more than the radius.
    pts = outline((0.0, 0.0), BOX_W, BOX_H)
    assert max(abs(x) for x, _ in pts) <= BOX_W / 2 + 1e-9
    assert max(abs(y) for _, y in pts) <= BOX_H / 2 + 1e-9
    assert max(abs(x) for x, _ in pts) > BOX_W / 2 - 1e-9, "not rounded at all"

    # Both pools are centred on the same line, so growing from one to the
    # other is the tank getting bigger rather than the tank sliding sideways.
    assert BEAKER[0] + BEAKER[2] == BASIN[0] + BASIN[2]

    # The box has real presence in the beaker — the whole reason there are two.
    assert BOX_W > 0.4 * width(BEAKER), "the box is lost in the beaker"

    _the_pool_holds(BEAKER, BOX_W, BOX_H)
    _the_pool_holds(BASIN, BOX_W, BOX_H)

    # The ship has to fit the basin, float in the water that is in it, and not
    # push that water over the rim on the way.
    assert HULL_W < width(BASIN) - 120, "no water either side of the ship"
    draft = SHIP_SUBMERGED * HULL_H
    assert draft < HULL_H, "the ship floats lower than it is tall"
    assert draft < 0.9 * (BASIN[3] - BASIN[4]), (draft, BASIN)
    ship = floating_bottom(SHIP_SUBMERGED, HULL_W, HULL_H, BASIN)
    assert ship < floor(BASIN), (ship, floor(BASIN))
    assert surface(ship, HULL_W, HULL_H, BASIN)[0] > BASIN[1] + 20, "over the rim"

    # It displaces its own steel — the one number the ship scene rests on.
    assert abs(displaced(ship, HULL_W, HULL_H, BASIN)
               - BOX_AREA * RHO_STEEL / RHO_WATER) < 1e-6

    # The longest arrow lands inside the tank rather than in the caption, and
    # the ship's push arrow stops short of the title.
    deepest = floating_bottom(1.0, BOX_W, BOX_H, BEAKER)
    assert deepest - BOX_H / 2 + ARROW_CAP < BEAKER[3] + 8, "weight arrow escapes"
    assert ship - HULL_H / 2 - ARROW_CAP > 100.0, "push arrow reaches the title"
    return True


def _the_pool_holds(pool, w, h):
    """The surface and the depth agree with each other, in either container."""
    rest, span = pool[4], width(pool)
    for bottom in (rest - 40, rest + h / 2, rest + h, floor(pool)):
        level, sub = surface(bottom, w, h, pool)
        assert abs(level - water_level(w * h * sub, pool)) < 1e-9, bottom
        assert abs(rise(bottom, w, h, pool) - w * h * sub / span) < 1e-9, bottom
        if 0.0 < sub < 1.0:
            assert abs((bottom - level) / h - sub) < 1e-9, bottom

    # A box fully under raises it by its own volume over the width — small,
    # but the thing the bracket has to be able to show.
    assert abs(rise(floor(pool), w, h, pool) - w * h / span) < 1e-9

    # The floating depth is the one the densities ask for, and there the push
    # up equals the weight down.
    bottom = floating_bottom(ICE_SUBMERGED, w, h, pool)
    _, sub = surface(bottom, w, h, pool)
    assert abs(sub - ICE_SUBMERGED) < 1e-9, sub
    assert abs(buoyancy(w * h * sub) - weight(RHO_ICE, w * h)) < 1e-6

    # Nothing overflows, and a sunk box still has water over the top of it.
    assert water_level(w * h, pool) > pool[1] + 12, ("overflow", pool)
    assert floor(pool) - h > water_level(w * h, pool) + 8, ("too shallow", pool)


assert _the_physics_holds()
