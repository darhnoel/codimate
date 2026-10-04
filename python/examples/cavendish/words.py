"""How a caption is cut into the pieces its mark steps over, and how long each is
held at reading pace. `main.py` draws with these; `speak.py` lines the voice up
against them, so both must cut a caption the same way."""

ZWSP = "\u200b"
WORD, PER_LETTER, CLAUSE = 0.24, 0.035, 0.55  # reading pace, as archimedes
STOPS = "។៕៖,.;:?!"


def chunks(line):
    """`line` as the pieces the mark steps over — Khmer words where the line
    has been segmented (ZWSP), spaced words otherwise — each flagged with
    whether a real space comes before it."""
    pieces = []
    for w, token in enumerate(line.split()):
        first = True
        for part in token.split(ZWSP):
            if part:
                pieces.append((part, (w > 0) and first))
                first = False
    return pieces


def clusters(line):
    """`line` split into orthographic clusters, so a word is timed by what a
    reader sees and not by how many code points hold its vowel signs."""
    marks = range(0x17B4, 0x17D4)
    line = line.replace(ZWSP, "")
    out, i = [], 0
    while i < len(line):
        piece, i = line[i], i + 1
        while i < len(line):
            here = ord(line[i])
            if here == 0x17D2 and i + 1 < len(line):
                piece, i = piece + line[i : i + 2], i + 2
            elif here in marks or here == 0x17DD or 0x0300 <= here <= 0x036F:
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
