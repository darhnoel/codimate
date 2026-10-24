"""Generated from api/surface.toml by tools/generate.py. Do not edit.

The Python Skin's side of the Interface Manifest: each method sends its
arguments across the Waist and keeps what comes back."""

from __future__ import annotations

from typing import TYPE_CHECKING

from ._waist import _at, _film_font, _key, _number, _string, _text

if TYPE_CHECKING:
    from .scene import Handle


class _GroupSurface:
    def rect(self, key, *, h, w=None, at=None) -> "Handle":
        """A rectangle. Say where with a point, a Slot, or `at(...)`. Round the corners with `.round(12)`."""
        return self._build("rect", {"group": self._frame(), "key": _key(key), "h": _number(h), "w": _number(w), "at": _at(at)})

    def circle(self, key, *, r, at=None) -> "Handle":
        """A filled circle. `fill("none", edge=...)` draws a ring."""
        return self._build("circle", {"group": self._frame(), "key": _key(key), "r": _number(r), "at": _at(at)})

    def text(self, key, content, *, size=16, font=None, at=None) -> "Handle":
        """Text, centred horizontally. `font` is a font file or an installed font's name, for this text only; unset, the film's font."""
        return self._build("text", {"group": self._frame(), "film_font": _film_font(), "key": _key(key), "content": _text(content), "size": _number(size), "font": _string(font), "at": _at(at)})


class _HandleSurface:
    def fill(self, color="white", *, edge=None, edge_w=None) -> "Handle":
        """Colour it. `edge` outlines it; `color="none"` leaves it unfilled. Saying nothing about the outline leaves it alone."""
        self._change("fill", {"shape": self._shape(), "color": _string(color), "edge": _string(edge), "edge_w": _number(edge_w)})
        return self
