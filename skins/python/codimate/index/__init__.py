"""The index: what is on screen at every beat, and what put it there.

An index answers questions about a film without drawing it. It has three forms,
behind one small interface (`Index`), so the Previewer asks the same questions of
all of them:

- **`LiveIndex`** — a film held in memory (the Previewer running a script). It keeps
  only a small table with one entry per beat and answers "what is on screen at beat
  k?" from the scene it already has, about a millisecond for fourteen hundred shapes.
  Building the whole index first cost seconds and hundreds of megabytes.
- **`CompactIndex`** — a file, `<video>.index.bin`, written by `render`. See below.
- **`DictIndex`** — the readable `<video>.index.json`, and the dict
  `Explanation.index()` returns. Kept so nothing written before still breaks.

**The compact file.** JSON stores every number as decimal text and repeats every field
name for every shape; for a film where everything moves each beat that is hundreds of
megabytes. The compact file stores the same facts as columns of small integers:

- A shape's box is kept to 1/16 of a pixel, and written as the difference from where
  that shape was last. Motion is regular, so the differences repeat, and a compressor
  finds the repetition that decimal text hides.
- Names and kinds are written once per chunk and referred to by number.
- Beats are grouped in chunks of `CHUNK`. A chunk begins with everything on screen
  when it starts, so it can be read on its own: asking for beat k decodes one chunk, not
  the film. The small per-beat table is kept apart, in a footer, so opening the file
  reads that and nothing else.
- Chunks are written as they are produced, so the writer's memory does not grow with
  the film.

    file   := MAGIC  chunk*  footer  footer_length(u64, little endian)
    footer := lzma(JSON: version, canvas, chunk, quant, chunks[(offset, length)], beats)
    chunk  := lzma(u32 header_length, JSON header, then little-endian columns:
               int32, and int64 for boxes, which can be huge when a shape is zoomed)

On `year`, whose index is about 390 MB as JSON, this comes to a few megabytes. To read
one, `python -m codimate.preview --json film.mp4` prints it as JSON (ADR 0020).


The parts:

- `base` — The Index interface, and its in-memory forms: a dict, and a live film.
- `compact` — The compact index file: written in chunks, read a chunk at a
  time (ADR 0020).
- `locate` — Finding the index beside a video, and turning anything into an Index.
"""

from .base import (
    Index, DictIndex, LiveIndex,
)
from .compact import (
    MAGIC, QUANT, CHUNK, PRESET, CACHED, _quantised, _encode, write, CompactIndex,
)
from .locate import (
    beside, as_index,
)

__all__ = [
    'CACHED', 'CHUNK', 'CompactIndex', 'DictIndex', 'Index', 'LiveIndex', 'MAGIC',
    'PRESET', 'QUANT', '_encode', '_quantised', 'as_index', 'beside', 'write',
]
