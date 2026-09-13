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

SITE = Path(__file__).resolve().parent.parent / "site"


def main() -> int:
    if not SITE.exists():
        raise SystemExit("no site/ — run docs/build_site.py first")

    broken, checked = [], 0
    for page in sorted(SITE.rglob("*.html")):
        for match in re.finditer(r'(?:href|src)="([^"]+)"', page.read_text()):
            target = match.group(1)
            # `${...}` is pdoc's own JavaScript building a URL at runtime.
            if target.startswith(("http", "//", "#", "mailto:", "data:", "${")):
                continue
            checked += 1
            if not (page.parent / target.split("#")[0]).resolve().exists():
                broken.append(f"{page.relative_to(SITE)} -> {target}")

    print(f"checked {checked} internal links across "
          f"{len(list(SITE.rglob('*.html')))} pages")
    for link in broken:
        print(f"  broken: {link}")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main())
