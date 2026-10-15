"""What happened: Items with identity, and the Trace an algorithm records."""

from __future__ import annotations

import copy
import itertools
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


def _emitter(rec: "_Recorder") -> Callable[..., None]:
    """The `emit` handed to an algorithm: record that something happened."""

    def emit(name: str, **data: Any) -> None:
        """Record that something just happened.

        Call it *after* changing your data — Codimate snapshots the result for you.
        """
        rec.events.append(Event(name=name, state=rec.snapshot(), data=data))

    return emit


def trace(fn, state, /, *, snapshot=None) -> "Trace":
    """Run `fn` over `state` and record what happened.

        def bubble_sort(values, emit):
            ...
            emit("compare", items=[a, b])

        cm.explain(trace=cm.trace(bubble_sort, cm.items([3, 1, 4, 2])), ...)

    Your function is handed the state and an `emit`. Call `emit` *after*
    changing the state — Codimate snapshots the result for you.

    `emit` is an argument rather than something ambient, so the answer to
    "where did that come from?" is the line above it, and so the function
    stays ordinary Python: `bubble_sort(values, print)` runs it and prints the
    events.

    Write the state as a dataclass, so the algorithm and the view read it as
    `state.x` and a misspelled field is an error rather than a new key::

        @dataclass
        class Ball:
            x: float = 100

        cm.trace(move, Ball())

    Any object that `copy.deepcopy` handles works; a list of Items is the
    usual state for things that move, and a dict is fine for a quick sketch.

    `snapshot(state)` returns the part worth showing, if a deep copy of the
    whole state is not what you want. Items keep their identity through the
    default copy.
    """

    def capture():
        return snapshot(state) if snapshot is not None else copy.deepcopy(state)

    rec = _Recorder(snapshot=capture, events=[])
    initial = capture()
    fn(state, _emitter(rec))
    return Trace(initial=initial, events=tuple(rec.events))


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

        Use it when you emitted a list: ``emit("compare", items=[a, b])``.
        Empty for the opening moment.
        """
        if self.event is None:
            return []
        return list(self.event.data.get(key, ()))
