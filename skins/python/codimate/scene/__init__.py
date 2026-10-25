"""What one moment looks like: Groups, Scenes, and the flat shape payload.

A package of small parts behind this one front door:

- `shape`    — the flat payload record the Engine diffs, and names (`_key`)
- `handle`   — what a shape call returns: fill, turn, grow, on, write
- `group`    — the shapes you draw, placed relative to a group's point
- markers and highlights are drawn by the Authoring Core (ADR 0027, 0032)
- `coverage` — where a shape sits, and which labels are drawn over
- `root`     — `Scene`, the root group
"""

from .coverage import box, covered
from .character import Character, CharacterPart
from .group import Group
from .handle import Handle
from .root import Scene
from .shape import PIVOTS, _key, _Shape

__all__ = [
    "Character",
    "CharacterPart",
    "Group",
    "Handle",
    "PIVOTS",
    "Scene",
    "_Shape",
    "_key",
    "box",
    "covered",
]
