"""Codimate — turn a running algorithm into an explainer video.

You write four things, and never a keyframe:

    algorithm   your normal code, with emit() where something happens
    view        what one moment looks like
    motion      how things travel between moments
    timing      how long each moment lasts

Codimate pairs shapes between moments **by name** and turns the differences
into movement. Everything per-frame — diffing, interpolation, drawing,
encoding — happens in Rust (ADR 0008).

## What your algorithm did

- `trace` — mark a function so Codimate can watch it run
- `emit` — say that something worth showing just happened
- `items` — a list whose entries keep their identity when they move
- `Item`, `Event`, `Trace`, `Frame` — what your view is handed

## What a moment looks like

- `Scene` — one picture: `rect`, `circle`, `polygon`, `curve`, `arrow`,
  `text`, `line`, `formula`, `svg`
- `ngon`, `star` — corners for a polygon, so you do not compute them
- `Group` — several shapes that move together
- `Handle` — what a shape call returns: `.fill()`, `.round()`, `.turn()`,
  `.grow()`, `.on()`, `.write()`, chained

## Where things sit

- `canvas`, `width`, `height` — the frame
- `row`, `column`, `Slot` — divide it up, without coordinates
- `measure`, `measure_math` — how big text or a formula will actually be

## How it moves, and for how long

- `Rule` — the path a shape travels
- `Timing` — how long each event lasts
- `ease` — the curve the Engine uses, if you need to draw it

## Running it

- `explain` — gather algorithm, view, motion and timing
- `Explanation.render` — write the video
"""

from __future__ import annotations

from .explain import Explanation, Rule, Timing, ease, explain
from .layout import (Place, Slot, at, canvas, column, height, measure, measure_math,
                     ngon, row, star, width)
from .scene import Group, Handle, Scene
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
    "Handle",
    # where things sit
    "canvas",
    "measure",
    "measure_math",
    "width",
    "height",
    "Slot",
    "Place",
    "at",
    "row",
    "column",
    "ngon",
    "star",
    # putting it together
    "Rule",
    "Timing",
    "ease",
    "explain",
    "Explanation",
]
