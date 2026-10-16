"""Painting a ball's shading as an image, for the ``"painted"`` style. Private.

A ball lit by a light that is fixed relative to the camera looks the same every
frame, so its shading is painted once, pixel by pixel, and the picture is
reused: placed, sized and stretched like any image. Each look (colour, light,
ambient) is painted at most once and kept on disk, so a film pays for a handful
of paintings, not one a frame.

Needs numpy and Pillow, which Codimate does not otherwise: they come with
``pip install "codimate[painted]"``.
"""

from __future__ import annotations

import hashlib
import os
from pathlib import Path

SIZE = 256  # pixels across: a ball drawn bigger than this is stretched, softly
SUPERSAMPLE = 3  # pixels averaged per pixel kept, for a clean edge
STEP = 0.02  # how finely a light's direction is told apart before repainting


def _cache() -> Path:
    base = os.environ.get("XDG_CACHE_HOME") or Path.home() / ".cache"
    folder = Path(base) / "codimate" / "balls"
    folder.mkdir(parents=True, exist_ok=True)
    return folder


def ball(color: str, shadow: str, light: tuple, ambient: float) -> str:
    """The path of a painted ball: ``color`` lit from ``light``, a direction in
    the picture's own terms (x right, y up, z toward the viewer), mixing toward
    ``shadow`` where it turns away. Painted the first time it is asked for."""
    light = tuple(round(v / STEP) * STEP for v in light)
    look = f"{color}|{shadow}|{light}|{ambient:.3f}|{SIZE}"
    path = _cache() / (hashlib.sha1(look.encode()).hexdigest()[:16] + ".png")
    if not path.exists():
        _paint(path, color, shadow, light, ambient)
    return str(path)


def _paint(path, color, shadow, light, ambient):
    try:
        import numpy as np
        from PIL import Image
    except ImportError as missing:
        raise ImportError(
            'style="painted" paints with numpy and Pillow: '
            'pip install "codimate[painted]"') from missing

    n = SIZE * SUPERSAMPLE
    rows, cols = np.mgrid[0:n, 0:n]
    x = (cols + 0.5) / n * 2 - 1
    y = 1 - (rows + 0.5) / n * 2
    inside = x * x + y * y <= 1.0
    z = np.sqrt(np.clip(1 - x * x - y * y, 0, 1))
    to_light = np.asarray(light, float)
    size = np.linalg.norm(to_light)
    to_light = to_light / size if size else np.array((0.0, 0.0, 1.0))
    facing = np.clip(x * to_light[0] + y * to_light[1] + z * to_light[2], 0, 1)
    # A soft sheen where the surface faces halfway between light and eye.
    half = to_light + np.array((0.0, 0.0, 1.0))
    half /= np.linalg.norm(half) or 1.0
    sheen = 0.22 * np.clip(x * half[0] + y * half[1] + z * half[2], 0, 1) ** 14
    rim = 0.88 + 0.12 * z  # a touch darker toward the outline
    lit = np.array([int(color[i:i + 2], 16) for i in (1, 3, 5)], float)
    dark = np.array([int(shadow[i:i + 2], 16) for i in (1, 3, 5)], float)
    rgb = (dark + (lit - dark) * (ambient + (1 - ambient) * facing)[..., None])
    rgb = rgb * rim[..., None]
    rgb = rgb + (255 - rgb) * sheen[..., None]
    rgba = np.zeros((n, n, 4))
    rgba[..., :3] = np.clip(rgb, 0, 255)
    rgba[..., 3] = inside * 255.0
    # Average each block: a smooth gradient, and an edge with no stair steps.
    rgba[..., :3] *= rgba[..., 3:4] / 255.0
    small = rgba.reshape(SIZE, SUPERSAMPLE, SIZE, SUPERSAMPLE, 4).mean(axis=(1, 3))
    alpha = small[..., 3:4] / 255.0
    small[..., :3] = np.where(alpha > 0, small[..., :3] / np.maximum(alpha, 1e-6), 0)
    image = Image.fromarray(np.clip(small, 0, 255).astype(np.uint8), "RGBA")
    temporary = path.with_suffix(f".{os.getpid()}.tmp")
    image.save(temporary, format="PNG")
    temporary.replace(path)  # whole or not at all, if two films paint at once
