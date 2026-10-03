"""The caption: a plate sized to the line, a bright mark stepping word by word,
never more than eight words on screen, held as long as the line takes to read.

The same caption as `archimedes` and the Cavendish film. It is here again
because it has not found its own home yet (ADR 0019 leaves it out of the kit).
"""

import codimate as cm

ZWSP = "\u200b"
PLATE, READ, UNREAD = "#141a26", "#58C4DD", "#5d6a7e"
SIZE, ROOM = 34, 1150.0
PAGE = 8                                       # the most words shown at once
WORD, PER_LETTER, CLAUSE = 0.24, 0.035, 0.55   # reading pace, as archimedes
STOPS = "។៕៖,.;:?!"
_MARKS = range(0x17B4, 0x17D4)


def chunks(line):
    """`line` as the pieces the mark steps over — Khmer words where the line
    has been segmented (ZWSP), spaced words otherwise — each flagged with
    whether a real space comes before it."""
    pieces = []
    for w, token in enumerate(line.split()):
        first = True
        for part in token.split(ZWSP):
            if not part:
                continue
            if pieces and all(ch in STOPS for ch in part):
                pieces[-1] = (pieces[-1][0] + part, pieces[-1][1])   # ៖ is not a word
                continue
            pieces.append((part, (w > 0) and first))
            first = False
    return pieces


def clusters(line):
    """`line` as orthographic clusters, so a word is timed by what a reader
    sees and not by how many code points hold its vowel signs."""
    line = line.replace(ZWSP, "")
    out, i = [], 0
    while i < len(line):
        piece, i = line[i], i + 1
        while i < len(line):
            here = ord(line[i])
            if here == 0x17D2 and i + 1 < len(line):
                piece, i = piece + line[i:i + 2], i + 2
            elif here in _MARKS or here == 0x17DD or 0x0300 <= here <= 0x036F:
                piece, i = piece + line[i], i + 1
            else:
                break
        out.append(piece)
    return out


def pace(piece):
    """Seconds one word holds the mark: a moment, a little per letter, and a
    rest where a clause ends."""
    rest = CLAUSE if piece and piece[-1] in STOPS else 0.0
    return WORD + PER_LETTER * len(clusters(piece)) + rest


def per_page(count):
    """Words on each page: as few pages as `PAGE` allows, and as even as they
    can be, so twelve words are two pages of six rather than eight and four."""
    pages = -(-count // PAGE)
    return -(-count // pages)


def draw(scene, line, said, y, layer):
    """The caption for `line`, the mark on word number `said` (1-based).
    Words over `PAGE` turn a page rather than shrinking to fit."""
    pieces = chunks(line) if line else []
    if not pieces:
        return
    per = per_page(len(pieces))
    page = max(said - 1, 0) // per
    shown = pieces[page * per:(page + 1) * per]
    here = (said - 1) - page * per

    size = SIZE
    while size > 13 and cm.measure(" ".join(p for p, _ in shown), size=size)[0] > ROOM:
        size -= 1
    space = cm.measure(" ", size=size)[0]
    widths = [cm.measure(p, size=size)[0] for p, _ in shown]
    # Khmer sets its words flush: only a space the author typed is a space.
    leads = [0.0] + [space if gap else 0.0 for _, gap in shown[1:]]
    centres, run = [], 0.0
    for lead, wide in zip(leads, widths):
        run += lead
        centres.append(run + wide / 2)
        run += wide
    left = cm.width() / 2 - run / 2

    scene.rect(("say", "plate", line, page), w=run + 56, h=size + 32,
               at=(cm.width() / 2, y)).fill(PLATE).round(15).on(layer=layer)
    for i, ((piece, _), centre) in enumerate(zip(shown, centres)):
        scene.text(("say", page * per + i, piece), piece, size=size,
                   at=cm.at(x=left + centre, y=y)) \
            .fill(READ if i == here else UNREAD).on(layer=layer + 1)
