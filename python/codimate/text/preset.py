"""Small, reusable text entrance recipes for a View.

Each preset draws a stable set of named Text shapes. Change ``state`` between
Trace Events and pass the preset's ``rules()`` to ``explain`` to animate them.
"""

from __future__ import annotations

import unicodedata
from dataclasses import dataclass
from typing import TYPE_CHECKING

from ..layout import measure

if TYPE_CHECKING:
    from ..scene import Group


_PRESETS = {
    "words_up": ("word", "up", False),
    "words_down": ("word", "down", False),
    "letters_up": ("letter", "up", False),
    "letters_down": ("letter", "down", False),
    "words_left": ("word", "left", False),
    "words_right": ("word", "right", False),
    "words_up_blur": ("word", "up", True),
}
_BLUR_OFFSETS = (
    (-2.0, -2.0),
    (-2.0, 0.0),
    (-2.0, 2.0),
    (0.0, -2.0),
    (0.0, 2.0),
    (2.0, -2.0),
    (2.0, 0.0),
    (2.0, 2.0),
)


@dataclass(frozen=True)
class TextPresetStyle:
    """Text appearance and travel distance shared by a preset's units.

    ``size`` is in canvas points. ``font`` and ``fill`` are optional, and
    ``distance`` is how far each word or letter starts from its final place.
    """

    size: float = 16.0
    font: str | None = None
    fill: str | None = None
    distance: float = 48.0


def _clusters(text: str) -> list[str]:
    """Keep combining marks and joined emoji with the character before them."""
    result: list[str] = []
    join_next = False
    regional_count = 0
    for char in text:
        code = ord(char)
        regional = 0x1F1E6 <= code <= 0x1F1FF
        attaches = bool(result) and (
            char == "\u200d"
            or unicodedata.category(char).startswith("M")
            or 0xFE00 <= code <= 0xFE0F
            or 0x1F3FB <= code <= 0x1F3FF
            or join_next
            or (regional and regional_count % 2 == 1)
        )
        if attaches:
            result[-1] += char
        else:
            result.append(char)
        join_next = char in ("\u200d", "\u17d2")
        regional_count = regional_count + 1 if regional else 0
    return result


