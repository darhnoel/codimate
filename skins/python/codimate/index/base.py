"""The Index interface, and its in-memory forms: a dict, and a live film."""

from __future__ import annotations



class Index:
    """What the Previewer asks of an index.

    `canvas` is the size the boxes are in; `beats` has one dict per beat — at, secs,
    event, state, covered, and chapter where one starts — and may carry more, such as
    the legacy `shapes`. `shapes_at(k)` is every shape on screen once beat `k` has
    arrived, as `{"name", "kind", "layer", "box"}`, in no particular order.
    """

    canvas: "list | None" = None
    beats: "list" = ()

    def shapes_at(self, beat: int) -> list:
        raise NotImplementedError

    def changes(self, start: int = 0, stop: "int | None" = None):
        """The beats in the shape `Explanation.index()` gives: shapes as what arrived
        or moved and what left, so the output can be written as the JSON file."""
        stop = len(self.beats) if stop is None else stop
        before = {s["name"]: s for s in self.shapes_at(start - 1)} if start > 0 else {}
        for k in range(start, stop):
            now = {s["name"]: s for s in self.shapes_at(k)}
            beat = {key: v for key, v in self.beats[k].items() if key != "shapes"}
            beat["shapes"] = {"set": [s for n, s in now.items() if before.get(n) != s],
                              "gone": [n for n in before if n not in now]}
            yield beat
            before = now

    def starts(self) -> list:
        """When each beat starts, in seconds, for finding the one a moment falls in."""
        if not hasattr(self, "_starts"):
            self._starts = [b["at"] for b in self.beats]
        return self._starts


class DictIndex(Index):
    """An index held as the dict `Explanation.index()` or an `.index.json` gives.

    Shapes are stored as differences, so asking for a beat replays them from the
    nearest checkpoint, kept every 32 beats in the dict itself.
    """

    def __init__(self, data: dict):
        self.data = data
        self.canvas = data.get("canvas")
        self.beats = data["beats"]

    def shapes_at(self, beat: int) -> list:
        marks = self.data.setdefault("_checkpoints", {})
        start = max((k for k in marks if k <= beat), default=-1)
        now = dict(marks[start]) if start >= 0 else {}
        for k in range(start + 1, beat + 1):
            changed = self.beats[k]["shapes"]
            for shape in changed["set"]:
                now[shape["name"]] = shape
            for name in changed["gone"]:
                now.pop(name, None)
            if k % 32 == 31:
                marks[k] = dict(now)
        return list(now.values())

    def changes(self, start: int = 0, stop: "int | None" = None):
        yield from self.beats[start:stop]


class LiveIndex(Index):
    """The index of a film held in memory, answered from its own scenes.

    Only the per-beat table is built, up front; the shapes at a beat are read off the
    scene that beat arrives at, when asked.
    """

    def __init__(self, explanation):
        from ..layout import height, width
        self.explanation = explanation
        self.canvas = [width(), height()]
        self.beats = list(explanation._index_beats(shapes=False))

    def shapes_at(self, beat: int) -> list:
        return self.explanation._shapes_after(beat)

    def changes(self, start: int = 0, stop: "int | None" = None):
        yield from list(self.explanation._index_beats())[start:stop]
