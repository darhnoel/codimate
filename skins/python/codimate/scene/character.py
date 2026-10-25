"""Scene-local handles for posing an SVG-backed character."""

from __future__ import annotations

import json
import math
from dataclasses import replace

from .handle import Handle


class Character(Handle):
    """A connected figure whose pose belongs to this Scene."""

    def part(self, name: str) -> "CharacterPart":
        """Select a joint by the `id` of its marked SVG group."""
        if not name:
            raise ValueError("a character part needs a non-empty SVG group id")
        return CharacterPart(self._shapes, self._name, name)

    def expression(
        self, name: str, *, transition: str = "crossfade"
    ) -> "Character":
        """Choose an expression, crossfading or cutting to it between Scenes."""
        if not name:
            raise ValueError("a character expression needs a non-empty name")
        if transition not in {"crossfade", "cut"}:
            raise ValueError("expression transition must be 'crossfade' or 'cut'")
        pose = self._pose()
        pose["expression"] = name
        if transition == "cut":
            pose["expression_transition"] = transition
        else:
            pose.pop("expression_transition", None)
        self._store_pose(pose)
        return self

    def _pose(self) -> dict:
        raw = self._shape().get("pose", "")
        return json.loads(raw) if raw else {"expression": None, "joints": {}}

    def _store_pose(self, pose: dict) -> None:
        shape = self._shapes[self._name]
        data = json.dumps(
            pose, ensure_ascii=False, separators=(",", ":"), sort_keys=True
        )
        self._shapes[self._name] = replace(shape, pose=data)


class CharacterPart:
    """A joint handle. Its children follow when its angle changes."""

    def __init__(self, shapes: dict, item: str, name: str):
        self._shapes = shapes
        self._item = item
        self._name = name

    def turn(self, degrees: float) -> "CharacterPart":
        """Set this joint's angle in degrees for the current Scene."""
        degrees = float(degrees)
        if not math.isfinite(degrees):
            raise ValueError("a character joint angle must be finite")
        character = Character(self._shapes, self._item)
        pose = character._pose()
        joints = pose.setdefault("joints", {})
        joints[self._name] = degrees
        character._store_pose(pose)
        return self
