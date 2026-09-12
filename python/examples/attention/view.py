"""Three acts: the sentence, one query worked out, then the whole matrix.

Each act owns the screen. Nothing from the previous one lingers, so there is
only ever one place to look.
"""

import re

import codimate as cm

import attention as A
from layout import (BAND_PAD, CAPTION_Y, CELL, DIM, EMPTY, FORMULA_Y, GRID,
                    HEADS_IN_LAYER, INK, KEY, CAPTION_BG, MARK, MASKED, MIX_Y, NOTE_Y, ON_MARK,
                    OUT_Y, PAIR_CELL, PAIR_LEFT, PAIR_RIGHT, PANEL, QUERY,
                    QUIET, ROW_ALT, ROW_VALUE, ROW_WORDS, SENTENCE_HIGH,
                    SENTENCE_MID, STRIP_W,
                    STRIP_Y, TABLE_LABEL, UNLIT, VBOX_H, VBOX_W, VBOX_Y,
                    WARN, WEIGHT_Y, cell, column, head_marker, heat)

HEAD_INDEX = {"subject": 3, "previous": 11}

WORKING = ("dot", "sqrt", "scale", "softmax", "sum", "compare")
VALUES = ("values", "mix", "output")
MATRIX = ("matrix", "rows", "whole")
PAIR = ("coda", "why")

# The title names the video, which is worth knowing once. After that it is a
# line of text competing with the thing it introduced.
TITLED = ("sentence", "word")

# The twelve markers only mean something once a *head's* matrix is on screen.
# Before that there is no head to be looking at, so they are decoration sitting
# on top of the sentence and the equation.
STRIPPED = MATRIX + PAIR


def _sentence(scene, walk):
    """The six tokens, lit in three passes.

    `word` picks one out and leaves the rest alone — one new idea. Only from
    `mask` on does the sentence split into what may be looked at and what may
    not, which is a second idea and so gets its own beat.
    """
    span = 132.0
    left = 640.0 - span * (A.N - 1) / 2
    y = SENTENCE_HIGH if walk.stage == "formula" else SENTENCE_MID
    for i, word in enumerate(A.WORDS):
        if walk.query is None:
            colour = INK
        elif i == walk.query:
            colour = QUERY
        elif walk.stage == "word":
            colour = INK          # nothing is ruled out yet
        elif i < walk.query:
            colour = KEY
        else:
            colour = QUIET        # masked: it exists, it just cannot be seen
        scene.text(("word", i), word, x=left + span * i, y=y,
                   size=40, color=colour)

    # Which token supplies the query and which supply the keys — the one piece
    # of Q/K vocabulary the animation needs, attached to the words themselves
    # rather than explained in the abstract.
    if walk.stage == "roles":
        for i in range(walk.query + 1):
            role = "Q" if i == walk.query else "K"
            scene.text(("role", i), role, x=left + span * i, top=y + 22,
                       size=24, color=QUERY if i == walk.query else KEY)


def _formula(scene, walk):
    r"""The equation, typeset rather than approximated.

    It used to be spelled with Unicode lookalikes — `QKᵀ / √dₖ` — because the
    Authoring Surface had no way to say `\frac`. It does now, so the equation
    on screen is the equation from the paper.

    It is drawn on the opening scene too, fully hidden, so that arriving is a
    `reveal` from 0 to 1 — a pen traces each glyph's outline and the solid
    letter fills in behind it, instead of the whole block fading up at once.
    """
    scene.formula(
        "formula",
        r"\mathrm{Attention}(Q, K, V) = \mathrm{softmax}\!\left("
        r"\frac{QK^{T}}{\sqrt{d_k}}\right)V",
        x=640, y=FORMULA_Y, size=42, color=INK, pen=2.2,
        reveal=1.0 if walk.stage == "formula" else 0.0)


