"""Drawing the system: helices first, then the bodies riding their heads."""

import random

from space import (INK, PLANETS, STAR_SEED, STARS, SUN_COLOR, SUN_R, TRAIL,
                   project)

# Drawn once here rather than every frame: the sky does not change.
_sky = random.Random(STAR_SEED)
SKY = [(_sky.uniform(20, 1260), _sky.uniform(20, 700),
        _sky.uniform(0.7, 2.1), _sky.uniform(0.25, 0.85))
       for _ in range(STARS)]

SUN_TRAIL = "#f0c86a"


def _helix(scene, key, history, color, width, camera):
    """A body's recent path, one line per tick, fading out behind it.

    Each segment is named after the tick it records, so it is fixed the moment
    it is drawn — a trail is a record of where something was, not a thing that
    moves. New segments fade in at the head; the oldest fade out at the tail,
    which is all the Engine needs to be told.
    """
    for (tick, here), (_, there) in zip(history, history[1:]):
        age = (tick - history[0][0]) / TRAIL
        scene.line(("trail", key, tick),
                   start=project(here, camera), end=project(there, camera),
                   w=width, color=color, opacity=0.10 + 0.90 * age)


def draw(scene, system):
    # Stars are named after where they are and never move — the system moves
    # against them, which is the only reason the travel is visible at all.
    for i, (x, y, size, glow) in enumerate(SKY):
        scene.circle(("star", i), x=x, y=y, r=size, color="white", opacity=glow)

    scene.text("title", "The helical model", x=640, y=58, size=36, color=INK)
    scene.text("subtitle",
               "the Sun moves, so every orbit is a helix — "
               "and the plane is inclined, not square to the travel",
               x=640, y=98, size=18, color="grey")

    camera = system.sun.here
    _helix(scene, "sun", system.sun.history, SUN_TRAIL, 2.0, camera)
    for body, (_, _, _, _, color) in zip(system.planets, PLANETS):
        _helix(scene, body.name, body.history, color, 2.0, camera)

    # The Sun last of the trails, so the planets' helices read against it.
    sx, sy = project(system.sun.here, camera)
    scene.circle("sun_glow", x=sx, y=sy, r=SUN_R * 1.9, color=SUN_COLOR,
                 opacity=0.18, layer=8)
    scene.circle("sun", x=sx, y=sy, r=SUN_R, color=SUN_COLOR, layer=9)

    for body, (_, _, _, size, color) in zip(system.planets, PLANETS):
        px, py = project(body.here, camera)
        scene.circle(("planet", body.name), x=px, y=py, r=size,
                     color=color, layer=10)
