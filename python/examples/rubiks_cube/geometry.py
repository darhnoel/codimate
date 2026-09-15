"""Where the 54 stickers are in space, and what a turn does to them.

A turn is a layer of the cube rotating 90 degrees about an axis. Sampled into
steps and handed to the Engine one at a time, the same way `dharma_wheel`
turns its wheel and `pendulum` swings — continuous motion in this library is
sampled, not guessed at between endpoints.

Because the sticker rotates rather than being repainted, nothing ever flies
across the picture and no sticker ever shows a colour that is not on a cube.
"""

import math

from cube import FACES, STRIPS, solved, turn

CELL = 58.0
MIDDLE = (905.0, 366.0)

_COS30, _SIN30 = math.cos(math.radians(30)), math.sin(math.radians(30))

# Each face: the corner to start from, a step per column, a step per row.
_PLAN = {
    "U": ((-1.5, 1.5, -1.5), (1, 0, 0), (0, 0, 1)),
    "R": ((1.5, 1.5, 1.5), (0, 0, -1), (0, -1, 0)),
    "F": ((-1.5, 1.5, 1.5), (1, 0, 0), (0, -1, 0)),
    "D": ((-1.5, -1.5, 1.5), (1, 0, 0), (0, 0, -1)),
    "L": ((-1.5, 1.5, -1.5), (0, 0, 1), (0, -1, 0)),
    "B": ((1.5, 1.5, -1.5), (-1, 0, 0), (0, -1, 0)),
}

# Outward normals, which are also the axes a turn spins about.
NORMAL = {"U": (0, 1, 0), "D": (0, -1, 0), "R": (1, 0, 0),
          "L": (-1, 0, 0), "F": (0, 0, 1), "B": (0, 0, -1)}

# Where the camera is. A sticker shows when its face points somewhat this way.
EYE = (1.0, 1.0, 1.0)


def corners(index):
    """The four corners of facelet `index`, in cube space."""
    face = FACES[index // 9]
    row, col = divmod(index % 9, 3)
    start, across, down = _PLAN[face]
    return [
        tuple(start[k] + (col + dc) * across[k] + (row + dr) * down[k]
              for k in range(3))
        for dc, dr in ((0, 0), (1, 0), (1, 1), (0, 1))
    ]


def layer(face):
    """Every facelet that moves when `face` turns: its own nine, and the
    twelve carried round from the faces beside it."""
    base = FACES.index(face) * 9
    moving = set(range(base, base + 9))
    for strip in STRIPS[face]:
        moving.update(strip)
    return moving


LAYER = {f: layer(f) for f in FACES}


def spin(point, axis, degrees):
    """Rodrigues' rotation — a point turned about an axis through the centre."""
    t = math.radians(degrees)
    c, s = math.cos(t), math.sin(t)
    ax, ay, az = axis
    x, y, z = point
    dot = ax * x + ay * y + az * z
    cx, cy, cz = ay * z - az * y, az * x - ax * z, ax * y - ay * x
    return (x * c + cx * s + ax * dot * (1 - c),
            y * c + cy * s + ay * dot * (1 - c),
            z * c + cz * s + az * dot * (1 - c))


def project(p):
    x, y, z = p
    return (MIDDLE[0] + (x - z) * _COS30 * CELL,
            MIDDLE[1] + ((x + z) * _SIN30 - y) * CELL)


def spun(index, turning=None, degrees=0.0):
    """Facelet `index` in space, part way through a turn of `turning`."""
    pts = corners(index)
    if turning and index in LAYER[turning]:
        # Negative because a clockwise turn seen from outside the face is a
        # left-handed rotation about the outward normal. Checked against the
        # model in `_agrees_with_the_model` rather than reasoned about twice.
        pts = [spin(p, NORMAL[turning], -degrees) for p in pts]
    return pts


def quad(index, turning=None, degrees=0.0):
    """Facelet `index` on screen, part way through a turn of `turning`."""
    return [project(p) for p in spun(index, turning, degrees)]


def depth(index, turning=None, degrees=0.0):
    """How near the camera this facelet is *now*. Painter's order needs the
    turned position, not the one it started from: half way through a turn the
    layer sticking out is in front of faces it was behind at rest."""
    pts = spun(index, turning, degrees)
    return sum(sum(a * b for a, b in zip(p, EYE)) for p in pts) / 4.0


def aim(index, turning=None, degrees=0.0):
    """Which way this sticker is pointing *now*."""
    normal = NORMAL[FACES[index // 9]]
    if turning and index in LAYER[turning]:
        normal = spin(normal, NORMAL[turning], -degrees)
    return normal


def facing(index, turning=None, degrees=0.0):
    """Is this sticker pointing at the camera?"""
    return sum(a * b for a, b in zip(aim(index, turning, degrees), EYE)) > 0.1


def _a_turn_is_cut_clean_from_what_stays():
    """What the drawing order leans on: a turning layer and the two that stay
    are separated by a plane, so one is wholly in front of the other and no
    sticker of either can wander between them.

    The plane is not where it first looks. A face's own nine reach an axis and
    a half out, but the twelve carried round come down the side and reach only
    half a cell — which is exactly where the middle layer's stickers stop. The
    turn does not move it, because spinning about an axis leaves how far along
    that axis a point sits alone.
    """
    for face in FACES:
        axis = NORMAL[face]

        def far(index, axis=axis):
            return [sum(a * b for a, b in zip(p, axis)) for p in corners(index)]

        turning = min(min(far(i)) for i in range(54) if i in LAYER[face])
        staying = max(max(far(i)) for i in range(54) if i not in LAYER[face])
        assert turning >= staying - 1e-9, (face, turning, staying)
    return True


def _shift(face, index):
    """How far round a facelet's own corner list a turn carries the sticker.

    Ninety degrees lands the four corners back on four corners — the same four,
    but not each on the one it started from. A sticker on the turning face is
    rotating in its own plane, so its corners come round by one. The reconciler
    tweens a polygon corner by corner, so handing it the destination facelet's
    list in the destination facelet's order asks it to spin the square in place
    on the way. Carrying this shift with the sticker is what stops that.
    """
    home = {s: f for f, s in enumerate(turn(solved(), face))}
    went, waits = quad(index, face, 90.0), quad(home[index])
    for k in range(4):
        if all(abs(a - b) < 1e-6
               for p, q in zip(went, waits[k:] + waits[:k])
               for a, b in zip(p, q)):
            return k
    raise AssertionError((face, index))


SHIFT = {f: [_shift(f, i) for i in range(54)] for f in FACES}


def _a_finished_turn_is_already_home():
    """Ninety degrees round lands a sticker exactly on the facelet the model
    hands it to — same four corners, in the same order once `SHIFT` is carried
    with it, to the pixel. It is what lets the view name its shapes after
    stickers and let a turn run straight into the scene after it with nothing
    left to travel, and nothing left to spin."""
    for face in FACES:
        home = {s: f for f, s in enumerate(turn(solved(), face))}
        for sticker in range(54):
            went = quad(sticker, face, 90.0)
            waits = quad(home[sticker])
            k = SHIFT[face][sticker]
            assert all(abs(a - b) < 1e-9
                       for p, q in zip(went, waits[k:] + waits[:k])
                       for a, b in zip(p, q)), (face, sticker)
    return True


assert _a_turn_is_cut_clean_from_what_stays()
assert _a_finished_turn_is_already_home()
