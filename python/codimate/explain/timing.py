"""Motion rules and timing: how a change moves, and how long it takes."""

from __future__ import annotations

from ..scene import _key
from ..trace import Event


PATHS = ("straight", "arc", "linear", "fall", "lift_carry_drop")
EASES = ("linear", "in", "out", "in_out", "back", "spring", "bounce")
ORDERS = ("screen", "reverse", "name", "centre")


class Rule:
    """How things matching ``pattern`` change. First matching rule wins.

        cm.Rule("*", position="lift_carry_drop", clearance=90)
        cm.Rule("bar/*", ease="spring", stagger=0.08)
        cm.Rule("card", position="arc", bend=-0.4, ease="out")

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
    * ``arc`` — a curve bowed to one side. ``bend`` is how far, as a fraction of
      the distance travelled (default 0.3); negative bows the other way.

    Timing and feel, for everything about the shape — where it is, its size,
    colour, opacity, turn, and a marker's reveal — not only its position:

    * ``ease`` — ``linear``, ``in``, ``out``, ``in_out``, ``back`` (overshoots and
      settles), ``spring`` (takes ``bounciness`` 0..1) or ``bounce``. Unset, a
      shape moves as it always has: eased position, the rest linear.
    * ``delay`` — wait this fraction of the beat before starting.
    * ``stagger`` — matching shapes start this fraction of the beat apart, in
      ``order``: ``screen`` (left to right, then top to bottom; the default),
      ``reverse``, ``name`` or ``centre`` (from the middle out). They all take
      the same time, and the last one finishes as the beat ends.
    """

    def __init__(self, pattern, position: str = "straight", *, ease: str = None,
                 order: str = "screen", **options: float) -> None:
        if position not in PATHS:
            raise ValueError(
                f"unknown path {position!r} — use one of {', '.join(PATHS)}")
        if ease is not None and ease not in EASES:
            raise ValueError(f"unknown ease {ease!r} — use one of {', '.join(EASES)}")
        if order not in ORDERS:
            raise ValueError(
                f"unknown order {order!r} — use one of {', '.join(ORDERS)}")
        for name in ("delay", "stagger"):
            if not 0.0 <= float(options.get(name, 0.0)) < 1.0:
                raise ValueError(f"{name} is a fraction of the beat, 0 to under 1")
        self.pattern = _key(pattern) if isinstance(pattern, tuple) else str(pattern)
        self.position = position
        self.ease = ease
        self.order = order
        self.options = {k: float(v) for k, v in options.items()}

    def _payload(self):
        return (self.pattern, self.position, self.options, self.ease or "", self.order)

    def __repr__(self):
        extra = f", ease={self.ease!r}" if self.ease else ""
        if self.order != "screen":
            extra += f", order={self.order!r}"
        return (f"Rule({self.pattern!r}, position={self.position!r}{extra}, "
                f"**{self.options})")


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
