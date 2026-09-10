"""Codimate — turn a running algorithm into an explainer video.

You write four things:

    algorithm   your normal code, with emit() where something happens
    view        what one moment looks like
    motion      how things travel between moments
    timing      how long each moment lasts

Everything per-frame — diffing, interpolation, drawing, encoding — happens in
Rust. See docs/adr/0008-python-authoring-surface.md.

    codimate.layout    the canvas, Slots, row, column
    codimate.scene     Groups, Scenes, the shapes you can draw
    codimate.trace     Items, emit, trace, Frame
    codimate.explain   Rule, Timing, render
"""

from __future__ import annotations

from .explain import Explanation, Rule, Timing, ease, explain
from .layout import Slot, canvas, column, height, row, width
from .scene import Group, Scene
from .trace import Event, Frame, Item, Trace, emit, items, trace

__all__ = [
    # what happened
    "emit",
    "trace",
    "items",
    "Item",
    "Event",
    "Trace",
    "Frame",
    # what it looks like
    "Scene",
    "Group",
    # where things sit
    "canvas",
    "width",
    "height",
    "Slot",
    "row",
    "column",
    # putting it together
    "Rule",
    "Timing",
    "ease",
    "explain",
    "Explanation",
]
