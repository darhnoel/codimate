"""Drawing the system: helices first, then the bodies riding their heads."""

import random

from space import (INK, PLANETS, STAR_FAR, STAR_NEAR, STAR_SEED, STARS,
                   SUN_COLOR, SUN_R, TRAIL, project, travel_on_screen)

# Where each star sits and how far away it is. Laid out well beyond the frame
# so the near ones can stream across it without ever running out.
_sky = random.Random(STAR_SEED)
SKY = [(_sky.uniform(-260, 1560), _sky.uniform(-220, 960),
        _sky.uniform(0.7, 2.2), _sky.uniform(STAR_FAR, STAR_NEAR))
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
                   w=width * (0.45 + 0.75 * age),
                   color=color, opacity=0.10 + 0.90 * age)


def draw(scene, system):
    # The sky streams backwards past a camera that is travelling, and a nearer
    # star streams faster. Without this the Sun would appear to be standing
    # still, which is the one thing this animation is arguing against.
    drift_x, drift_y = travel_on_screen(system.years)
    for i, (x, y, size, nearness) in enumerate(SKY):
        scene.circle(("star", i),
                     x=x - drift_x * nearness, y=y - drift_y * nearness,
                     r=size, color="white",
                     opacity=0.25 + 2.4 * nearness)

    scene.text("title", "The helical model", x=640, y=44, size=34, color=INK)
    scene.text("subtitle",
               "the Sun moves, so every orbit is a helix — "
               "and the plane is inclined, not square to the travel",
               x=640, y=80, size=17, color="grey")

    camera = system.sun.here
    _helix(scene, "sun", system.sun.history, SUN_TRAIL, 2.6, camera)
    for body, (_, _, _, _, color) in zip(system.planets, PLANETS):
        _helix(scene, body.name, body.history, color, 2.6, camera)

    # The Sun last of the trails, so the planets' helices read against it.
    sx, sy = project(system.sun.here, camera)
    scene.circle("sun_glow", x=sx, y=sy, r=SUN_R * 1.9, color=SUN_COLOR,
                 opacity=0.18, layer=8)
    scene.circle("sun", x=sx, y=sy, r=SUN_R, color=SUN_COLOR, layer=9)

    for body, (_, _, _, size, color) in zip(system.planets, PLANETS):
        px, py = project(body.here, camera)
        scene.circle(("planet", body.name), x=px, y=py, r=size,
                     color=color, layer=10)
