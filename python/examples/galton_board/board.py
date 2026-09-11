"""The board itself: how big it is, where everything sits, what colour it is.

Shared by the simulation and the drawing, so neither has to import the other.
"""

import math

ROWS = 6
BALLS = 300

CX = 640.0
HALF = 50.0                     # half the gap between neighbouring slots
TOP_Y, ROW_GAP = 208.0, 52.0    # TOP_Y is the first row of pegs
SPAWN_Y = 160.0                 # balls are dropped from here, above the board
BIN_BASE, BIN_W, TALLEST_BAR = 676.0, 84.0, 132.0
BALL_R, PEG_R = 9.0, 7.0
PERCH = BALL_R + PEG_R - 3.0    # a ball rides on top of the peg it just struck

FALLING_IN = -1                 # dropped, not yet down to the first peg

PEG = "#3a465e"
BALL = "orange"
BIN = "#4a9eff"
INK = "#e8eef7"

# What an even chance each way predicts: the binomial, which for this many rows
# is already the bell the whole thing is about.
EXPECTED = [BALLS * math.comb(ROWS, k) / 2 ** ROWS for k in range(ROWS + 1)]

# Scale the bars so the expected peak just fills the space. Fixed for the whole
# render — bars that rescaled as counts came in would animate a lie.
BIN_UNIT = TALLEST_BAR / max(EXPECTED)


def slot_x(level, rights):
    """Where a ball sits after `rights` rights out of `level` decisions."""
    return CX + (2 * rights - level) * HALF


def level_y(level):
    """Where the pegs of a row sit."""
    return TOP_Y + level * ROW_GAP


def perch_y(level):
    """Where a ball sits when it has just struck a peg of that row."""
    return level_y(level) - PERCH
