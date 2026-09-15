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


_FOUND = _pairings()
AT = max(_FOUND, key=lambda w: (_agrees(w), tuple(sorted(w.items()))))
AT = [AT[i] for i in range(54)]          # place -> sticker
PLACE = [0] * 54                         # sticker -> place
for _place, _sticker in enumerate(AT):
    PLACE[_sticker] = _place

# Which circle is which face's layer, and where its centre is on the page.
CIRCLE = {}
for _face in cube.FACES:
    _axis = [k for k in range(3) if geo.NORMAL[_face][k] != 0][0]
    _slot = geo.NORMAL[_face][_axis]
    _want = set(BAND[(_axis, _slot)])
    CIRCLE[_face] = next(c for c in RING if {AT[i] for i in RING[c]} == _want)


def _slide(face):
    """Three places along, or three back — which way this face's circle runs."""
    after = cube.turn(cube.solved(), face)
    ring = RING[CIRCLE[face]]
    step, = {(ring.index(PLACE[after[AT[p]]]) - ring.index(p)) % 12
             for p in ring}
    return 1 if step == 3 else -1


# Places are listed anticlockwise on the page, so this is which way round the
# dots travel — the same for every sticker the turn touches, ring or not.
SLIDE = {f: _slide(f) for f in cube.FACES}


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

    # A quarter turn slides one circle's twelve exactly three places along.
    for face in cube.FACES:
        after = cube.turn(cube.solved(), face)
        ring = RING[CIRCLE[face]]
        moved = {(ring.index(PLACE[after[AT[p]]]) - ring.index(p)) % 12
                 for p in ring}
        assert moved in ({3}, {9}), (face, moved)
    return True


assert _checks()
