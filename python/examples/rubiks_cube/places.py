"""Which place in the drawing holds which sticker — worked out, not tabulated.

The drawing *is* the cube. Its nine circles carry twelve places each, every
place sits on exactly two circles, and walking any circle the other two
alternate in runs of three. That is the cube's nine layers: three slices on
each of three axes, twelve stickers wrapping round each slice, every sticker
on two of them. Nothing else has that shape, which is why the pairing below
can be found rather than guessed.

Once paired, a quarter turn slides the twelve places of one circle exactly
three positions along it. That is the whole idea the drawing encodes.
"""

import collections
import itertools
import math

import cube
import geometry as geo
import graph

_ON = 8.5          # a place counts as on a circle within this many pixels

# ---------------------------------------------------------------- the cube

_MID = [tuple(sum(c[k] for c in geo.corners(i)) / 4.0 for k in range(3))
        for i in range(54)]
_NORMAL = [geo.NORMAL[cube.FACES[i // 9]] for i in range(54)]

# A sticker belongs to the slice its cubie sits in, on each axis it does not
# point along. Its own axis holds it half a cell proud of the cube, so the
# normal is what says which axis to skip.
_SLICE = [tuple(round(_MID[i][k]) if _NORMAL[i][k] == 0 else None
                for k in range(3)) for i in range(54)]

LAYERS = [(axis, slot) for axis in range(3) for slot in (-1, 0, 1)]


def _band(axis, slot):
    """The twelve stickers wrapped round one slice, in the order they sit."""
    u, v = [k for k in range(3) if k != axis]
    here = [i for i in range(54) if _SLICE[i][axis] == slot]
    return sorted(here, key=lambda i: math.atan2(_MID[i][v], _MID[i][u]))


BAND = {k: _band(*k) for k in LAYERS}

# ------------------------------------------------------------- the drawing

_MEMBERS = collections.defaultdict(list)
for _i, (_x, _y) in enumerate(graph.AT):
    for _c, (_cx, _cy) in enumerate(graph.CENTRES):
        _d = math.hypot(_x - _cx, _y - _cy)
        for _r, _radius in enumerate(graph.RADII):
            if abs(_d - _radius) < _ON:
                _MEMBERS[(_c, _r)].append(_i)


def _ring(circle):
    cx, cy = graph.CENTRES[circle[0]]
    return sorted(_MEMBERS[circle],
                  key=lambda i: math.atan2(graph.AT[i][1] - cy,
                                           graph.AT[i][0] - cx))


RING = {c: _ring(c) for c in _MEMBERS}

_CIRCLES_OF = collections.defaultdict(list)
for _c, _places in RING.items():
    for _i in _places:
        _CIRCLES_OF[_i].append(_c)

_LAYERS_OF = collections.defaultdict(list)
for _k, _stickers in BAND.items():
    for _i in _stickers:
        _LAYERS_OF[_i].append(_k)

# ------------------------------------------------------------ the pairing


def _grow(aligned):
    """Walk out from an aligned circle. Each place it fixes fixes the other
    circle that place is on, until all nine agree or one contradicts."""
    where, seen = {}, set()
    todo = list(aligned)
    while todo:
        circle = todo.pop()
        if circle in seen:
            continue
        seen.add(circle)
        layer, offset, step = aligned[circle]
        for index, place in enumerate(RING[circle]):
            sticker = BAND[layer][(step * index + offset) % 12]
            if where.setdefault(place, sticker) != sticker:
                return None
            other = [c for c in _CIRCLES_OF[place] if c != circle][0]
            beside = [k for k in _LAYERS_OF[sticker] if k != layer][0]
            i = RING[other].index(place)
            j = BAND[beside].index(sticker)
            if other in aligned:
                l2, o2, s2 = aligned[other]
                if l2 != beside or (s2 * i + o2) % 12 != j:
                    return None
            else:
                for turn in (1, -1):
                    branch = dict(aligned)
                    branch[other] = (beside, (j - turn * i) % 12, turn)
                    got = _grow(branch)
                    if got:
                        return got
                return None
    return where if len(set(where.values())) == 54 else None


def _pairings():
    seen, out = set(), []
    first = next(iter(RING))
    for layer in LAYERS:
        for offset in range(12):
            for step in (1, -1):
                got = _grow({first: (layer, offset, step)})
                key = got and tuple(sorted(got.items()))
                if got and key not in seen:
                    seen.add(key)
                    out.append(got)
    return out


def _agrees(where):
    """How many places already show the colour the drawing was traced with,
    on a cube that has not been touched. Only a tie-break — it picks one of
    the drawing's symmetric orientations so the pairing is the same every run.
    """
    return sum(cube.colour_of(where[i]) == graph.COLOUR[i] for i in range(54))


def _read(at):
    """Given a pairing, which circle is each face's layer and which way it runs.

    Places are listed by growing angle and the page's y points down, so `+1`
    means the twelve travel clockwise to the eye.
    """
    place_of = [0] * 54
    for place, sticker in enumerate(at):
        place_of[sticker] = place
    circle, slide = {}, {}
    for face in cube.FACES:
        axis = [k for k in range(3) if geo.NORMAL[face][k] != 0][0]
        want = set(BAND[(axis, geo.NORMAL[face][axis])])
        found = next(c for c in RING if {at[i] for i in RING[c]} == want)
        ring = RING[found]
        after = cube.turn(cube.solved(), face)
        step, = {(ring.index(place_of[after[at[p]]]) - ring.index(p)) % 12
                 for p in ring}
        circle[face], slide[face] = found, 1 if step == 3 else -1
    return place_of, circle, slide


def _on_screen(face):
    """Which way this layer looks to turn on the page when it turns clockwise.

    The cube is drawn from a corner, so half the faces point away and their
    clockwise reads as anticlockwise. That is the thing the pairing has to
    agree with, and it is why colour alone could not choose one.
    """
    axis = geo.NORMAL[face]
    hub = geo.project(axis)
    sweep = []
    for i in geo.LAYER[face]:
        if i // 9 == cube.FACES.index(face):
            continue
        mid = [sum(c[k] for c in geo.corners(i)) / 4.0 for k in range(3)]
        a, b = geo.project(mid), geo.project(geo.spin(mid, axis, -10.0))
        sweep.append(math.remainder(
            math.atan2(b[1] - hub[1], b[0] - hub[0])
            - math.atan2(a[1] - hub[1], a[0] - hub[0]), math.tau))
    return 1 if sum(sweep) > 0 else -1


SCREEN = {f: _on_screen(f) for f in cube.FACES}


def _keeps_the_turn(where):
    """How many of the six circles turn the way the cube beside them does.

    Half the pairings are mirror images, and a mirror passes every structural
    test there is — twelve to a circle, two circles to a place, three places to
    a quarter turn. Only the direction tells them apart, so this has to choose
    before colour does.
    """
    _, _, slide = _read([where[i] for i in range(54)])
    return sum(slide[f] == SCREEN[f] for f in cube.FACES)


_FOUND = _pairings()
_BEST = max(_FOUND, key=lambda w: (_keeps_the_turn(w), _agrees(w),
                                   tuple(sorted(w.items()))))
assert _keeps_the_turn(_BEST) == 6, _keeps_the_turn(_BEST)

AT = [_BEST[i] for i in range(54)]       # place -> sticker
PLACE, CIRCLE, SLIDE = _read(AT)         # sticker -> place, and the two tables

# The dot a face turns about, and which way its other eight go round it. The
# twelve on the circle are only two thirds of a move; the other nine sit in a
# rosette about the face's middle sticker, which a turn leaves exactly where
# it is.
MIDDLE = {f: PLACE[cube.FACES.index(f) * 9 + 4] for f in cube.FACES}


def _rosette(face):
    """Which way a face's other eight go round its middle dot."""
    base = cube.FACES.index(face) * 9
    hub = graph.AT[MIDDLE[face]]
    home = {s: f for f, s in enumerate(cube.turn(cube.solved(), face))}
    ways = []
    for sticker in range(base, base + 9):
        if sticker == base + 4:
            continue
        here, there = PLACE[sticker], PLACE[home[sticker]]
        ways.append(math.remainder(
            math.atan2(graph.AT[there][1] - hub[1], graph.AT[there][0] - hub[0])
            - math.atan2(graph.AT[here][1] - hub[1], graph.AT[here][0] - hub[0]),
            math.tau))
    assert len({w > 0 for w in ways}) == 1, (face, ways)
    return 1 if ways[0] > 0 else -1


SPIN = {f: _rosette(f) for f in cube.FACES}


def centre(face):
    """The middle of the circle that turns when `face` turns."""
    return graph.CENTRES[CIRCLE[face][0]]


def _checks():
    assert all(len(v) == 12 for v in BAND.values())
    assert all(len(v) == 12 for v in _MEMBERS.values()), \
        {k: len(v) for k, v in _MEMBERS.items()}
    assert all(len(v) == 2 for v in _CIRCLES_OF.values())

    # Walking a circle, the other two alternate in runs of three.
    for circle, places in RING.items():
        beside = [[c for c in _CIRCLES_OF[i] if c != circle][0][0]
                  for i in places]
        cut = next(i for i in range(12) if beside[i] != beside[i - 1])
        rolled = beside[cut:] + beside[:cut]
        runs = [len(list(g)) for _, g in itertools.groupby(rolled)]
        assert runs == [3, 3, 3, 3], (circle, beside)

    # A quarter turn slides one circle's twelve exactly three places along,
    # and swings the face's other eight about its middle dot, which stays.
    for face in cube.FACES:
        after = cube.turn(cube.solved(), face)
        ring = RING[CIRCLE[face]]
        moved = {(ring.index(PLACE[after[AT[p]]]) - ring.index(p)) % 12
                 for p in ring}
        assert moved in ({3}, {9}), (face, moved)

        base = cube.FACES.index(face) * 9
        assert after[AT[MIDDLE[face]]] == base + 4, face
        assert SLIDE[face] == SCREEN[face], face
    return True


assert _checks()
