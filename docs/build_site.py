#!/usr/bin/env python3
"""Build the documentation site into `site/`.

    python docs/build_site.py            # then open site/index.html

The guides are Markdown in this folder; the API reference is generated from
the docstrings by pdoc. This puts both behind one nav so a reader can go from
"what is this" to "what does `focus` take" without leaving the page.

The ADRs are deliberately not here. They record why the library is shaped the
way it is, which is a contributor's question, not a user's — they stay in the
repository, and anything that links to one links to GitHub.

Nothing here needs the Rust extension: every `_codimate` import in the package
is inside the function that uses it, so `import codimate` works on a bare
checkout and the docs build is Python-only.

    pip install markdown pygments pdoc
"""

from __future__ import annotations

import re
import shutil
import subprocess
import sys
from pathlib import Path

import markdown
from pygments.formatters import HtmlFormatter

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
OUT = ROOT / "site"

# Order is the reading order, and it is also the nav. A page not listed here
# is still built if something links to it — it just does not get a nav entry.
NAV = [
    ("index.html", "Home"),
    ("tutorial.html", "Tutorial"),
    ("concepts.html", "Concepts"),
    ("drawing.html", "Drawing"),
    ("reference.html", "Reference"),
    ("api/codimate.html", "API"),
]

PAGES = {
    ROOT / "README.md": "index.html",
    DOCS / "tutorial.md": "tutorial.html",
    DOCS / "concepts.md": "concepts.html",
    DOCS / "drawing.md": "drawing.html",
    DOCS / "reference.md": "reference.html",
}

# Pygments paints the code blocks. Both themes are emitted and the dark one is
# scoped to the media query, so a page carries its own highlighting rather than
# fetching a stylesheet the CDN might not have.
HIGHLIGHT = (
    HtmlFormatter(style="default").get_style_defs(".codehilite")
    + "\n@media (prefers-color-scheme: dark) {\n"
    + HtmlFormatter(style="github-dark").get_style_defs(".codehilite")
    + "\n}"
)

SHELL = """<!doctype html>
<html lang="en"><head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} — Codimate</title>
<style>
:root {{ color-scheme: light dark; --ink:#1b2332; --dim:#5b6478; --bg:#fff;
         --panel:#f5f6f8; --line:#e3e6ec; --link:#1f6feb; }}
@media (prefers-color-scheme: dark) {{
  :root {{ --ink:#e8eef7; --dim:#9aa4b8; --bg:#12161f; --panel:#1a202b;
           --line:#2a3140; --link:#6ea8ff; }}
}}
* {{ box-sizing: border-box; }}
body {{ margin:0; background:var(--bg); color:var(--ink); line-height:1.65;
        font:16px/1.65 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }}
header {{ border-bottom:1px solid var(--line); position:sticky; top:0;
          background:var(--bg); z-index:2; }}
nav {{ max-width:min(860px, 92vw); margin:0 auto; display:flex; flex-wrap:wrap;
       gap:1.25rem; padding:.9rem 0; font-size:.93rem; }}
nav a {{ color:var(--dim); text-decoration:none; }}
nav a.here, nav a:hover {{ color:var(--ink); }}
nav .brand {{ font-weight:700; color:var(--ink); margin-right:auto; }}
main {{ max-width:min(860px, 92vw); margin:0 auto; padding:2.5rem 0 6rem; }}
a {{ color:var(--link); }}
h1, h2, h3 {{ line-height:1.25; margin:2.2rem 0 .8rem; }}
h1 {{ font-size:2rem; margin-top:0; }}
h2 {{ font-size:1.4rem; padding-top:.6rem; border-top:1px solid var(--line); }}
h3 {{ font-size:1.1rem; }}
code {{ background:var(--panel); padding:.12em .35em; border-radius:4px;
        font-size:.88em; }}
pre {{ background:var(--panel); border:1px solid var(--line); border-radius:8px;
       padding:1rem; overflow-x:auto; }}
pre code {{ background:none; padding:0; font-size:.85rem; line-height:1.55; }}
table {{ border-collapse:collapse; width:100%; display:block; overflow-x:auto; }}
th, td {{ border:1px solid var(--line); padding:.5rem .7rem; text-align:left; }}
th {{ background:var(--panel); }}
blockquote {{ border-left:3px solid var(--line); margin:1rem 0; padding:0 1rem;
              color:var(--dim); }}
img {{ max-width:100%; }}
{highlight}
</style>
</head><body>
<header><nav>{nav}</nav></header>
<main>{body}</main>
</body></html>
"""


