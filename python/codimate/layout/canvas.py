"""The canvas's size, the film's font, and how big text and formulas measure."""

from __future__ import annotations



# ponytail: module-level so a view function can lay out without being handed a
# canvas. `render()` reads the same values, so there is one source of truth.
_CANVAS = [1280.0, 720.0]
# The film's font, the same way: "" is the built-in DejaVu Sans Mono.
_FONT = [""]


def font(name: str = None) -> None:
    """Set the font every text in the film uses, unless it names its own.

        cm.font("fonts/Inter-Regular.ttf")    # a .ttf or .otf file
        cm.font("Helvetica Neue")             # or an installed font, by name
        cm.font()                             # back to the built-in font

    A name with a ``/`` or a font file's extension is a file; anything else is
    looked up among the fonts installed on this machine. A file travels with
    your project and renders the same everywhere, so prefer one for a film
    you will share. Bold or italic is its own file or name: ``"Inter Bold"``.
    Characters the font lacks are drawn in the built-in fonts, so Khmer and
    Chinese still show. Call it before building scenes, like ``canvas``. A font
    that will not load is a ``ValueError`` here, not a surprise at render.
    """
    name = str(name or "")
    if name:
        from .. import _codimate

        _codimate.measure("", 1.0, name)  # loads it, or says why it cannot
    _FONT[0] = name


def measure(text: str, size: float = 16.0, font: str = None) -> tuple[float, float]:
    """How wide and tall ``text`` will be at ``size``: ``(w, h)``.

    For drawing a box around a label without guessing::

        w, h = cm.measure(label, size=30)
        scene.rect("box", w=w + 24, h=h + 12, at=(x, y)).round(6)
        scene.text("label", label, size=30, at=(x, y))

    Measured by the engine with the real fonts, including fallback, so it is
    right for Khmer and anything else that is not plain ASCII — which is why
    estimating ``len(text) * size * k`` is not good enough.

    The height is the line height, the same for "cat" and "Qgy", so a row of
    boxes lines up instead of jittering with whatever letters it holds.

    ``font`` is measured in, as for ``scene.text``; unset, the film's font.
    """
    from .. import _codimate  # imported here so the pure Python is testable

    return _codimate.measure(str(text), float(size), font or _FONT[0])


def measure_math(latex: str, size: float = 16.0) -> tuple[float, float]:
    """How wide and tall a LaTeX formula will be at ``size``: ``(w, h)``.

    The counterpart of :func:`measure`, so a formula can be laid out beside
    words — a caption that mixes prose and mathematics needs both.
    """
    from .. import _codimate

    return _codimate.measure_formula(str(latex), float(size))


def canvas(w: float, h: float) -> None:
    """Set the size of the video. Defaults to 1280x720."""
    _CANVAS[:] = [float(w), float(h)]


def width() -> float:
    """The canvas width. ``cm.width() / 2`` is the horizontal centre."""
    return _CANVAS[0]


def height() -> float:
    """The canvas height."""
    return _CANVAS[1]
