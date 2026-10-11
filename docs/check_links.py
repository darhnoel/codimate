#!/usr/bin/env python3
"""Fail if a built page links to something that is not there.

    python docs/check_links.py

The guides link to each other by filename, and `build_site.py` rewrites those
links as it converts them. A rename or a moved page breaks the rewrite
silently — the build still succeeds and the site still deploys, with a dead
link on it. This is the check that says so instead.

Only links inside the site are followed; anything with a scheme is somebody
else's problem.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote

SITE = Path(__file__).resolve().parent.parent / "site"


def main() -> int:
    if not SITE.exists():
        raise SystemExit("no site/ — run docs/build_site.py first")

    pages = sorted(SITE.rglob("*.html"))
    anchors = {
        page.resolve(): set(re.findall(r'\bid="([^"]+)"', page.read_text()))
        for page in pages
    }
    broken, checked = [], 0
    for page in pages:
        for match in re.finditer(r'(?:href|src)="([^"]+)"', page.read_text()):
            target = match.group(1)
            # `${...}` is pdoc's own JavaScript building a URL at runtime.
            if target.startswith(("http", "//", "mailto:", "data:", "${")):
                continue
            checked += 1
            filename, _, anchor = target.partition("#")
            destination = ((page.parent / filename).resolve()
                           if filename else page.resolve())
            if not destination.exists():
                broken.append(f"{page.relative_to(SITE)} -> {target}")
            elif (anchor and destination in anchors
                  and unquote(anchor) not in anchors[destination]):
                broken.append(f"{page.relative_to(SITE)} -> {target}")

    print(f"checked {checked} internal links across "
          f"{len(pages)} pages (including section anchors)")
    for link in broken:
        print(f"  broken: {link}")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main())