def _working(scene, walk):
    """One query's arithmetic as a single row that changes value.

    The numbers keep the same key from one stage to the next, so QK^T's -13.61
    *becomes* -1.70 rather than a second row appearing below it. That is the
    whole reason to animate this: the division is something you watch happen to
    the numbers you were just reading.

    Two stages add a typeset aside below the row rather than a second row of
    numbers: `sqrt` shows where the 8 comes from, `sum` shows that the weights
    total 1. `compare` is the one stage that needs a real second row, because
    it is a comparison — it appears, is read, and leaves.
    """
    raw = A.raw_scores(walk.head, walk.query)
    scaled = A.scale(raw)
    weights = A.softmax(scaled)

    label, values, size = {
        "dot": (r"QK^{T}", raw, 38),
        "sqrt": (r"QK^{T}", raw, 38),
        "scale": (r"\div \sqrt{d_k}", scaled, 38),
        "softmax": (r"\mathrm{softmax}", weights, 40),
        "sum": (r"\mathrm{softmax}", weights, 40),
        "compare": (r"\mathrm{with} \div \sqrt{d_k}", weights, 40),
    }[walk.stage]

    # Only `dot` reveals the numbers one at a time; by every later stage all
    # three are on screen and are being transformed together.
    shown = walk.keys if walk.stage == "dot" else len(raw)
    best = max(range(len(values)), key=lambda j: values[j])

    # No winner while the scores are still landing — there is nothing to
    # compare one number against yet.
    if shown == len(raw):
        # Sized from the number it is actually behind, so it stays correct when
        # the value, the font size or the sentence changes.
        width = cm.measure(f"{values[best]:.2f}", size)[0] + BAND_PAD
        scene.rect("band", x=column(best), top=ROW_WORDS - 32, w=width,
                   h=(ROW_VALUE + 42) - (ROW_WORDS - 32), radius=10, color=PANEL)

    for j in range(len(raw)):
        scene.text(("word", j), A.WORDS[j], x=column(j), y=ROW_WORDS,
                   size=34, color=QUERY if j == walk.query else KEY)

    scene.formula("l_row", label, x=TABLE_LABEL, y=ROW_VALUE, size=32, color=DIM)
    for j in range(shown):
        scene.text(("val", j), f"{values[j]:.2f}", x=column(j), y=ROW_VALUE,
                   size=size, color=INK if j == best else DIM)

    # Where the 8 comes from, and then the division actually being done. Shown
    # once each, on the winning number, so the arithmetic is checkable rather
    # than asserted.
    if walk.stage == "sqrt":
        scene.formula("note", rf"\sqrt{{d_k}} = \sqrt{{{A.D_K}}} = 8",
                      x=640, y=NOTE_Y, size=40, color=KEY)
    elif walk.stage == "scale":
        scene.formula("note", rf"\frac{{{raw[best]:.2f}}}{{8}} = {scaled[best]:.2f}",
                      x=640, y=NOTE_Y, size=40, color=KEY)
    elif walk.stage == "sum":
        total = " + ".join(f"{w:.2f}" for w in weights)
        scene.formula("note", rf"{total} = 1.00", x=640, y=NOTE_Y, size=38, color=KEY)

    if walk.stage == "compare":
        flat = A.softmax(raw)
        scene.formula("l_flat", r"\mathrm{without}", x=TABLE_LABEL, y=ROW_ALT,
                      size=26, color=WARN)
        for j in range(len(flat)):
            scene.text(("flat", j), f"{flat[j]:.2f}", x=column(j), y=ROW_ALT,
                       size=36, color=WARN)


