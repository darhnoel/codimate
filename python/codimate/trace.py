"""What happened: Items with identity, and the Trace an algorithm records."""

from __future__ import annotations

import copy
import itertools
from contextvars import ContextVar
from dataclasses import dataclass, field
from typing import Any, Callable


_next_id = itertools.count()


class Item:
    """A value with an identity of its own.

    Two 3s in a list are two different bars, and only an identity can say so.
    An Item compares by **value** (so your algorithm sorts normally) and is
    equal by **id** (so it is still itself after Codimate snapshots the state).
    """

    __slots__ = ("id", "value")

    def __init__(self, value: Any, id: "int | str | None" = None) -> None:
        self.value = value
        self.id = next(_next_id) if id is None else id

    # Ordering follows the value, so `items[j] > items[j + 1]` reads normally.
    # Do NOT replace these with functools.total_ordering: it derives __le__ as
    # `__lt__ or __eq__`, and __eq__ here compares ids, so Item(3) <= Item(3)
    # would come out False.
    def __lt__(self, other):
        return self.value < (other.value if isinstance(other, Item) else other)

    def __gt__(self, other):
        return self.value > (other.value if isinstance(other, Item) else other)

    def __le__(self, other):
        return self.value <= (other.value if isinstance(other, Item) else other)

    def __ge__(self, other):
        return self.value >= (other.value if isinstance(other, Item) else other)

    # Identity follows the id, which survives the deep copy taken at each
    # event. Comparing by `is` would fail across that boundary.
    def __eq__(self, other):
        return isinstance(other, Item) and self.id == other.id

    def __hash__(self):
        return hash(self.id)

    def __repr__(self):
        return f"Item({self.value!r})"


def items(values) -> "list[Item]":
    """Give each value an identity of its own.

        values = cm.items([3, 1, 4, 2])

    Use it when the things on screen should be able to *move* — two equal
    values are still two different things. For a grid whose cells stay put,
    you do not need this: key those on their position instead.
    """
    return [Item(v) for v in values]

@dataclass(frozen=True)
class Event:
    name: str
    state: Any
    data: "dict[str, Any]" = field(default_factory=dict)


@dataclass(frozen=True)
class Trace:
    initial: Any
    events: "tuple[Event, ...]"


@dataclass
class _Recorder:
    snapshot: Callable[[], Any]
    events: "list[Event]"


_recorder: ContextVar["_Recorder | None"] = ContextVar(
    "codimate_recorder", default=None)


def emit(name: str, **data: Any) -> None:
    """Record that something just happened.

    Call it *after* changing your data — Codimate snapshots the result for you.
    """
    rec = _recorder.get()
    if rec is None:
        raise RuntimeError("emit() called outside a @trace function")
    rec.events.append(Event(name=name, state=rec.snapshot(), data=data))


def trace(*, snapshot: Callable[..., Any] = None):
    """Turn a normal, state-mutating function into a Trace.

    ``snapshot`` receives the same arguments as your function and returns a
    copy of the data worth showing. Defaults to a deep copy of the first
    argument — Items keep their identity through it.
    """

    def decorator(fn):
        def run(*args, **kwargs) -> Trace:
            def capture():
                if snapshot is not None:
                    return snapshot(*args, **kwargs)
                return copy.deepcopy(args[0]) if args else None

            rec = _Recorder(snapshot=capture, events=[])
            initial = capture()

            token = _recorder.set(rec)
            try:
                fn(*args, **kwargs)
            finally:
                _recorder.reset(token)

            return Trace(initial=initial, events=tuple(rec.events))

        run.__name__ = fn.__name__
        run.__doc__ = fn.__doc__
        return run

    return decorator

@dataclass(frozen=True)
class Frame:
    """What your view function receives: the data, and what just happened.

    ``event`` is ``None`` for the opening moment, before anything has happened.
    """

    state: Any
    event: "Event | None"

    def is_(self, name: str) -> bool:
        """Did this moment come from an event called ``name``?"""
        return self.event is not None and self.event.name == name


    def items(self, key: str = "items") -> list:
        """The things this event named, e.g. ``frame.items()``.

        Use it when you emitted a list: ``cm.emit("compare", items=[a, b])``.
        Empty for the opening moment.
        """
        if self.event is None:
            return []
        return list(self.event.data.get(key, ()))
