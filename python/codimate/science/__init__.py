"""The science kit — the marks and the space that explaining physics keeps needing.

Optional, and separate from the core on purpose: nothing here is needed to
make a Codimate film, and none of it is in the Engine. It is the Authoring
Surface's own arithmetic, kept in one place instead of copied into every
sketch (ADR 0019).

    from codimate import science

Every piece follows `cm.axes` (ADR 0016): it works out pixels and hands them
back, and where it draws it draws ordinary named shapes, so they tween, can be
framed by `focus`, and can be aimed at by a motion Rule.

## Marks — flat things you put on a picture

- `tag` — a label on a plate, sized to its content; `Tag.leader` points at its edge
- `Glow` — a soft halo round one dot or one ring
- `wave` — the points of a wave between two places
- `Bracket` — a measured span, with its label beside it
- `ForceScale` — one scale for every arrow in a film

## Space — things in three dimensions, drawn flat in the right order

- `Camera` — where you look from; turns a point in space into a pixel
- `Sphere`, `Light` — a ball, and what lights it
- `Orbit` — a real ellipse with the orbited body at a focus
- `World` — draws bodies and orbits together, sorted far to near
- `RingArrows` — arrows riding a body's equator, so its spin can be seen
- `icosphere` — the mesh under a sphere
"""

from __future__ import annotations

from .marks import Bracket, ForceScale, Glow, Tag, tag, wave
from .space import (ABOVE, Camera, Light, Orbit, RingArrows, Sphere, World,
                    icosphere)

__all__ = [
    # marks
    "tag",
    "Tag",
    "Glow",
    "wave",
    "Bracket",
    "ForceScale",
    # space
    "Camera",
    "Sphere",
    "Light",
    "Orbit",
    "World",
    "RingArrows",
    "icosphere",
    "ABOVE",
]
