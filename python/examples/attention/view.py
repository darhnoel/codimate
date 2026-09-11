"""Three acts: the sentence, one query worked out, then the whole matrix.

Each act owns the screen. Nothing from the previous one lingers, so there is
only ever one place to look.
"""

import attention as A
from layout import (CAPTION_Y, CELL, DIM, EMPTY, FORMULA_Y, GRID,
                    HEADS_IN_LAYER, INK, KEY, MASKED, PAIR_CELL, PAIR_LEFT,
                    PAIR_RIGHT, QUERY, QUIET, SENTENCE_Y, STRIP_W, STRIP_Y,
                    TABLE_LABEL, UNLIT, WARN, cell, column, head_marker, heat,
                    line)

HEAD_INDEX = {"subject": 3, "previous": 11}

WORKING = ("dot", "scale", "softmax", "compare")
MATRIX = ("matrix", "rows", "whole", "coda")


def _sentence(scene, walk):
    """The six tokens. Amber is the query; green is what it may look at."""
    span = 132.0
    left = 640.0 - span * (A.N - 1) / 2
    for i, word in enumerate(A.WORDS):
        if walk.query is None:
            colour = INK
        elif i == walk.query:
            colour = QUERY
        elif i < walk.query and (walk.stage != "dot" or i < walk.keys):
            colour = KEY
        else:
            colour = QUIET
        scene.text(("word", i), word, x=left + span * i, y=SENTENCE_Y,
                   size=34, color=colour)


def _formula(scene, walk):
    dim = walk.stage in WORKING
    scene.text("formula", "Attention(Q, K, V)  =  softmax( QKᵀ / √dₖ ) V",
               x=640, y=FORMULA_Y, size=30, color=DIM if dim else INK)
    if walk.stage != "sentence":
        scene.text("dk", f"dₖ = {A.D_K}, the width of one head",
                   x=640, y=FORMULA_Y + 38, size=18, color=QUIET)


def _working(scene, walk):
    """One query's arithmetic, revealed a column then a row at a time."""
    raw = A.raw_scores(walk.head, walk.query)
    scaled = A.scale(raw)
    weights = A.softmax(scaled)

    for j in range(walk.keys):
        scene.text(("col", j), A.WORDS[j], x=column(j), y=line(0),
                   size=24, color=KEY)
        scene.text(("raw", j), f"{raw[j]:.2f}", x=column(j), y=line(1),
                   size=30, color=INK)

    scene.text("l_raw", "QKᵀ", x=TABLE_LABEL, y=line(1), size=26, color=DIM)

    if walk.stage in ("scale", "softmax", "compare"):
        scene.text("l_scale", "÷ √dₖ", x=TABLE_LABEL, y=line(2), size=26, color=DIM)
        for j in range(walk.keys):
            scene.text(("scaled", j), f"{scaled[j]:.2f}", x=column(j),
                       y=line(2), size=30, color=INK)

    if walk.stage in ("softmax", "compare"):
        scene.text("l_soft", "softmax", x=TABLE_LABEL, y=line(3), size=26, color=DIM)
        for j in range(walk.keys):
            scene.text(("soft", j), f"{weights[j]:.2f}", x=column(j),
                       y=line(3), size=34, color=heat(max(weights[j], 0.4)))

    if walk.stage == "compare":
        flat = A.softmax(raw)
        scene.text("l_flat", "without it", x=TABLE_LABEL, y=line(4), size=22, color=WARN)
        for j in range(walk.keys):
            scene.text(("flat", j), f"{flat[j]:.2f}", x=column(j),
                       y=line(4), size=30, color=WARN)


def _heads(scene, walk):
    """One marker per head in the layer, the one in use lit.

    Drawn from the first moment, so by the time the head changes you already
    know there are twelve of them and which one you have been watching.
    """
    pair = walk.stage == "coda"
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

    label = ("GPT-2, layer 4 — two of its twelve heads" if pair
             else "GPT-2, layer 4 — twelve heads, and this is the one we follow")
    scene.text("strip_label", label, x=640, top=STRIP_Y + 16, size=17, color=QUIET)


def _grid(scene, walk, head, origin, size, tag, rows):
    for j, word in enumerate(A.WORDS):
        x, _ = cell(0, j, origin, size)
        scene.text((tag, "col", j), word, x=x, bottom=origin[1] - 14,
                   size=18 if size < CELL else 20, color=DIM)
    for i, word in enumerate(A.WORDS):
        _, y = cell(i, 0, origin, size)
        scene.text((tag, "row", i), word, x=origin[0] - 46, y=y,
                   size=18 if size < CELL else 20, color=DIM)

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
                box.text("v", f"{weights[j]:.2f}", size=16 if size < CELL else 17,
                         color=INK if weights[j] > 0.3 else DIM)


def _matrix(scene, walk):
    if walk.stage != "coda":
        _grid(scene, walk, walk.head, GRID, CELL, "one", walk.rows)
        return

    every = set(range(A.N))
    for head, origin, tag, note in (
            ("subject", PAIR_LEFT, "left", "head 3 — finds the subject"),
            ("previous", PAIR_RIGHT, "right", "head 11 — looks one word back")):
        _grid(scene, walk, head, origin, PAIR_CELL, tag, every)
        scene.text((tag, "note"), note,
                   x=origin[0] + PAIR_CELL * A.N / 2, y=origin[1] + PAIR_CELL * A.N + 38,
                   size=21, color=KEY if head == "previous" else DIM)


def draw(scene, walk):
    scene.text("title", "Scaled dot-product attention", x=640, y=52,
               size=30, color=INK)
    _heads(scene, walk)

    if walk.stage in MATRIX:
        _matrix(scene, walk)
    else:
        _sentence(scene, walk)
        if walk.stage != "sentence":
            _formula(scene, walk)
        if walk.stage in WORKING:
            _working(scene, walk)

    if walk.caption:
        scene.text("caption", walk.caption, x=640, y=CAPTION_Y, size=24, color=KEY)
