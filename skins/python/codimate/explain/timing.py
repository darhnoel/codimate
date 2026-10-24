"""Motion rules and timing: how a change moves, and how long it takes."""

from __future__ import annotations

from ..trace import Event



# Rule is built by the Authoring Core (`rule` in api/surface.toml): checked
# there once, for every language.
from .._surface import Rule  # noqa: E402, F401


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


def ease(t):
    """The easing curve the Engine applies between two moments. Worked out by
    the Authoring Core; see `ease` in api/surface.toml."""
    from .._surface import ease as answer

    return answer(t)


def __getattr__(name):
    """PATHS, EASES and ORDERS: what a Rule may say, as the Core lists them,
    so there is one list and it is the one that checks."""
    if name in ("PATHS", "EASES", "ORDERS"):
        from .._waist import call

        return tuple(call("choices", {})[name.lower()])
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