def _values(scene, walk):
    """What the weights are *for*.

    The animation used to stop at the weights, but the equation ends in `V`.
    This act spends it: three weights, three value vectors, one weighted sum.
    Deliberately schematic — a real value vector is 64 numbers wide, and
    drawing that would teach nothing the boxes do not.
    """
    weights = A.row(walk.head, walk.query)

    for j in range(walk.query + 1):
        lit = walk.stage != "values"
        scene.text(("w", j), f"{weights[j]:.2f}", x=column(j), y=WEIGHT_Y,
                   size=36, color=INK if j == 1 else DIM)

        # The value vectors. One box each, labelled — not 64 numbers.
        scene.rect(("vbox", j), x=column(j), y=VBOX_Y, w=VBOX_W, h=VBOX_H,
                   radius=10, color=PANEL)
        scene.formula(("vlab", j), rf"V_{{\mathrm{{{A.WORDS[j]}}}}}",
                      x=column(j), y=VBOX_Y, size=30,
                      color=INK if j == 1 else DIM)

        # Each value flows into the sum, weighted. Thickness carries the
        # weight, so 0.96 is visibly most of what arrives.
        if lit:
            scene.line(("flow", j),
                       start=(column(j), VBOX_Y + VBOX_H / 2),
                       end=(640.0, MIX_Y - 14),
                       w=1.0 + 7.0 * weights[j],
                       color=heat(max(weights[j], 0.25)))

    if walk.stage == "values":
        return

    scene.formula(
        "mix",
        " + ".join(rf"{weights[j]:.2f}\,V_{{\mathrm{{{A.WORDS[j]}}}}}"
                   for j in range(walk.query + 1)),
        x=640, y=MIX_Y + 22, size=32, color=INK)

    if walk.stage == "output":
        label = 'a new representation of "sat"'
        scene.rect("outbox", x=640, y=OUT_Y, w=cm.measure(label, 22)[0] + 56,
                   h=VBOX_H, radius=10, color=PANEL)
        # Not a repeat of the caption — the caption says what just happened,
        # the box says what the thing *is*. "ក្នុង Head នេះ" lives in the
        # caption so the shorter label keeps to the box's width.
        scene.text("outlab", "លទ្ធផល Attention សម្រាប់ 'sat'",
                   x=640, y=OUT_Y, size=24, color=KEY)


def _heads(scene, walk):
    """One marker per head in the layer, the one in use lit.

    Drawn from the moment the equation appears — not on the opening scene —
    so by the time the head changes at the end you already know there are
    twelve of them and which one you have been watching.
    """
    pair = walk.stage in PAIR
    live = HEAD_INDEX[walk.head]
    for i in range(HEADS_IN_LAYER):
        x, y = head_marker(i)
        if i == live:
            colour = KEY
        elif pair and i in HEAD_INDEX.values():
            colour = QUERY
        else:
            colour = UNLIT
        scene.rect(("head_mark", i), x=x, y=y, w=STRIP_W, h=16, color=colour)

    label = ("GPT-2, layer 4 — 2 ក្នុងចំណោម Head 12" if pair
             else "GPT-2, layer 4 — Head 12 នេះជា Head ដែលយើងកំពុងតាម")
    scene.text("strip_label", label, x=640, top=STRIP_Y + 16, size=17, color=QUIET)


def _grid(scene, walk, head, origin, size, tag, rows):
    for j, word in enumerate(A.WORDS):
        x, _ = cell(0, j, origin, size)
        scene.text((tag, "col", j), word, x=x, bottom=origin[1] - 14,
                   size=20 if size < CELL else 22, color=DIM)
    for i, word in enumerate(A.WORDS):
        _, y = cell(i, 0, origin, size)
        scene.text((tag, "row", i), word, x=origin[0] - 46, y=y,
                   size=20 if size < CELL else 22, color=DIM)

    for i in range(A.N):
        done = i in rows
        weights = A.row(head, i) if done else None
        for j in range(A.N):
            x, y = cell(i, j, origin, size)
            box = scene.group((tag, "cell", i, j), x=x, y=y)
            if j > i:
                box.rect("fill", w=size - 3, h=size - 3, color=MASKED)
            elif not done:
                box.rect("fill", w=size - 3, h=size - 3, color=EMPTY)
            else:
                box.rect("fill", w=size - 3, h=size - 3, color=heat(weights[j]))
                box.text("v", f"{weights[j]:.2f}", size=18 if size < CELL else 19,
                         color=INK if weights[j] > 0.3 else DIM)


