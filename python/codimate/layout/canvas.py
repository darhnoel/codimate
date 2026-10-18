"""The canvas's size, and how big text and formulas measure on it."""

from __future__ import annotations



# ponytail: module-level so a view function can lay out without being handed a
# canvas. `render()` reads the same values, so there is one source of truth.
_CANVAS = [1280.0, 720.0]


def measure(text: str, size: float = 16.0) -> tuple[float, float]:
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
    """
    from .. import _codimate  # imported here so the pure Python is testable

    return _codimate.measure(str(text), float(size))


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
