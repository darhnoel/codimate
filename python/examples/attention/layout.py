"""Where each act sits. Only one act is on screen at a time."""

# Act 1 and 2: the sentence, the formula, and one query worked out.
SENTENCE_Y = 184.0
FORMULA_Y = 250.0
TABLE_TOP = 358.0
TABLE_ROW = 58.0
TABLE_LABEL = 372.0              # "QKᵀ", "÷ √dₖ", "softmax"
TABLE_COL0 = 560.0               # first key's column
TABLE_GAP = 186.0
CAPTION_Y = 648.0

# The twelve heads of layer 4, as a strip. Without it, switching head at the
# end changes a word in a subtitle and nothing else — there is no way to notice
# that a layer has heads at all, let alone that we swapped one.
HEADS_IN_LAYER = 12
STRIP_Y = 104.0
STRIP_W = 26.0
STRIP_GAP = 8.0

# Act 3: the whole matrix, centred now that the working has gone.
CELL = 64.0
GRID = (448.0, 212.0)

# The coda puts both heads side by side, which is the only way "they learned
# different things" is something you can see rather than be told.
PAIR_CELL = 50.0
PAIR_LEFT = (196.0, 268.0)
PAIR_RIGHT = (744.0, 268.0)

INK = "#e8eef7"
DIM = "#68738a"
QUIET = "#3d4657"
UNLIT = "#222a39"
EMPTY = "#151c28"
MASKED = "#0d1118"
QUERY = "#f59e0b"
KEY = "#4ade80"
WARN = "#f87171"
WEAK = (0x14, 0x22, 0x3a)
STRONG = (0x60, 0xa5, 0xfa)


def cell(row, col, origin=GRID, size=CELL):
    return (origin[0] + size * (col + 0.5), origin[1] + size * (row + 0.5))


def head_marker(index):
    span = STRIP_W + STRIP_GAP
    left = 640.0 - span * (HEADS_IN_LAYER - 1) / 2
    return (left + span * index, STRIP_Y)


def column(j):
    return TABLE_COL0 + TABLE_GAP * j


def line(n):
    return TABLE_TOP + TABLE_ROW * n


def heat(weight):
    t = 0.0 if weight < 0 else 1.0 if weight > 1 else weight
    t = t ** 0.65
    return "#%02x%02x%02x" % tuple(
        round(WEAK[i] + (STRONG[i] - WEAK[i]) * t) for i in range(3))
