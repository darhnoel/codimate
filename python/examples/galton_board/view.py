"""Drawing the board: hopper, pegs, dividers, bins, balls.

Reads the simulation's state; the simulation knows nothing about this file.
"""

import codimate as cm
from board import (BALL, BALL_R, BIN, BIN_BASE, BIN_UNIT, BIN_W, BALLS, CX,
                   HALF, INK, PEG, PEG_R, ROWS, SPAWN_Y, level_y, perch_y,
                   slot_x)


def draw(scene, board):
    scene.text("title", "Galton board", size=38, at=(CX, 56)).fill(INK)
    scene.text("subtitle", f"{BALLS} balls, {ROWS} rows — {sum(board.bins)} landed",
               size=20, at=(CX, 92)).fill("grey")

    # The hopper the balls are dropped from.
    for side in (-1, 1):
        scene.line(("hopper", side), start=(CX + side * 62, SPAWN_Y - 26),
                   end=(CX + side * 21, SPAWN_Y + 14), w=3.0).fill(PEG)

    # Pegs: named after where they are, so they never move. A ball's resting
    # place is the top of a peg, so these sit exactly on its path.
    for row in range(ROWS):
        for i in range(row + 1):
            scene.circle(("peg", row, i), r=PEG_R,
                         at=(slot_x(row, i), level_y(row))).fill(PEG)

    # The dividers. Without them a ball leaving the last peg sideways would
    # drift out of the column its coin flips put it in.
    for j in range(ROWS + 2):
        x = CX + (2 * j - ROWS - 1) * HALF
        scene.line(("divider", j), start=(x, perch_y(ROWS) - 4), end=(x, BIN_BASE),
                   w=2.0).fill(PEG)

    scene.line("floor", start=(slot_x(ROWS, 0) - BIN_W / 2 - 6, BIN_BASE),
               end=(slot_x(ROWS, ROWS) + BIN_W / 2 + 6, BIN_BASE), w=2.0).fill(PEG)
    # Bins: also named after where they are. They grow, they do not travel.
    tallest = max(board.bins)
    for k, count in enumerate(board.bins):
        bar = scene.group(("bin", k), at=cm.at(x=slot_x(ROWS, k), bottom=BIN_BASE))
        bar.rect("box", w=BIN_W, h=max(count * BIN_UNIT, 1.0),
                 at=cm.at(bottom=0)).fill(BIN if count < tallest else "#7cc0ff")
        bar.text("count", count if count else "", size=20,
                 at=cm.at(top=12)).fill("grey")

    # Balls: named after the ball, so each one travels its own path.
    for ball in board.balls:
        scene.circle(("ball", ball.id), r=BALL_R,
                     at=(ball.x, ball.y)).fill(BALL).on(layer=5)


# `linear`: the simulation is sampled densely enough that the parabola is
# already in the samples. Easing between them would add a wobble physics
# never asked for.
