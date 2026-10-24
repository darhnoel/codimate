"""Axes: a graph's frame, its ticks, and the map from data to pixels (ADR 0016).

Made, mapped and drawn by the Authoring Core (`axes` and its methods in
api/surface.toml); `line` asks your function for each value here, because only
Python can run it.
"""

from .._surface import Axes, axes

__all__ = ["Axes", "axes"]
