"""Drawing the board: hopper, pegs, dividers, bins, balls.

Reads the simulation's state; the simulation knows nothing about this file.
"""

from board import (BALL, BALL_R, BIN, BIN_BASE, BIN_UNIT, BIN_W, BALLS, CX,
                   HALF, INK, PEG, PEG_R, ROWS, SPAWN_Y, level_y, perch_y,
                   slot_x)


def draw(scene, board):
    scene.text("title", "Galton board", x=CX, y=56, size=38, color=INK)
    scene.text("subtitle",
               f"{BALLS} balls, {ROWS} rows — {sum(board.bins)} landed",
               x=CX, y=92, size=20, color="grey")

    # The hopper the balls are dropped from.
    for side in (-1, 1):
        scene.line(("hopper", side),
                   start=(CX + side * 62, SPAWN_Y - 26),
                   end=(CX + side * 21, SPAWN_Y + 14), w=3.0, color=PEG)

    # Pegs: named after where they are, so they never move. A ball's resting
    # place is the top of a peg, so these sit exactly on its path.
    for row in range(ROWS):
        for i in range(row + 1):
            scene.circle(("peg", row, i), x=slot_x(row, i), y=level_y(row),
                         r=PEG_R, color=PEG)

    # The dividers. Without them a ball leaving the last peg sideways would
    # drift out of the column its coin flips put it in.
    for j in range(ROWS + 2):
        x = CX + (2 * j - ROWS - 1) * HALF
        scene.line(("divider", j), start=(x, perch_y(ROWS) - 4), end=(x, BIN_BASE),
                   w=2.0, color=PEG)

    scene.line("floor",
               start=(slot_x(ROWS, 0) - BIN_W / 2 - 6, BIN_BASE),
               end=(slot_x(ROWS, ROWS) + BIN_W / 2 + 6, BIN_BASE),
               w=2.0, color=PEG)
    # Bins: also named after where they are. They grow, they do not travel.
    tallest = max(board.bins)
    for k, count in enumerate(board.bins):
        bar = scene.group(("bin", k), x=slot_x(ROWS, k), bottom=BIN_BASE)
        bar.rect("box", w=BIN_W, h=max(count * BIN_UNIT, 1.0), bottom=0,
                 color=BIN if count < tallest else "#7cc0ff")
        bar.text("count", count if count else "", top=12, size=20, color="grey")

    # Balls: named after the ball, so each one travels its own path.
    for ball in board.balls:
        scene.circle(("ball", ball.id), x=ball.x, y=ball.y,
                     r=BALL_R, color=BALL, layer=5)


# `linear`: the simulation is sampled densely enough that the parabola is
# already in the samples. Easing between them would add a wobble physics
# never asked for.