class TextPreset:
    """Draw text as individually moving words or letters.

    Use a named helper such as ``codimate.text.words_up`` for a fixed motion.
    This class is the configurable form when the preset name is data::

        title = cm.TextPreset(
            "title", "Hello world", preset, style=cm.TextPresetStyle(size=72)
        )

        def view(frame):
            scene = cm.Scene()
            title.draw(scene, state="shown" if frame.state.ready else "hidden")
            return scene

        cm.explain(..., motion=title.rules(stagger=0.12))

    ``preset`` is one of ``words_up``, ``words_down``, ``letters_up``,
    ``letters_down``, ``words_left``, ``words_right``, or ``words_up_blur``.

    ``state`` is ``"hidden"`` or ``"shown"``. For ``words_up_blur``, it may
    also be ``"blurred"``; switch from that state to ``"shown"`` to resolve the
    soft halo into sharp text. The halo uses small translucent text copies, so
    it needs no image or optional imaging dependency.

    ``at`` is the text block's centre (a point or Slot); line breaks are
    centred independently. ``letters_*`` keeps combining marks and joined
    emoji together as one moving unit.
    """

    def __init__(
        self,
        name: str,
        content: str,
        preset: str,
        *,
        style: TextPresetStyle | None = None,
        at=None,
    ) -> None:
        if not name or any(char in name for char in "/*?"):
            raise ValueError("name must be non-empty and cannot contain /, * or ?")
        if preset not in _PRESETS:
            choices = ", ".join(_PRESETS)
            raise ValueError(f"unknown text preset {preset!r}; choose from {choices}")
        style = style or TextPresetStyle()
        if style.size <= 0 or style.distance < 0:
            raise ValueError("size must be positive and distance cannot be negative")
        self.name = name
        self.content = str(content)
        self.preset = preset
        self.style = style
        self.at = at

    def draw(self, group: "Group", *, state: str = "hidden") -> tuple:
        """Draw this preset into a Scene or Group at one of its stable states."""
        unit, direction, blurred = _PRESETS[self.preset]
        states = ("hidden", "blurred", "shown") if blurred else ("hidden", "shown")
        if state not in states:
            allowed = ", ".join(repr(value) for value in states)
            raise ValueError(f"{self.preset} state must be one of {allowed}")

        size = self.style.size
        x, y = group._where(self.at, size / 2)
        units = self._units(unit, x, y)
        distance = self.style.distance
        dx, dy = {
            "up": (0.0, distance),
            "down": (0.0, -distance),
            "left": (distance, 0.0),
            "right": (-distance, 0.0),
        }[direction]
        hidden = state == "hidden"
        shown = state == "shown"
        handles = []

        for index, text, ux, uy in units:
            px = ux + (dx if hidden else 0.0)
            py = uy + (dy if hidden else 0.0)
            if blurred:
                handles.extend(self._blurred(group, (index, text, px, py), state))
            handle = group.text(
                (self.name, "unit", index),
                text,
                size=size,
                font=self.style.font,
                at=(px, py),
            )
            if self.style.fill is not None:
                handle.fill(self.style.fill)
            handle.on(opacity=1.0 if shown else 0.0)
            handles.append(handle)
        return tuple(handles)

    def rules(
        self, *, ease: str = "out", stagger: float = 0.12, order: str = "screen"
    ) -> tuple:
        """Return motion rules for this preset's words, letters, and blur."""
        from ..explain import Rule

        rules = [Rule(f"{self.name}/unit/*", ease=ease, stagger=stagger, order=order)]
        if self.preset == "words_up_blur":
            rules.extend(
                Rule(
                    f"{self.name}/blur/{index}/*",
                    ease=ease,
                    stagger=stagger,
                    order=order,
                )
                for index in range(len(_BLUR_OFFSETS))
            )
        return tuple(rules)

    def _units(self, kind: str, center_x: float, center_y: float) -> list[tuple]:
        units = []
        lines = self.content.splitlines() or [self.content]
        size, font = self.style.size, self.style.font
        line_height = size * 1.2
        first_y = center_y - (len(lines) - 1) * line_height / 2
        index = 0
        for line_number, line in enumerate(lines):
            line_y = first_y + line_number * line_height
            if kind == "word":
                words = line.split()
                gap = measure(" ", size, font)[0]
                widths = [measure(word, size, font)[0] for word in words]
                left = center_x - (sum(widths) + gap * max(len(words) - 1, 0)) / 2
                for word, width in zip(words, widths):
                    units.append((index, word, left + width / 2, line_y))
                    index += 1
                    left += width + gap
                continue

            glyphs = _clusters(line)
            full_width = measure(line, size, font)[0]
            left = center_x - full_width / 2
            prefix = ""
            advance = 0.0
            for glyph in glyphs:
                next_advance = measure(prefix + glyph, size, font)[0]
                width = next_advance - advance
                if not glyph.isspace():
                    units.append((index, glyph, left + advance + width / 2, line_y))
                    index += 1
                prefix += glyph
                advance = next_advance
        return units

    def _blurred(self, group, unit, state):
        index, text, x, y = unit
        handles = []
        opacity = 0.12 if state == "blurred" else 0.0
        for halo, (dx, dy) in enumerate(_BLUR_OFFSETS):
            handle = group.text(
                (self.name, "blur", halo, index),
                text,
                size=self.style.size,
                font=self.style.font,
                at=(x + dx, y + dy),
            )
            if self.style.fill is not None:
                handle.fill(self.style.fill)
            handle.on(opacity=opacity)
            handles.append(handle)
        return handles


def words_up(
    name: str,
    content: str,
    *,
    style: TextPresetStyle | None = None,
    at=None,
) -> TextPreset:
    """Create a preset that brings words up into place."""
    return TextPreset(name, content, "words_up", style=style, at=at)


def words_down(
    name: str,
    content: str,
    *,
    style: TextPresetStyle | None = None,
    at=None,
) -> TextPreset:
    """Create a preset that brings words down into place."""
    return TextPreset(name, content, "words_down", style=style, at=at)


def letters_up(
    name: str,
    content: str,
    *,
    style: TextPresetStyle | None = None,
    at=None,
) -> TextPreset:
    """Create a preset that brings letter clusters up into place."""
    return TextPreset(name, content, "letters_up", style=style, at=at)


def letters_down(
    name: str,
    content: str,
    *,
    style: TextPresetStyle | None = None,
    at=None,
) -> TextPreset:
    """Create a preset that brings letter clusters down into place."""
    return TextPreset(name, content, "letters_down", style=style, at=at)


def words_left(
    name: str,
    content: str,
    *,
    style: TextPresetStyle | None = None,
    at=None,
) -> TextPreset:
    """Create a preset that brings words in from the left."""
    return TextPreset(name, content, "words_left", style=style, at=at)


def words_right(
    name: str,
    content: str,
    *,
    style: TextPresetStyle | None = None,
    at=None,
) -> TextPreset:
    """Create a preset that brings words in from the right."""
    return TextPreset(name, content, "words_right", style=style, at=at)


def words_up_blur(
    name: str,
    content: str,
    *,
    style: TextPresetStyle | None = None,
    at=None,
) -> TextPreset:
    """Create a preset that brings soft, blurred words up into focus."""
    return TextPreset(name, content, "words_up_blur", style=style, at=at)
