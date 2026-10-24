"""Small, reusable text entrance recipes for a View.

Each preset draws a stable set of named Text shapes. Change ``state`` between
Trace Events and pass the preset's ``rules()`` to ``explain`` to animate them.
Made, drawn and timed by the Authoring Core (`text_preset` and its shortcuts in
api/surface.toml), so a preset looks the same from every language.
"""

from .._surface import (  # noqa: F401
    TextPreset,
    TextPresetStyle,
    letters_down,
    letters_up,
    words_down,
    words_left,
    words_right,
    words_up,
    words_up_blur,
)