def _matrix(scene, walk):
    if walk.stage not in PAIR:
        _grid(scene, walk, walk.head, GRID, CELL, "one", walk.rows)
        return

    every = set(range(A.N))
    for head, origin, tag, note in (
            ("subject", PAIR_LEFT, "left", "head 3 — subject-like pattern"),
            ("previous", PAIR_RIGHT, "right", "head 11 — previous-token pattern")):
        _grid(scene, walk, head, origin, PAIR_CELL, tag, every)
        scene.text((tag, "note"), note,
                   x=origin[0] + PAIR_CELL * A.N / 2, y=origin[1] + PAIR_CELL * A.N + 38,
                   size=21, color=KEY if head == "previous" else DIM)


def _wrap(text, size=40, max_px=1150):
    """Split a statement into at most two readable lines.

    A guess at each glyph's advance, good enough to flag a line that is far
    too long: Khmer characters are ~half an em wide, Latin ~0.6 em.
    """
    def w(ch):
        if ch == " ":
            return 0.32 * size
        if 0x1780 <= ord(ch) <= 0x17FF:
            return 0.52 * size
        return 0.60 * size

    width = sum(w(c) for c in text)
    if width <= max_px:
        return [text]

    target = width / 2
    cum, best, best_d = 0.0, -1, float("inf")
    for i, c in enumerate(text):
        if c == " ":
            d = abs(cum - target)
            if d < best_d:
                best_d, best = d, i
        cum += w(c)
    if best < 0:
        return [text]
    return [text[:best], text[best + 1:]]


def _statement(scene, walk):
    """Read-first: the whole screen is the sentence, nothing else. The visual
    that pays it off takes over on the next beat — a card is the message, so
    the payoff never repeats it as a subtitle."""
    lines = _wrap(walk.card)
    y0 = 360.0 - (len(lines) - 1) * 26.0
    for i, line in enumerate(lines):
        scene.text(("statement", i), line, x=640, y=y0 + i * 52,
                   size=40, color=INK)


def draw(scene, walk):
    if walk.stage in TITLED:
        scene.text("title", "ដំណើរការនៃ QKV នៅក្នុង ស្ថាបត្យកម្ម Attention",
                   x=640, y=52, size=30, color=INK)
    if walk.card:
        _statement(scene, walk)
        return
    if walk.stage in STRIPPED:
        _heads(scene, walk)

    if walk.stage in MATRIX or walk.stage in PAIR:
        _matrix(scene, walk)
    elif walk.stage in WORKING:
        _working(scene, walk)
    elif walk.stage in VALUES:
        _values(scene, walk)
    else:
        _sentence(scene, walk)
        # Drawn on the opening scene too, but fully hidden — see `_formula`.
        _formula(scene, walk)

    _caption(scene, walk)


CAPTION_SIZE = 24

# How far the mark extends past its word. Must stay under the word gap above.
MARK_PAD = 13.0

# One Khmer orthographic cluster: a base letter, any subscript consonants hung
# under it with COENG, then its vowels and signs.
_CLUSTER = re.compile(r"[\u1780-\u17B3](?:\u17D2[\u1780-\u17B3])*[\u17B6-\u17D3]*")

# U+200B ZERO WIDTH SPACE — what Khmer actually uses to mark a word boundary.
_ZWSP = "\u200b"


_MATH = re.compile(r"\$[^$]+\$")


def _words(text, preceded):
    """Split a run of plain caption text into the pieces the mark steps over."""
    pieces = []
    segmented = _ZWSP in text
    for w, token in enumerate(text.split()):
        first = True
        gap = preceded or w > 0
        if segmented:
            for part in token.split(_ZWSP):
                if part:
                    pieces.append((part, gap and first))
                    first = False
            continue
        i = 0
        while i < len(token):
            match = _CLUSTER.match(token, i)
            if match and match.end() > i:
                piece, i = match.group(), match.end()
            else:
                # A run of anything else — "Model", "0.96" — stays whole.
                j = i
                while j < len(token) and not _CLUSTER.match(token, j):
                    j += 1
                piece, i = token[i:j], j
            pieces.append((piece, gap and first))
            first = False
    return pieces


