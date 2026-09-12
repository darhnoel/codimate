"""Particles carried by the flow, and the pair that tests the old story.

Nothing here decides where the air goes — `airfoil.velocity` does, and it is
the exact solution. This only carries things along it.
"""

import math
import random
from dataclasses import dataclass, field

from airfoil import velocity

DT = 0.030
SPAN_X = (-4.15, 4.45)           # where parcels enter and leave
LANES = 15
LANE_Y = (-1.615, 0.972)    # solved: clear of the text, top and bottom
RELEASE_EVERY = 0.34
SEED = 5
UNTIL = 13.0

# The streamline that actually splits at the nose passes through here, measured
# by bisection AT THE RELEASE LINE — it is not zero, and it is not the same
# value further downstream, because circulation pulls it up from well below.
SPLIT_Y = -0.9526

# The band of air that is shown. The limits are not guessed: each was solved
# for by tracing its streamline and asking where it reaches on the screen, so
# the flow stays clear of the title above and the captions below. No air
# crosses a streamline, so a parcel released inside this band stays inside it —
# nothing needs to be clamped.
MARK_AT = 1.2                    # when the marked pair is released
MARK_GAP = 0.05


@dataclass
class Parcel:
    id: int
    x: float
    y: float
    speed: float = 0.0
    marked: bool = False         # one of the pair that split at the nose
    side: str = ""
    born: float = 0.0
    trail: list = field(default_factory=list)


def _advance(parcel, dt):
    """Midpoint step along the flow — Euler alone drifts through the wing."""
    first = velocity(complex(parcel.x, parcel.y))
    if first is None:
        return False
    half = velocity(complex(parcel.x + first[0] * dt * 0.5,
                            parcel.y + first[1] * dt * 0.5))
    if half is None:
        return False
    parcel.x += half[0] * dt
    parcel.y += half[1] * dt
    parcel.speed = math.hypot(*half)
    return True


def streamline(y0, dt=0.02):
    """Follow the flow from the entry line to the exit, and keep the path."""
    parcel = Parcel(-1, SPAN_X[0], y0)
    path = [(parcel.x, parcel.y)]
    while parcel.x < SPAN_X[1]:
        if not _advance(parcel, dt):
            break
        path.append((parcel.x, parcel.y))
    return path


@dataclass
class Flow:
    rng: random.Random = field(default_factory=lambda: random.Random(SEED))
    clock: float = 0.0
    next_release: float = 0.0
    released: int = 0
    pair_released: bool = False
    parcels: list = field(default_factory=list)
    arrived: dict = field(default_factory=dict)   # side -> when it passed the tail

    def _release_row(self):
        """A ragged row, not a rank.

        Evenly spaced parcels released on a fixed beat draw a lattice that
        marches, and a lattice does not look like air. The jitter is smaller
        than the lane spacing, so the flow stays legible.
        """
        lo, hi = LANE_Y
        spacing = (hi - lo) / (LANES - 1)
        for lane in range(LANES):
            y = lo + spacing * lane + self.rng.uniform(-0.34, 0.34) * spacing
            x = SPAN_X[0] + self.rng.uniform(0.0, 0.30)
            self.parcels.append(Parcel(self.released, x, y))
            self.released += 1

    def _release_pair(self):
        """Two parcels side by side, either side of the dividing streamline.

        Released together, so when each reaches the trailing edge is a
        measurement rather than a claim.
        """
        for side, y in (("over", SPLIT_Y + MARK_GAP), ("under", SPLIT_Y - MARK_GAP)):
            parcel = Parcel(self.released, SPAN_X[0], y, marked=True)
            parcel.side = side
            self.parcels.append(parcel)
            self.released += 1

    def done(self):
        return self.clock >= UNTIL

    def step(self):
        self.clock += DT

        for parcel in list(self.parcels):
            was_before_tail = parcel.x < 2.0
            if not _advance(parcel, DT) or parcel.x > SPAN_X[1]:
                self.parcels.remove(parcel)
                continue
            if parcel.marked:
                parcel.trail.append((parcel.x, parcel.y))
                if was_before_tail and parcel.x >= 2.0 and parcel.side not in self.arrived:
                    self.arrived[parcel.side] = self.clock - parcel.born

        if self.clock >= self.next_release:
            self._release_row()
            self.next_release += RELEASE_EVERY

        if not self.pair_released and self.clock >= MARK_AT:
            self._release_pair()
            for parcel in self.parcels:
                if parcel.marked:
                    parcel.born = self.clock
            self.pair_released = True
