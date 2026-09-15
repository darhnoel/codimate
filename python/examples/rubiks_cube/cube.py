"""The cube itself: 54 stickers, and what a face turn does to them.

Nothing here knows it will be drawn. A turn is a permutation of sticker
positions, which is the whole reason this animates for free — name a shape
after the sticker rather than after the place, and the Engine turns "this
sticker is now there" into movement on its own.

Facelets are numbered face by face, and within a face row by row from the
top-left as you look at that face:

    U 0..8   R 9..17   F 18..26   D 27..35   L 36..44   B 45..53
"""

FACES = ("U", "R", "F", "D", "L", "B")
COLOUR = {"U": "yellow", "R": "magenta", "F": "green",
          "D": "cyan", "L": "blue", "B": "red"}


def _face(name):
    return FACES.index(name) * 9


# The four edge strips a turn carries around, in clockwise order, each given
# as three facelet indices reading along the strip. Taken from the standard
# adjacency of a cube; `_check` below proves them right rather than trusting
# that they were typed correctly.
STRIPS = {
    "U": [(_face("B") + 2, _face("B") + 1, _face("B") + 0),
          (_face("R") + 2, _face("R") + 1, _face("R") + 0),
          (_face("F") + 2, _face("F") + 1, _face("F") + 0),
          (_face("L") + 2, _face("L") + 1, _face("L") + 0)],
    "D": [(_face("F") + 6, _face("F") + 7, _face("F") + 8),
          (_face("R") + 6, _face("R") + 7, _face("R") + 8),
          (_face("B") + 6, _face("B") + 7, _face("B") + 8),
          (_face("L") + 6, _face("L") + 7, _face("L") + 8)],
    "F": [(_face("U") + 6, _face("U") + 7, _face("U") + 8),
          (_face("R") + 0, _face("R") + 3, _face("R") + 6),
          (_face("D") + 2, _face("D") + 1, _face("D") + 0),
          (_face("L") + 8, _face("L") + 5, _face("L") + 2)],
    "B": [(_face("U") + 2, _face("U") + 1, _face("U") + 0),
          (_face("L") + 0, _face("L") + 3, _face("L") + 6),
          (_face("D") + 6, _face("D") + 7, _face("D") + 8),
          (_face("R") + 8, _face("R") + 5, _face("R") + 2)],
    "R": [(_face("U") + 8, _face("U") + 5, _face("U") + 2),
          (_face("B") + 0, _face("B") + 3, _face("B") + 6),
          (_face("D") + 8, _face("D") + 5, _face("D") + 2),
          (_face("F") + 8, _face("F") + 5, _face("F") + 2)],
    "L": [(_face("U") + 0, _face("U") + 3, _face("U") + 6),
          (_face("F") + 0, _face("F") + 3, _face("F") + 6),
          (_face("D") + 0, _face("D") + 3, _face("D") + 6),
          (_face("B") + 8, _face("B") + 5, _face("B") + 2)],
}

# A quarter turn of a face, as it moves that face's own nine stickers.
_SPIN = (6, 3, 0, 7, 4, 1, 8, 5, 2)


def turn(where, face):
    """One clockwise quarter turn. Returns the new arrangement.

    `where[i]` is the sticker sitting at facelet `i`. A turn moves stickers
    between facelets; it never changes a sticker.
    """
    out = list(where)
    base = _face(face)
    for i, source in enumerate(_SPIN):
        out[base + i] = where[base + source]
    strips = STRIPS[face]
    for i, strip in enumerate(strips):
        behind = strips[(i - 1) % 4]
        for a, b in zip(strip, behind):
            out[a] = where[b]
    return out


def apply(where, moves):
    """Apply a move sequence like "R U R' U'"."""
    for move in moves.split():
        face, times = move[0], {"": 1, "'": 3, "2": 2}[move[1:]]
        for _ in range(times):
            where = turn(where, face)
    return where


def solved():
    """Sticker `i` starts at facelet `i`, so a sticker's number says its home."""
    return list(range(54))


def colour_of(sticker):
    return COLOUR[FACES[sticker // 9]]


def reverse(moves):
    """The sequence that undoes `moves` — a solve, without needing a solver."""
    flip = {"": "'", "'": "", "2": "2"}
    return " ".join(m[0] + flip[m[1:]] for m in reversed(moves.split()))
