"""A Galton board — where the bell curve comes from.

    python python/examples/galton_board/main.py

Balls fall through a triangle of pegs. At each peg a ball goes left or right,
each with even chance, and lands in the bin counting how many times it went
right. Nothing aims for a bell curve; it is what evenly-weighted coin flips
add up to.

The falling is simulated, not faked. Balls accelerate under gravity, lose
vertical speed to every peg they strike, and travel a parabola between one peg
and the next. The trace samples that simulation at a fixed time step — so a
ball near the bottom, moving faster, covers more ground per tick than one
near the top. Stepping row by row instead would have made every ball descend
at the same rate no matter what the motion looked like.

Two identity modes at once, which is what makes this one worth reading:

    pegs and bins  named after their PLACE  — they never move
    balls          named after the BALL     — they fall
"""

import math
import random
from dataclasses import dataclass

import codimate as cm

ROWS = 6
BALLS = 80
SEED = 3                        # fixed, so the video is the same every render

# Physics, in pixels and seconds.
GRAVITY = 1400.0
BOUNCE = 0.5                    # share of downward speed kept after a peg
DT = 0.025                      # simulation and sampling step
RELEASE_EVERY = 0.11
REST = 0.08                     # how long a ball sits before joining its bin

CX = 640.0
HALF = 50.0                     # half the gap between neighbouring slots
TOP_Y, ROW_GAP = 208.0, 52.0    # TOP_Y is the first row of pegs
SPAWN_Y = 160.0                 # balls are dropped from here, above the board
BIN_BASE, BIN_W, BIN_UNIT = 676.0, 84.0, 5.0
BALL_R, PEG_R = 9.0, 7.0
PERCH = BALL_R + PEG_R - 3.0    # a ball rides on top of the peg it just struck

FALLING_IN = -1                 # dropped, not yet down to the first peg

PEG = "#3a465e"
BALL = "orange"
BIN = "#4a9eff"
INK = "#e8eef7"


def slot_x(level, rights):
    """Where a ball sits after `rights` rights out of `level` decisions."""
    return CX + (2 * rights - level) * HALF


def level_y(level):
    """Where the pegs of a row sit."""
    return TOP_Y + level * ROW_GAP


def perch_y(level):
    """Where a ball sits when it has just struck a peg of that row."""
    return level_y(level) - PERCH


def fall_time(vy, height):
    """How long to fall `height`, starting at downward speed `vy`."""
    return (-vy + math.sqrt(vy * vy + 2 * GRAVITY * height)) / GRAVITY


# --- the state --------------------------------------------------------------


@dataclass
class Ball:
    id: int
    x: float
    y: float
    vx: float = 0.0
    vy: float = 0.0
    rights: int = 0
    level: int = FALLING_IN
    landed_at: float = -1.0     # when it came to rest, or -1 while falling


class Board:
    def __init__(self):
        self.rng = random.Random(SEED)
        self.clock = 0.0
        self.released = 0
        self.balls = []
        self.bins = [0] * (ROWS + 1)

    def done(self):
        return sum(self.bins) == BALLS

    def _deflect(self, ball):
        """A peg: takes vertical speed, gives sideways speed.

        The sideways speed is whatever lands the ball exactly one slot over by
        the time gravity has carried it down to the next row — so the lattice
        stays exact while the path between pegs stays ballistic.
        """
        ball.vy *= BOUNCE
        direction = 1 if self.rng.random() < 0.5 else -1
        ball.rights += direction > 0
        ball.vx = direction * HALF / fall_time(ball.vy, ROW_GAP)

    def step(self):
        self.clock += DT

        for ball in self.balls:
            if ball.landed_at >= 0:
                continue

            ball.vy += GRAVITY * DT
            ball.x += ball.vx * DT
            ball.y += ball.vy * DT

            if ball.level == FALLING_IN:
                # Dropped from above: a plain free fall onto the first peg.
                if ball.y >= perch_y(0):
                    ball.level, ball.y, ball.x = 0, perch_y(0), CX
                    self._deflect(ball)

            elif ball.level < ROWS:
                arrival = perch_y(ball.level + 1)
                if ball.y >= arrival:
                    ball.level += 1
                    ball.y = arrival
                    ball.x = slot_x(ball.level, ball.rights)   # stay on lattice
                    if ball.level < ROWS:
                        self._deflect(ball)
                    else:
                        ball.vx, ball.vy = 0.0, ball.vy * BOUNCE
            else:
                floor = BIN_BASE - self.bins[ball.rights] * BIN_UNIT - BALL_R
                if ball.y >= floor:
                    ball.y, ball.vy = floor, 0.0
                    ball.landed_at = self.clock

        # A ball that has sat still for a moment becomes part of the stack: it
        # stops being drawn and its bin is one taller. The Engine fades it out
        # as the bar grows, so it reads as settling in.
        for ball in self.balls:
            if 0 <= ball.landed_at <= self.clock - REST:
                self.bins[ball.rights] += 1
        self.balls = [b for b in self.balls
                      if not (0 <= b.landed_at <= self.clock - REST)]

        if self.released < BALLS and self.clock >= self.released * RELEASE_EVERY:
            self.balls.append(Ball(id=self.released, x=CX, y=SPAWN_Y))
            self.released += 1


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def drop(board):
    while not board.done():
        board.step()
        cm.emit("tick")
    cm.emit("settled")


# --- the view ---------------------------------------------------------------


def board_view(frame):
    scene = cm.Scene()
    board = frame.state

    scene.text("title", "Galton board", x=CX, y=56, size=38, color=INK)
    scene.text("subtitle", f"{BALLS} balls, {ROWS} rows, an even chance each way",
               x=CX, y=92, size=20, color="grey")

    # The hopper the balls are dropped from.
    for side in (-1, 1):
        scene.line(("hopper", side),
                   start=(CX + side * 54, SPAWN_Y - 26),
                   end=(CX + side * 15, SPAWN_Y + 14), w=3.0, color=PEG)

    # Pegs: named after where they are, so they never move. A ball's resting
    # place is the top of a peg, so these sit exactly on its path.
    for row in range(ROWS):
        for i in range(row + 1):
            scene.circle(("peg", row, i), x=slot_x(row, i), y=level_y(row),
                         r=PEG_R, color=PEG)

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

    return scene


# `linear`: the simulation is sampled densely enough that the parabola is
# already in the samples. Easing between them would add a wobble physics
# never asked for.
cm.explain(
    trace=drop(Board()),
    view=board_view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=DT, events={"settled": 0.8},
                     opening=1.0, final_hold=2.4),
).render("results/galton_board.mp4", fps=60, scale=1.5)

print("wrote results/galton_board.mp4")
