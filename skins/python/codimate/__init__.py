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

- `trace` — run your algorithm and record what it did; it hands the algorithm an
  `emit`, to say that something worth showing just happened
- `items` — a list whose entries keep their identity when they move
- `Item`, `Event`, `Trace`, `Frame` — what your view is handed

## What a moment looks like

- `Scene` — one picture: `rect`, `circle`, `arc`, `polygon`, `curve`, `arrow`,
  `text`, `line`, `formula`, `svg`, `image`, or a poseable `character`
- `ngon`, `star` — corners for a polygon, so you do not compute them
- `Group` — several shapes that move together
- `Handle` — what a shape call returns: `.fill()`, `.round()`, `.turn()`,
  `.grow()`, `.on()`, `.write()`, chained
- `Character` — a connected SVG figure with per-Scene joints and expressions
- `TextPreset`, `TextPresetStyle` — reusable word and letter entrances

## Where things sit

- `canvas`, `width`, `height` — the frame
- `font` — the font every text uses, by file or installed name
- `row`, `column`, `Slot` — divide it up, without coordinates
- `measure`, `measure_math` — how big text or a formula will actually be

## How it moves, and for how long

- `Rule` — the path a shape travels
- `Timing` — how long each event lasts
- `ease` — the curve the Engine uses, if you need to draw it

## Running it

- `explain` — gather algorithm, view, motion and timing
- `Explanation.render` — write the video

## Explaining physics

- `science` — labels, glows, brackets, force arrows; a camera, spheres, orbits
"""

from __future__ import annotations

from . import characters, science, text
from .explain import Explanation, Rule, Timing, ease, explain
from .layout import (
    Axes,
    Place,
    Slot,
    at,
    axes,
    canvas,
    column,
    font,
    height,
    measure,
    measure_math,
    ngon,
    row,
    star,
    width,
)
from .scene import Character, CharacterPart, Group, Handle, Scene
from .text import TextPreset, TextPresetStyle
from .trace import Event, Frame, Item, Trace, items, trace

__all__ = [
    # what happened
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
    "Character",
    "CharacterPart",
    "characters",
    "TextPreset",
    "TextPresetStyle",
    "text",
    # where things sit
    "canvas",
    "font",
    "measure",
    "measure_math",
    "width",
    "height",
    "Slot",
    "Place",
    "at",
    "row",
    "column",
    "axes",
    "Axes",
    "ngon",
    "star",
    # putting it together
    "Rule",
    "Timing",
    "ease",
    "explain",
    "Explanation",
    # optional kits
    "science",
]
