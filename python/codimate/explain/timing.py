"""Motion rules and timing: how a change moves, and how long it takes."""

from __future__ import annotations

from ..scene import _key
from ..trace import Event


PATHS = ("straight", "linear", "fall", "lift_carry_drop")


class Rule:
    """How things matching ``pattern`` travel. First matching rule wins.

        cm.Rule("*", position="lift_carry_drop", clearance=90)

    ``pattern`` matches a shape's full name with ``*`` and ``?``. A shape
    inside a group is named ``group/child``, so ``"3/*"`` targets one group and
    ``"*"`` targets everything.

    Paths:

    * ``straight`` — a straight line, easing in and out. The default, and what
      you want when each event is a distinct step.
    * ``linear`` — a straight line at constant speed. Use it when a thing is
      mid-journey at every event, like something turning: easing would make it
      accelerate and stop inside each segment.
    * ``fall`` — a parabola: sideways at a constant rate, downwards
      accelerating. What a dropped thing does, and what each hop of a falling
      ball needs, since an eased path would settle gently instead of arriving.
    * ``lift_carry_drop`` — arcs up and over, then falls. Takes ``clearance``.
    """

    def __init__(self, pattern, position: str = "straight", **options: float) -> None:
        if position not in PATHS:
            raise ValueError(
                f"unknown path {position!r} — use one of {', '.join(PATHS)}")
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
        """How long ``event`` lasts: its own entry, or ``default``.

        A name with no entry takes ``default`` silently — which is what a
        default is for, but it means a misspelled event name costs you the
        default duration rather than an error.
        """
        return self.events.get(event.name, self.default)


def ease(t: float) -> float:
    """The easing curve the Engine applies between two moments.

        cm.ease(0.5)  ->  0.5

    This calls into the Engine, so it is the same curve your animation is
    actually using — not a copy of it.
    """
    from .. import _codimate

    return _codimate.ease(float(t))
