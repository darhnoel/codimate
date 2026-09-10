"""Putting it together: motion rules, timing, and the render call."""

from __future__ import annotations

from .layout import height, width
from .scene import Scene, _key
from typing import Callable

from .trace import Event, Frame, Trace


View = Callable[[Frame], Scene]

PATHS = ("straight", "lift_carry_drop")


class Rule:
    """How things matching ``pattern`` travel. First matching rule wins.

        cm.Rule("*", position="lift_carry_drop", clearance=90)

    ``pattern`` matches a shape's full name with ``*`` and ``?``. A shape
    inside a group is named ``group/child``, so ``"3/*"`` targets one group and
    ``"*"`` targets everything.
    """

    def __init__(self, pattern, position: str = "straight", **options: float) -> None:
        if position not in PATHS:
            raise ValueError(f"unknown path {position!r} — use one of {', '.join(PATHS)}")
        self.pattern = _key(pattern) if isinstance(pattern, tuple) else str(pattern)
        self.position = position
        self.options = {k: float(v) for k, v in options.items()}

    def _payload(self):
        return (self.pattern, self.position, self.options)

    def __repr__(self):
        return f"Rule({self.pattern!r}, position={self.position!r}, **{self.options})"


class Timing:
    """How long each event lasts, in seconds."""

    def __init__(
        self,
        *,
        default: float = 0.6,
        events: "dict[str, float] | None" = None,
        opening: float = 0.8,
        final_hold: float = 1.2,
    ) -> None:
        self.default = default
        self.events = events or {}
        self.opening = opening
        self.final_hold = final_hold

    def for_event(self, event: Event) -> float:
        return self.events.get(event.name, self.default)


def ease(t: float) -> float:
    """The easing curve the Engine applies between two moments.

        cm.ease(0.5)  ->  0.5

    This calls into the Engine, so it is the same curve your animation is
    actually using — not a copy of it.
    """
    from . import _codimate

    return _codimate.ease(float(t))


class Explanation:
    def __init__(
        self,
        *,
        trace: Trace,
        view: View,
        motion: "list[Rule] | None" = None,
        timing: "Timing | None" = None,
    ) -> None:
        self.timing = timing or Timing()
        self.motion = motion or []

        # The view runs once per event, not per frame — this is the whole
        # reason Python is fast enough to be the authoring language.
        self.scenes = [view(Frame(state=trace.initial, event=None))]
        for event in trace.events:
            self.scenes.append(view(Frame(state=event.state, event=event)))

        self.durations = [self.timing.for_event(e) for e in trace.events]

        # A held opening and a held ending, expressed as segments that go
        # nowhere. No special case in the Engine.
        self.scenes.insert(0, self.scenes[0])
        self.durations.insert(0, self.timing.opening)
        self.scenes.append(self.scenes[-1])
        self.durations.append(self.timing.final_hold)

    @property
    def duration(self) -> float:
        return sum(self.durations)

    def render(self, output: str, *, fps: float = 30) -> str:
        """Draw every frame and write the video. Size comes from `cm.canvas()`.

        The folder is created if it does not exist, so `render("results/x.mp4")`
        works on a fresh clone.
        """
        from pathlib import Path

        from . import _codimate  # imported here so the pure Python is testable

        Path(output).expanduser().resolve().parent.mkdir(parents=True, exist_ok=True)

        _codimate.render(
            scenes=[s._payload() for s in self.scenes],
            rules=[r._payload() for r in self.motion],
            durations=self.durations,
            output=output,
            width=width(),
            height=height(),
            fps=float(fps),
        )
        return output


def explain(
    *,
    trace: Trace,
    view: View,
    motion: "list[Rule] | None" = None,
    timing: "Timing | None" = None,
) -> Explanation:
    return Explanation(trace=trace, view=view, motion=motion, timing=timing)
