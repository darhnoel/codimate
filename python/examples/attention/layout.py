"""Where each act sits. Only one act is on screen at a time."""

# Act 1 and 2: the sentence, the formula, and one query worked out.
SENTENCE_Y = 168.0
FORMULA_Y = 250.0
TABLE_TOP = 358.0
TABLE_ROW = 58.0
TABLE_LABEL = 372.0              # "QKᵀ", "÷ √dₖ", "softmax"
TABLE_COL0 = 560.0               # first key's column
TABLE_GAP = 186.0
CAPTION_Y = 648.0

# Act 3: the whole matrix, centred now that the working has gone.
CELL = 64.0
GRID = (448.0, 212.0)

INK = "#e8eef7"
DIM = "#68738a"
QUIET = "#3d4657"
EMPTY = "#151c28"
MASKED = "#0d1118"
QUERY = "#f59e0b"
KEY = "#4ade80"
WARN = "#f87171"
WEAK = (0x14, 0x22, 0x3a)
STRONG = (0x60, 0xa5, 0xfa)


def cell(row, col):
    return (GRID[0] + CELL * (col + 0.5), GRID[1] + CELL * (row + 0.5))


def column(j):
    return TABLE_COL0 + TABLE_GAP * j


def line(n):
    return TABLE_TOP + TABLE_ROW * n


def heat(weight):
    t = 0.0 if weight < 0 else 1.0 if weight > 1 else weight
    t = t ** 0.65
    return "#%02x%02x%02x" % tuple(
        round(WEAK[i] + (STRONG[i] - WEAK[i]) * t) for i in range(3))
