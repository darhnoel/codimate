"""Finding the index beside a video, and turning anything into an Index."""

from __future__ import annotations

import json
from pathlib import Path
from .base import DictIndex, Index
from .compact import CompactIndex


def beside(video) -> Index:
    """The index written beside `video`: the compact one if there is one, else the
    readable JSON."""
    video = Path(video)
    compact = video.with_name(f"{video.stem}.index.bin")
    if compact.exists():
        return CompactIndex(compact)
    readable = video.with_name(f"{video.stem}.index.json")
    if readable.exists():
        return DictIndex(json.loads(readable.read_text()))
    raise FileNotFoundError(
        f"no index beside {video} — render with index=True, or preview the script "
        f"itself")


def as_index(index) -> Index:
    """`index` as an `Index`: one already is, and a dict is wrapped."""
    return index if isinstance(index, Index) else DictIndex(index)
