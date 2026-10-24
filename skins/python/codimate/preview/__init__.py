"""The Previewer: a window onto a film, and a way to ask what is on it.

    python -m codimate.preview archimedes/main.py km    # from codimate-examples
    python -m codimate.preview results/archimedes-km.mp4

Given a **script**, it runs it up to its `render` call and keeps the film in
memory — no video is drawn. Each moment you scrub to is drawn on request, and
saving the script (or pressing Reload) runs it again. Given an **mp4**, it
plays the finished film, with its sound, against the index written beside it.

Either way: click what is on screen, and learn its name, the event that drew
it and the State behind it. Attach a note, copy the notes out.

**It never writes.** There is no save, no edit, no path back into your Python.
It produces words about the film; changing the film stays an edit to the one
file, made by you or by an agent acting on the note (ADR 0018).

The lookup lives here rather than in the page, so the question it answers does
not need a window at all:

    from codimate import preview
    index = preview.load("results/archimedes-km.mp4")
    print(preview.note(preview.at(index, 55.0, (640, 290))))


The parts:

- `lookup` — Asking a film what is on it: at a time, at a point, and why.
- `server` — The Previewer's window: a film held open, and the local page that shows it.
- `cli` — `python -m codimate.preview`: open the Previewer, or print an index as JSON.
"""

from .lookup import (
    load, build, index_of, on_screen, at, beat_at, covered, _under, note, _brief,
    _short,
)
from .server import (
    Film, serve, _handler,
)
from .cli import (
    dump, main,
)

__all__ = [
    'Film', '_brief', '_handler', '_short', '_under', 'at', 'beat_at', 'build',
    'covered', 'dump', 'index_of', 'load', 'main', 'note', 'on_screen', 'serve',
]
