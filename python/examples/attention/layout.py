"""Where each act sits. Only one act is on screen at a time."""

# Act 1 and 2: the sentence, the formula, and one query worked out.
# The sentence is the only thing on screen at first, so it sits in the middle
# where the eye already is. It lifts to make room the moment the equation
# arrives — that move is what tells you something is about to appear under it.
SENTENCE_MID = 348.0
SENTENCE_HIGH = 196.0
# A typeset fraction is ~2.6em tall — far taller than the single line of
# Unicode this used to be — so it sits well clear of the sentence.
FORMULA_Y = 420.0
CAPTION_Y = 648.0

# The working act is ONE row of numbers that changes value, not a table that
# grows. Each stage relabels the row and moves the same three numbers to their
# new values, so the division is something you watch rather than read. A table
# would show the history; this shows the transformation.
# Placed in the same band of the frame the matrix act uses (212..596), so the
# eye does not have to travel when one act replaces the other.
ROW_WORDS = 265.0                # the three candidate words
ROW_VALUE = 355.0                # the numbers, whatever they currently mean
ROW_ALT = 430.0                  # the "without it" contrast, `compare` only

# Three keys, centred on the canvas — which puts "cat" at dead centre, and
# "cat" is what the whole animation is about.
TABLE_GAP = 186.0
TABLE_COL0 = 640.0 - TABLE_GAP   # first key's column
TABLE_LABEL = 280.0              # what is being applied: "QKᵀ", "÷ √dₖ", ...

# A short LaTeX aside under the working row — "sqrt(64) = 8", "14.04/8 = 1.76".
# Below the row so the numbers stay the thing being read.
NOTE_Y = 490.0

# Act 4: the weights are only half of attention. What they are *for* is mixing
# the value vectors, so the last act spends V.
WEIGHT_Y = 240.0                 # the three weights, carried over from act 2
VBOX_Y = 348.0                   # the value vectors
VBOX_W = 152.0
VBOX_H = 66.0
MIX_Y = 480.0                    # where the three lines converge
OUT_Y = 580.0                    # the attention output
OUT_W = 440.0

# The band is measured, not guessed. The hardcoded 150 that used to live here
# came with a comment estimating "-13.61" at ~128px; it actually measures
# 108.4, so the box was 40px wider than it needed to be at one size and would
# have been wrong at any other.
BAND_PAD = 42.0

# The twelve heads of layer 4, as a strip. Without it, switching head at the
# end changes a word in a subtitle and nothing else — there is no way to notice
# that a layer has heads at all, let alone that we swapped one.
HEADS_IN_LAYER = 12
STRIP_Y = 104.0
STRIP_W = 26.0
STRIP_GAP = 8.0

# Act 3: the whole matrix, centred now that the working has gone.
CELL = 70.0
GRID = (448.0, 200.0)

# The coda puts both heads side by side, which is the only way "they learned
# different things" is something you can see rather than be told.
PAIR_CELL = 50.0
PAIR_LEFT = (196.0, 268.0)
PAIR_RIGHT = (744.0, 268.0)

INK = "#e8eef7"
DIM = "#68738a"
QUIET = "#3d4657"
UNLIT = "#222a39"
PANEL = "#1b2332"
EMPTY = "#151c28"
MASKED = "#0d1118"
# One channel, one job: amber is the query, green is a key, the panel is where
# to look, and heat is magnitude. Green used to also mean "highlight" and
# "caption", which left three meanings fighting over one colour.
QUERY = "#f59e0b"
KEY = "#4ade80"
WARN = "#f87171"
WEAK = (0x14, 0x22, 0x3a)
STRONG = (0x60, 0xa5, 0xfa)

# The subtitle plate, and the mark that runs along it.
#
# The mark is green, like a key token, so the video keeps one accent colour
# rather than growing a hue for the narration. It is a deeper green than KEY
# though: the caption text stays white everywhere, including on the mark, and
# white on KEY green measures about 1.5:1 — unreadable at this size. This one
# clears 4.5:1, so the marked word reads exactly like every other word.
#
# The plate is a shade off the background: enough to group the narration and
# hold it apart from the diagram, not enough to compete with it.
CAPTION_BG = "#141a26"
MARK = "#15803d"
ON_MARK = INK


def cell(row, col, origin=GRID, size=CELL):
    return (origin[0] + size * (col + 0.5), origin[1] + size * (row + 0.5))


def head_marker(index):
    span = STRIP_W + STRIP_GAP
    left = 640.0 - span * (HEADS_IN_LAYER - 1) / 2
    return (left + span * index, STRIP_Y)


def column(j):
    return TABLE_COL0 + TABLE_GAP * j


def heat(weight):
    t = 0.0 if weight < 0 else 1.0 if weight > 1 else weight
    t = t ** 0.65
    return "#%02x%02x%02x" % tuple(
        round(WEAK[i] + (STRONG[i] - WEAK[i]) * t) for i in range(3))
