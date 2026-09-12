"""Where every body is, tick by tick.

Plain Kepler circles around a Sun that is itself moving. No Codimate here.
"""

import math
from dataclasses import dataclass, field

from space import DT, PLANETS, TRAIL, TRAVEL, orbit_point


@dataclass
class Body:
    name: str
    here: tuple                      # world position now
    history: list = field(default_factory=list)   # newest last, with its tick


class System:
    def __init__(self):
        self.tick = 0
        self.years = 0.0
        self.sun = Body("Sun", (0.0, 0.0, 0.0))
        self.planets = [Body(name, (0.0, 0.0, 0.0)) for name, *_ in PLANETS]
        self._place()

    def _place(self):
        """Every body's position at the current time."""
        travelled = TRAVEL * self.years
        self.sun.here = (travelled, 0.0, 0.0)

        for body, (_, radius, period, *_rest) in zip(self.planets, PLANETS):
            angle = 2.0 * math.pi * self.years / period
            local = orbit_point(radius, angle)
            body.here = (travelled + local[0], local[1], local[2])

    def step(self):
        for body in (self.sun, *self.planets):
            body.history.append((self.tick, body.here))
            del body.history[:-TRAIL]

        self.tick += 1
        self.years += DT
        self._place()
