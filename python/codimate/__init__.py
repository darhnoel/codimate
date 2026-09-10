"""Codimate — turn a running algorithm into an explainer video.

You write four things:

    algorithm   your normal code, with emit() where something happens
    view        what one moment looks like
    motion      how things travel between moments
    timing      how long each moment lasts

Everything per-frame — diffing, interpolation, drawing, encoding — happens in
Rust. See docs/adr/0008-python-authoring-surface.md.
"""

from __future__ import annotations

import copy
import itertools
from contextvars import ContextVar
from dataclasses import asdict, dataclass, field
from typing import Any, Callable, Hashable

__all__ = [
    "emit",
    "trace",
    "items",
    "Item",
    "Scene",
    "Group",
    "Frame",
    "Event",
    "Trace",
    "Rule",
    "Timing",
    "explain",
    "canvas",
    "width",
    "height",
    "Slot",
    "row",
]


# ============================================================
# The canvas
# ============================================================

# ponytail: module-level so a view function can lay out without being handed a
# canvas. `render()` reads the same values, so there is one source of truth.
_CANVAS = [1280.0, 720.0]


def canvas(w: float, h: float) -> None:
    """Set the size of the video. Defaults to 1280x720."""
    _CANVAS[:] = [float(w), float(h)]


def width() -> float:
    """The canvas width. ``cm.width() / 2`` is the horizontal centre."""
    return _CANVAS[0]


def height() -> float:
    """The canvas height."""
    return _CANVAS[1]


# ============================================================
# Items — identity that survives a snapshot
# ============================================================

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


# ============================================================
# Layout — where things sit
# ============================================================


@dataclass(frozen=True)
class Slot:
    """A place to put something: a centre point, a size, and the edge it is
    naturally anchored by.

    A Slot is not a shape and nothing draws it.
    """

    x: float
    y: float
    w: float
    h: float
    anchor: str = "center"

    @property
    def left(self) -> float:
        return self.x - self.w / 2

    @property
    def right(self) -> float:
        return self.x + self.w / 2

    @property
    def top(self) -> float:
        return self.y - self.h / 2

    @property
    def bottom(self) -> float:
        return self.y + self.h / 2

    def point(self, anchor: "str | None" = None) -> "tuple[float, float]":
        """The single point this Slot anchors things at."""
        anchor = anchor or self.anchor
        if anchor == "center":
            return (self.x, self.y)
        if anchor == "bottom":
            return (self.x, self.bottom)
        if anchor == "top":
            return (self.x, self.top)
        raise ValueError(f"unknown anchor {anchor!r} — use center, top or bottom")


def row(
    items,
    *,
    gap: float = 40.0,
    w: "float | None" = None,
    h: "float | None" = None,
    bottom: "float | None" = None,
    y: "float | None" = None,
):
    """One Slot per item, evenly spaced and centred on the canvas.

        for slot, item in cm.row(values, gap=40):
            ...

    A row lays things out on a shared baseline, so its Slots anchor at
    bottom-centre — hand one straight to ``scene.group()``.

    Yields ``(slot, item)`` pairs, or bare Slots if you passed a count.
    """
    sequence = list(range(items)) if isinstance(items, int) else list(items)
    count = len(sequence)
    if count == 0:
        return

    # Fill 70% of the canvas by default, so a row always looks deliberate.
    if w is None:
        usable = width() * 0.7 - gap * (count - 1)
        w = max(usable / count, 1.0)
    if h is None:
        h = w
    if y is None:
        y = (height() * 0.78 if bottom is None else bottom) - h / 2

    span = count * w + (count - 1) * gap
    first = (width() - span) / 2 + w / 2

    for index, item in enumerate(sequence):
        slot = Slot(x=first + index * (w + gap), y=y, w=w, h=h, anchor="bottom")
        yield slot if isinstance(items, int) else (slot, item)


_UNSET = object()


def _resolve(centre, low, high, half, names, default=_UNSET):
    """Turn whichever anchor was given into a centre coordinate."""
    given = [v for v in (centre, low, high) if v is not None]
    if len(given) > 1:
        raise ValueError(f"give only one of {', '.join(f'{n}=' for n in names)}")
    if not given:
        if default is _UNSET:
            raise ValueError(f"give one of {', '.join(f'{n}=' for n in names)}")
        return float(default)
    if centre is not None:
        return float(centre)
    if low is not None:
        return float(low) + half
    return float(high) - half


# ============================================================
# Scene — what one moment looks like
# ============================================================


@dataclass(frozen=True)
class _Shape:
    """One shape in the payload. Flat by design (ADR 0008) — the Engine diffs
    it field by field, so every kind carries every field."""

    item: str
    kind: str
    x: float
    y: float
    w: float = 0.0
    h: float = 0.0
    r: float = 0.0
    color: str = "white"
    text: str = ""
    size: float = 16.0
    layer: int = 0
    opacity: float = 1.0


def _key(path) -> str:
    """Join a path of keys into the string the Engine pairs shapes by."""
    return "/".join(str(p) for p in path)