def chunks(text):
    """Split a caption into the pieces the mark steps through.

    Returns `(piece, space_before)` pairs.

    `$...$` is LaTeX and comes out first, before anything else is split — it
    may contain spaces (`$\\sqrt{d_k} = 8$`) and must survive as one piece, or
    it is drawn as literal source instead of as mathematics.

    The rest is Khmer, which does not separate words with spaces.
    `tools/segment_captions.py` marks the real boundaries with ZWSP ahead of
    time; without any, this falls back to orthographic clusters so a caption
    still reads rather than arriving in one lump.
    """
    pieces, last = [], 0
    for found in _MATH.finditer(text):
        pieces += _words(text[last:found.start()], bool(pieces))
        pieces.append((found.group(), bool(pieces)))
        last = found.end()
    return pieces + _words(text[last:], bool(pieces))


def _is_math(piece):
    """A caption piece written as `$...$` is LaTeX, not prose."""
    return len(piece) > 2 and piece.startswith("$") and piece.endswith("$")


def _piece_width(piece):
    if _is_math(piece):
        return cm.measure_math(piece[1:-1], CAPTION_SIZE)[0]
    return cm.measure(piece, CAPTION_SIZE)[0]


def caption_width(text):
    """How wide a caption will draw, for checking it fits before rendering."""
    pieces = chunks(text)
    space = cm.measure(" ", CAPTION_SIZE)[0]
    width = sum(_piece_width(piece) for piece, _ in pieces)
    return width + sum(space * 1.7 if gap else space for _, gap in pieces[1:])


def _caption(scene, walk):
    """The narration, a word at a time.

    Each word is its own item keyed by its own text, so a word *arrives*
    rather than one long string swapping its contents. That also sidesteps the
    one-segment text lag entirely: nothing here ever changes its text, things
    only enter and leave.

    Laid out with `cm.measure`, because the words are Khmer, English and
    numbers mixed together and a character count is meaningless for that.
    """
    pieces = chunks(walk.caption)
    if not pieces:
        return

    space = cm.measure(" ", CAPTION_SIZE)[0]
    widths = [_piece_width(piece) for piece, _ in pieces]

    # Khmer runs its words together, so a word boundary found by the segmenter
    # has no gap of its own. Give it a full space; a real space in the source
    # gets more, so phrasing still reads. The gap also has to clear MARK_PAD,
    # or the mark slides under the neighbouring word — and near-white text on
    # the near-white mark is invisible rather than merely ugly.
    leads = [0.0] + [space * 1.7 if gap else space for _, gap in pieces[1:]]

    # Positions computed once, so the mark cannot drift away from the word it
    # is marking — two copies of this arithmetic is exactly how that happens.
    centres, x = [], 0.0
    for lead, wide in zip(leads, widths):
        x += lead
        centres.append(x + wide / 2)
        x += wide

    left = 640.0 - x / 2
    here = (walk.said or 0) - 1        # -1 before the first word is marked

    # The plate. Sized to the line it holds, so it never runs wider than the
    # words — a bar of fixed width would look like chrome rather than like the
    # narration having a place of its own.
    scene.rect("cap_bg", x=640, y=CAPTION_Y, w=x + 52, h=CAPTION_SIZE + 30,
               radius=15, color=CAPTION_BG)

    # ONE rect for the mark, so the engine slides and resizes it from word to
    # word rather than blinking it out and in. That movement is the reading.
    if 0 <= here < len(pieces):
        scene.rect("cap_mark", x=left + centres[here], y=CAPTION_Y,
                   w=widths[here] + MARK_PAD, h=CAPTION_SIZE + 14, radius=7,
                   color=MARK)

    for i, ((piece, _), centre) in enumerate(zip(pieces, centres)):
        colour = ON_MARK if i == here else INK
        if _is_math(piece):
            # Real mathematics in the narration. A caption saying "sqrt(d_k)"
            # in ASCII beside an equation that renders it properly is the kind
            # of detail that makes the whole thing look unfinished.
            scene.formula(("cap", i, piece), piece[1:-1], x=left + centre,
                          y=CAPTION_Y, size=CAPTION_SIZE, color=colour)
        else:
            scene.text(("cap", i, piece), piece, x=left + centre, y=CAPTION_Y,
                       size=CAPTION_SIZE, color=colour)
