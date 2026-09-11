"""Three acts: the sentence, one query worked out, then the whole matrix.

Each act owns the screen. Nothing from the previous one lingers, so there is
only ever one place to look.
"""

import attention as A
from layout import (CAPTION_Y, CELL, DIM, EMPTY, FORMULA_Y, GRID, INK, KEY,
                    MASKED, QUERY, QUIET, SENTENCE_Y, TABLE_LABEL, WARN,
                    cell, column, heat, line)

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


def _matrix(scene, walk):
    for j, word in enumerate(A.WORDS):
        x, _ = cell(0, j)
        scene.text(("head", j), word, x=x, bottom=GRID[1] - 16, size=20, color=DIM)
    for i, word in enumerate(A.WORDS):
        _, y = cell(i, 0)
        scene.text(("side", i), word, x=GRID[0] - 52, y=y, size=20, color=DIM)

    for i in range(A.N):
        done = i in walk.rows
        weights = A.row(walk.head, i) if done else None
        for j in range(A.N):
            x, y = cell(i, j)
            box = scene.group(("cell", i, j), x=x, y=y)
            if j > i:
                box.rect("fill", w=CELL - 3, h=CELL - 3, color=MASKED)
            elif not done:
                box.rect("fill", w=CELL - 3, h=CELL - 3, color=EMPTY)
            else:
                box.rect("fill", w=CELL - 3, h=CELL - 3, color=heat(weights[j]))
                box.text("v", f"{weights[j]:.2f}", size=17,
                         color=INK if weights[j] > 0.3 else DIM)


def draw(scene, walk):
    scene.text("title", "Scaled dot-product attention", x=640, y=52,
               size=30, color=INK)
    scene.text("source", f"GPT-2, {A.HEADS[walk.head][2]}", x=640, y=88,
               size=18, color=QUIET)

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