class Group:
    """Somewhere to draw, with its own origin and its own name.

    Everything drawn on a Group moves as one thing, because the Engine sees
    each child's name as part of the group's name.

    **Inside a Group, ``0`` is the Group's anchor point.** ``bottom=0`` stands
    a shape on it; ``top=20`` puts one 20 below it.
    """

    def __init__(self, scene: "Scene", path: tuple, ox: float, oy: float, w: float):
        self._scene = scene
        self._path = path
        self._ox = ox
        self._oy = oy
        self._w = w

    # -- the origin a child measures from ------------------------------------

    @property
    def _child_x_default(self):
        return 0.0 if self._path else _UNSET

    @property
    def _child_w_default(self):
        return self._w if self._path else _UNSET

    def _place(self, key, kind, *, x, y, w=0.0, h=0.0, r=0.0, **rest) -> "Group":
        path = self._path + (key,)
        name = _key(path)
        if name in self._scene._shapes:
            raise ValueError(
                f"two shapes share the name {name!r} — a name means exactly one thing"
            )
        self._scene._shapes[name] = _Shape(
            item=name, kind=kind, x=self._ox + x, y=self._oy + y, w=w, h=h, r=r, **rest
        )
        return self

    # -- shapes ---------------------------------------------------------------

    def rect(
        self,
        key: Hashable,
        *,
        h: float,
        w: float = None,
        x: float = None,
        y: float = None,
        left: float = None,
        right: float = None,
        top: float = None,
        bottom: float = None,
        color: str = "white",
        layer: int = 0,
        opacity: float = 1.0,
    ) -> "Group":
        """A rectangle. Place it by its centre or by any edge."""
        if w is None:
            w = _resolve(None, None, None, 0, ("w",), self._child_w_default)
        return self._place(
            key,
            "rect",
            x=_resolve(x, left, right, w / 2, ("x", "left", "right"), self._child_x_default),
            y=_resolve(y, top, bottom, h / 2, ("y", "top", "bottom")),
            w=w,
            h=h,
            color=color,
            layer=layer,
            opacity=opacity,
        )

    def circle(
        self,
        key: Hashable,
        *,
        r: float,
        x: float = None,
        y: float = None,
        left: float = None,
        right: float = None,
        top: float = None,
        bottom: float = None,
        color: str = "white",
        layer: int = 0,
        opacity: float = 1.0,
    ) -> "Group":
        """A circle. Place it by its centre or by any edge, like a rect."""
        return self._place(
            key,
            "circle",
            x=_resolve(x, left, right, r, ("x", "left", "right"), self._child_x_default),
            y=_resolve(y, top, bottom, r, ("y", "top", "bottom")),
            r=r,
            color=color,
            layer=layer,
            opacity=opacity,
        )

    def text(
        self,
        key: Hashable,
        content: Any,
        *,
        x: float = None,
        y: float = None,
        top: float = None,
        bottom: float = None,
        size: float = 16.0,
        color: str = "white",
        layer: int = 10,
        opacity: float = 1.0,
    ) -> "Group":
        """Text, centred horizontally. You never deal with baselines."""
        return self._place(
            key,
            "text",
            x=_resolve(x, None, None, 0.0, ("x",), self._child_x_default),
            y=_resolve(y, top, bottom, size / 2, ("y", "top", "bottom")),
            text=str(content),
            size=size,
            color=color,
            layer=layer,
            opacity=opacity,
        )

    # -- nesting --------------------------------------------------------------

    def group(
        self,
        key: Hashable,
        slot: "Slot | None" = None,
        *,
        at: "str | None" = None,
        x: float = None,
        y: float = None,
        left: float = None,
        right: float = None,
        top: float = None,
        bottom: float = None,
        w: float = None,
    ) -> "Group":
        """A place to draw a thing made of several shapes.

            bar = scene.group(item.id, slot)
            bar.rect("bar", h=item.value * 70, bottom=0)
            bar.text("label", item.value, top=20)

        Give it a Slot and it sits where the Slot says. Inside it, ``0`` is
        that point, and everything drawn on it moves as one.
        """
        if slot is not None:
            gx, gy = slot.point(at)
            gw = slot.w if w is None else w
        else:
            # A group with no Slot and no anchors sits exactly where its
            # parent does — a group that exists only to name things.
            gw = self._w if w is None else w
            gx = _resolve(x, left, right, gw / 2, ("x", "left", "right"), self._child_x_default)
            gy = _resolve(y, top, bottom, 0.0, ("y", "top", "bottom"), self._child_x_default)

        return Group(self._scene, self._path + (key,), self._ox + gx, self._oy + gy, gw)


class Scene(Group):
    """The picture at one moment. No animation, no timing, no memory of the
    frame before.

    Every shape carries a name — the identity of the thing you are talking
    about. **Motion is implied by identity:** the Engine pairs shapes by name
    between one moment and the next, and whatever changed becomes movement. A
    name that follows a *value* travels; a name that follows a *position*
    stays put and changes shape. Both are useful, and choosing wrong produces a
    confusing video, not an error.
    """

    def __init__(self) -> None:
        self._shapes: dict[str, _Shape] = {}
        super().__init__(self, (), 0.0, 0.0, width())

    def _payload(self) -> "list[dict[str, Any]]":
        return [asdict(s) for s in self._shapes.values()]


# ============================================================
# Trace — what happened
# ============================================================


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


_recorder: ContextVar["_Recorder | None"] = ContextVar("codimate_recorder", default=None)


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


# ============================================================
# View, motion, timing
# ============================================================


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

    def named(self, *keys: str) -> set:
        """The values this event named, e.g. ``frame.named("a", "b")``.

        Empty for the opening moment, so it is safe to call unconditionally.
        """
        if self.event is None:
            return set()
        return {self.event.data[k] for k in keys if k in self.event.data}

    def items(self, key: str = "items") -> list:
        """The things this event named, e.g. ``frame.items()``.

        Use it when you emitted a list: ``cm.emit("compare", items=[a, b])``.
        Empty for the opening moment.
        """
        if self.event is None:
            return []
        return list(self.event.data.get(key, ()))


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


# ============================================================
# Explanation
# ============================================================


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
        """Draw every frame and write the video. Size comes from `cm.canvas()`."""
        from . import _codimate  # imported here so the pure Python is testable

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
