"""Small, reusable scene backgrounds built from Codimate primitives."""

from __future__ import annotations

from math import ceil, floor
from typing import TYPE_CHECKING

from ..layout import height, width

if TYPE_CHECKING:
    from ..scene import Scene


def dot_grid(
    scene: Scene,
    *,
    spacing: float = 64.0,
    color: str = "#718096",
    background: str | None = None,
    offset: tuple[float, float] = (0.0, 0.0),
) -> None:
    """Draw a quiet, evenly spaced dot-grid across the canvas.

    Call it before drawing scene content. Dots sit below the default layer and
    have stable names between Scenes, so they remain still while foreground
    objects animate. Change ``offset`` between Scenes to move the grid. Set
    ``background`` to also paint a solid canvas color.

    ```python
    cm.backgrounds.dot_grid(scene, spacing=64, color="#8492a6",
                            offset=(frame.state.drift, 0))
    ```
    """
    if spacing <= 0:
        raise ValueError("spacing must be greater than zero")
    try:
        offset_x, offset_y = (float(value) for value in offset)
    except (TypeError, ValueError):
        raise ValueError("offset must be a pair of numbers") from None

    canvas_width, canvas_height = width(), height()
    if background is not None:
        scene.rect(
            "dot-grid/background",
            w=canvas_width,
            h=canvas_height,
            at=(canvas_width / 2, canvas_height / 2),
        ).fill(background).on(layer=-11)

    columns = max(1, ceil(canvas_width / spacing))
    rows = max(1, ceil(canvas_height / spacing))
    left = (canvas_width - (columns - 1) * spacing) / 2
    top = (canvas_height - (rows - 1) * spacing) / 2
    first_column = floor((-left - offset_x) / spacing)
    last_column = ceil((canvas_width - left - offset_x) / spacing)
    first_row = floor((-top - offset_y) / spacing)
    last_row = ceil((canvas_height - top - offset_y) / spacing)

    for row in range(first_row, last_row + 1):
        for column in range(first_column, last_column + 1):
            scene.circle(
                f"dot-grid/{row}/{column}",
                r=1.5,
                at=(left + column * spacing + offset_x, top + row * spacing + offset_y),
            ).fill(color).on(layer=-10, opacity=0.28)


__all__ = ["dot_grid"]