def nav_html(here: str, depth: int) -> str:
    up = "../" * depth
    out = [f'<a class="brand" href="{up}index.html">Codimate</a>']
    for href, label in NAV:
        mark = ' class="here"' if href == here else ""
        out.append(f'<a href="{up}{href}"{mark}>{label}</a>')
    return "\n".join(out)


def to_html(md_path: Path, dest: str, depth: int = 0) -> str:
    text = md_path.read_text()

    # A Markdown link between two source files has to become a link between
    # two built pages. Rewriting at this one point is why every guide can keep
    # linking the way it does when read on GitHub.
    def relink(m):
        label, target = m.group(1), m.group(2)
        if target.startswith(("http", "#", "mailto:")):
            return m.group(0)
        anchor = ""
        if "#" in target:
            target, anchor = target.split("#", 1)
            anchor = "#" + anchor
        name = target.split("/")[-1]
        if name.endswith(".md"):
            page = PAGES.get((md_path.parent / target).resolve())
            if page:
                return f"[{label}]({'../' * depth}{page}{anchor})"
        # Anything else — a source file, an example folder — points at GitHub,
        # because the site does not carry the repository.
        rel = (md_path.parent / target).resolve().relative_to(ROOT)
        return f"[{label}](https://github.com/darhnoel/codimate/blob/main/{rel}{anchor})"

    text = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", relink, text)

    body = markdown.markdown(
        text,
        extensions=["fenced_code", "codehilite", "tables", "toc", "sane_lists"],
        extension_configs={"codehilite": {"guess_lang": False}},
    )
    title = re.search(r"^#\s+(.+)$", text, re.M)
    return SHELL.format(title=title.group(1) if title else "Codimate",
                        nav=nav_html(dest, depth), body=body,
                        highlight=HIGHLIGHT)


def main() -> int:
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir()

    for source, dest in PAGES.items():
        (OUT / dest).write_text(to_html(source, dest))
        print(f"  {dest}")

    # The API reference, straight from the docstrings.
    subprocess.run(
        [sys.executable, "-m", "pdoc", "-o", str(OUT / "api"),
         "-d", "google", "--no-include-undocumented", "--no-show-source",
         "codimate"],
        cwd=ROOT / "python", check=True,
        env={**__import__("os").environ, "PYTHONPATH": str(ROOT / "python")},
    )
    # pdoc renders its own shell, so the site nav is grafted on afterwards —
    # without it the API pages are a one-way door out of the site.
    #
    # It goes in a `<header>`, not a `<nav>`: pdoc styles bare `nav` as its
    # fixed full-height sidebar, so a `<nav>` here lands underneath that and is
    # never seen. `header` is the element pdoc's own layout leaves room for.
    for page in (OUT / "api").rglob("*.html"):
        depth = len(page.relative_to(OUT).parts) - 1
        bar = nav_html("api/codimate.html", depth).replace(
            'class="brand"', 'style="font-weight:700;margin-right:auto"')
        page.write_text(page.read_text().replace(
            "<body>",
            '<body><header style="display:flex;flex-wrap:wrap;gap:1.25rem;'
            'align-items:baseline;font-size:.9rem;padding-bottom:0">'
            + bar + "</header>", 1))
    print("  api/")

    # GitHub Pages runs Jekyll over an artifact unless told not to, and Jekyll
    # drops files beginning with an underscore — which pdoc's search index and
    # assets use.
    (OUT / ".nojekyll").touch()

    print(f"\nbuilt {OUT.relative_to(Path.cwd())} — open site/index.html")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
