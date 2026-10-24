"""`python -m codimate.preview`: open the Previewer, or print an index as JSON."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from ..index import LiveIndex
from .lookup import build, load
from .server import serve


def dump(path, args=(), beats: "str | None" = None) -> int:
    """Print an index as the readable JSON `index="json"` would write: of an mp4,
    from the file beside it, and of a script, built in memory. How a compact index
    is read by eye."""
    path = Path(path)
    if path.suffix == ".mp4":
        index = load(path)
    else:
        index = LiveIndex(build(path, args))
    start, _, stop = (beats or "").partition(":")
    count = len(index.beats)
    first = int(start) if start else 0
    last = int(stop) if stop else count
    sys.stdout.write(json.dumps(
        {"version": 1, "canvas": index.canvas,
         "beats": list(index.changes(first, last))},
        ensure_ascii=False, indent=2) + "\n")
    return 0


def main(argv=None) -> int:
    import argparse

    parser = argparse.ArgumentParser(
        prog="python -m codimate.preview",
        description="Look at a film — a script, or a rendered mp4 — and ask "
                    "what is on screen.")
    parser.add_argument("path", help="a script that calls render(), or an mp4 "
                                     "rendered with its index beside it")
    parser.add_argument("args", nargs=argparse.REMAINDER,
                        help="passed to the script, as `python script args` would")
    parser.add_argument("--port", type=int, default=0)
    parser.add_argument("--no-open", action="store_true",
                        help="print the address instead of opening a browser")
    parser.add_argument("--json", action="store_true",
                        help="print the index as readable JSON and exit: of an mp4, "
                             "from the file beside it; of a script, built in memory. "
                             "Put it before the path, which takes the rest as the "
                             "script's own arguments")
    parser.add_argument("--beats", metavar="FROM:TO",
                        help="with --json, only these beats (e.g. 100:120)")
    options = parser.parse_args(argv)
    if options.json:
        return dump(options.path, options.args, options.beats)
    serve(options.path, options.args, port=options.port,
          show=not options.no_open)
    return 0
