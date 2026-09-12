"""The simulation: gravity, pegs, dividers, and balls with bodies.

Nothing here knows it is being drawn. It is a physics model that happens to be
animatable, which is the point — the view reads its state, it does not serve
the view.
"""

import math
import random
from dataclasses import dataclass

from board import (BALLS, BALL_R, BIN_BASE, BIN_UNIT, CX, FALLING_IN, HALF,
                   ROWS, ROW_GAP, SPAWN_Y, perch_y, slot_x)

# Physics, in pixels and seconds.
GRAVITY = 1400.0
BOUNCE = 0.5                    # share of downward speed kept after a peg
DT = 0.025                      # simulation and sampling step
RELEASE_EVERY = 0.028
REST = 0.05                     # how long a ball sits before joining its bin
SCATTER = 6.0                   # how far off a peg's centre a ball may come off
SPAWN_SPREAD = 17.0             # balls do not leave the hopper single file
KNOCK = 0.35                    # bounce between two balls; they are not lively
WALL_BOUNCE = 0.45              # off a divider between bins
PASSES = 3                      # separation sweeps per step; one leaves clumps

SEED = 3                        # fixed, so the video is the same every render


def fall_time(vy, height):
    """How long to fall `height`, starting at downward speed `vy`."""
    return (-vy + math.sqrt(vy * vy + 2 * GRAVITY * height)) / GRAVITY


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
        self.next_release = 0.0
        self.balls = []
        self.bins = [0] * (ROWS + 1)

    def done(self):
        return sum(self.bins) == BALLS

    def _deflect(self, ball):
        """A peg: takes vertical speed, gives sideways speed.

        The sideways speed is whatever carries the ball to the next peg by the
        time gravity has taken it down a row — computed from where the ball
        actually is, so a ball that came off the last peg slightly askew still
        arrives at the next one.

        No two balls strike a peg in quite the same place, so each comes off it
        a little differently. The bin a ball lands in is decided by the count
        of rights, never by where it drifted to, so the statistics stay exactly
        binomial while the paths stop looking like a machine.
        """
        ball.vy *= BOUNCE
        direction = 1 if self.rng.random() < 0.5 else -1
        ball.rights += direction > 0

        landing = ball.level + 1
        target = slot_x(landing, ball.rights)
        if landing < ROWS:      # the last row aims true, into its bin
            target += self.rng.uniform(-SCATTER, SCATTER)

        ball.vx = (target - ball.x) / fall_time(ball.vy, ROW_GAP)

    def _jostle(self):
        """Balls have bodies: no two of them share a space.

        Overlaps are pushed apart and the velocity along the line between the
        centres is exchanged, as equal masses do. A ball already resting in a
        bin does not move — it is held up by the pile beneath it.

        This can shove a ball well off the peg it was aiming for, which is
        fine: the next deflection is computed from where the ball actually is,
        and the bin it ends in is decided by its count of rights, never by
        where it drifted to.
        """
        for _ in range(PASSES):
          for i, a in enumerate(self.balls):
            for b in self.balls[i + 1:]:
                  dx, dy = b.x - a.x, b.y - a.y
                  gap = dx * dx + dy * dy
                  span = 2 * BALL_R
                  if gap >= span * span or gap < 1e-9:
                      continue

                  distance = math.sqrt(gap)
                  nx, ny = dx / distance, dy / distance
                  overlap = span - distance

                  a_fixed, b_fixed = a.landed_at >= 0, b.landed_at >= 0
                  if a_fixed and b_fixed:
                      continue

                  # Push apart: a settled ball holds its ground.
                  share_a = 0.0 if a_fixed else (1.0 if b_fixed else 0.5)
                  a.x -= nx * overlap * share_a
                  a.y -= ny * overlap * share_a
                  b.x += nx * overlap * (1.0 - share_a)
                  b.y += ny * overlap * (1.0 - share_a)

                  closing = (a.vx - b.vx) * nx + (a.vy - b.vy) * ny
                  if closing <= 0:
                      continue            # already moving apart
                  impulse = closing * (1.0 + KNOCK) * (0.5 if not (a_fixed or b_fixed) else 1.0)
                  if not a_fixed:
                      a.vx -= impulse * nx
                      a.vy -= impulse * ny
                  if not b_fixed:
                      b.vx += impulse * nx
                      b.vy += impulse * ny

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
                    ball.level, ball.y = 0, perch_y(0)
                    self._deflect(ball)

            elif ball.level < ROWS:
                arrival = perch_y(ball.level + 1)
                if ball.y >= arrival:
                    ball.level += 1
                    ball.y = arrival
                    if ball.level < ROWS:
                        self._deflect(ball)
                    else:
                        # Momentum is not confiscated at the last peg. The ball
                        # keeps whatever sideways speed it came off with; the
                        # dividers below are what keep it in its own column,
                        # which is what those dividers are for on a real board.
                        ball.vy *= BOUNCE
            else:
                # Below the pegs: bouncing down inside its own bin.
                centre = slot_x(ROWS, ball.rights)
                reach = HALF - BALL_R
                if ball.x < centre - reach:
                    ball.x, ball.vx = centre - reach, -ball.vx * WALL_BOUNCE
                elif ball.x > centre + reach:
                    ball.x, ball.vx = centre + reach, -ball.vx * WALL_BOUNCE

                floor = BIN_BASE - self.bins[ball.rights] * BIN_UNIT - BALL_R
                if ball.y >= floor:
                    ball.y, ball.vy, ball.vx = floor, 0.0, 0.0
                    ball.landed_at = self.clock

        self._jostle()

        # A ball that has sat still for a moment becomes part of the stack: it
        # stops being drawn and its bin is one taller. The Engine fades it out
        # as the bar grows, so it reads as settling in.
        for ball in self.balls:
            if 0 <= ball.landed_at <= self.clock - REST:
                self.bins[ball.rights] += 1
        self.balls = [b for b in self.balls
                      if not (0 <= b.landed_at <= self.clock - REST)]

        # A `while`, not an `if`: the release interval is finer than DT, and an
        # `if` would silently round every release up to a whole step — which
        # made the stream take nearly twice as long as asked for.
        while self.released < BALLS and self.clock >= self.next_release:
            self.balls.append(Ball(
                id=self.released,
                x=CX + self.rng.uniform(-SPAWN_SPREAD, SPAWN_SPREAD),
                y=SPAWN_Y,
                vx=self.rng.uniform(-18.0, 18.0),
            ))
            self.released += 1
            # Jittered, or the stream queues into a bead string down the middle.
            # Accumulated, not measured from the clock, so rounding does not drift.
            self.next_release += RELEASE_EVERY * self.rng.uniform(0.55, 1.45)
