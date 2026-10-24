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

import os
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
# This repository's own URLs, as the README writes them for PyPI.
OWN_URLS = (
    "https://raw.githubusercontent.com/darhnoel/codimate/main/",
    "https://github.com/darhnoel/codimate/blob/main/",
    "https://github.com/darhnoel/codimate/tree/main/",
)

# Order is the reading order, and it is also the nav. A page not listed here
# is still built if something links to it — it just does not get a nav entry.
NAV = [
    ("index.html", "Home"),
    ("concepts.html", "Concepts"),
    ("tutorial.html", "Tutorial"),
    ("reference.html", "Reference"),
    ("api/codimate.html", "API"),
]

# On a phone the nav shows these icons instead of its words (24-unit, stroked,
# like the theme toggle); the word stays as the link's name and tooltip.
ICONS = {
    "Home": '<path d="M3 10.5 12 3l9 7.5"/><path d="M5 9.5V21h5v-6h4v6h5V9.5"/>',
    "Concepts": '<path d="M9 18h6M10 21h4"/>'
                '<path d="M12 3a6 6 0 0 0-3.6 10.8c.6.5 1 1.2 1 2V16h5.2v-.2'
                'c0-.8.4-1.5 1-2A6 6 0 0 0 12 3Z"/>',
    "Tutorial": '<path d="M2 5.5C4 4.5 7 4.3 12 6c5-1.7 8-1.5 10-.5V19'
                'c-2-1-5-1.2-10 .5-5-1.7-8-1.5-10-.5Z"/><path d="M12 6v13.5"/>',
    "Reference": '<path d="M8 6h13M8 12h13M8 18h13"/>'
                 '<path d="M3.5 6h.01M3.5 12h.01M3.5 18h.01"/>',
    "API": '<path d="m8 7-5 5 5 5M16 7l5 5-5 5"/>',
}

# Words on wide screens, icons on narrow ones. Shared by the site and the
# pdoc-built API pages.
NAV_CSS = """
.nav-icon { display:none; width:1.3rem; height:1.3rem; fill:none; stroke:currentColor;
  stroke-width:1.7; stroke-linecap:round; stroke-linejoin:round; }
@media (max-width: 640px) {
  .header-links { width:100%; flex-wrap:nowrap; gap:0; padding:0 3rem 0 1rem; }
  .header-links .nav-label { display:none; }
  .header-links .nav-icon { display:block; }
  .header-links a:not(.brand) { display:inline-flex; align-items:center;
    justify-content:center; flex:none; width:2.1rem; height:2.5rem; }
  .site-header .theme-toggle { right:.4rem; }
}
"""

PAGES = {
    ROOT / "README.md": "index.html",
    DOCS / "concepts.md": "concepts.html",
    DOCS / "tutorial.md": "tutorial.html",
    DOCS / "reference.md": "reference.html",
    # Keep existing guide URLs working, without adding navigation choices.
    DOCS / "drawing.md": "drawing.html",
    DOCS / "recipes.md": "recipes.html",
    DOCS / "troubleshooting.md": "troubleshooting.html",
}

# Keep both syntax themes available so the toggle can override the OS theme.
HIGHLIGHT = (
    HtmlFormatter(style="default").get_style_defs(".codehilite")
    + "\n"
    + HtmlFormatter(style="github-dark").get_style_defs(
        'html[data-theme="dark"] .codehilite')
)

THEME_SCRIPT = """<script>
(() => {
  const root = document.documentElement;
  const key = 'codimate-theme';
  const button = document.querySelector('.theme-toggle');
  const update = () => {
    const dark = root.dataset.theme === 'dark';
    const label = dark ? 'Switch to light mode' : 'Switch to night mode';
    button.setAttribute('aria-label', label);
    button.title = label;
    button.setAttribute('aria-pressed', String(dark));
  };
  update();
  button.addEventListener('click', () => {
    root.dataset.theme = root.dataset.theme === 'dark' ? 'light' : 'dark';
    try { localStorage.setItem(key, root.dataset.theme); } catch {}
    update();
  });
})();
</script>"""

THEME_BOOT = """<script>
try {
  const saved = localStorage.getItem('codimate-theme');
  document.documentElement.dataset.theme = saved ||
    (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
} catch {
  document.documentElement.dataset.theme =
    matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}
</script>"""

PDOC_THEME = """<style>
html[data-theme="dark"] { --pdoc-background:#12161f; color-scheme:dark; }
html[data-theme="dark"] .pdoc {
  --text:#e8eef7; --muted:#9aa4b8; --link:#6ea8ff; --link-hover:#a6c8ff;
  --code:#1a202b; --active:#394052; --accent:#222a38; --accent2:#394355;
  --nav-hover:rgba(255,255,255,.12); --name:#8cc8ff; --def:#8ad49b;
  --annotation:#b1d6a7;
}
html[data-theme="dark"] .pdoc pre,
html[data-theme="dark"] .pdoc code { background-color:#1a202b; color:#d5deeb; }
.pdoc main { font-size:1.0625rem; line-height:1.7; }
.site-header { position:sticky; top:0; z-index:20; display:block; width:100%;
  max-width:none; min-height:4rem; padding:0; background:var(--pdoc-background);
  border-bottom:1px solid var(--accent2); }
.header-links a { color:#6c757d; }
.header-links a:hover, .header-links a.here { color:#212529; }
.header-links .brand { color:#212529; }
html[data-theme="dark"] .header-links a { color:#adb8ca; }
html[data-theme="dark"] .header-links a:hover,
html[data-theme="dark"] .header-links a.here,
html[data-theme="dark"] .header-links .brand { color:#e8eef7; }
.theme-toggle { display:inline-flex; align-items:center; justify-content:center;
  width:2.5rem; height:2.5rem; flex:none; border:0; border-radius:50%;
  padding:0; background:transparent; color:#6c757d; cursor:pointer; }
.theme-toggle:hover { background:#eee; color:#212529; }
html[data-theme="dark"] .theme-toggle { color:#adb8ca; }
html[data-theme="dark"] .theme-toggle:hover { background:#222a38; color:#e8eef7; }
.theme-toggle:focus-visible { outline:2px solid #6ea8ff; outline-offset:2px; }
.theme-toggle svg { width:1.25rem; height:1.25rem; fill:none; stroke:currentColor;
  stroke-width:1.7; stroke-linecap:round; stroke-linejoin:round; }
.theme-toggle .icon-sun { display:none; }
html[data-theme="dark"] .theme-toggle .icon-sun { display:block; }
html[data-theme="dark"] .theme-toggle .icon-moon { display:none; }
.header-links { width:min(860px,92vw); min-height:4rem; margin:0 auto;
  padding-right:3.5rem; display:flex; flex-wrap:wrap; align-items:center;
  gap:1.25rem; font-size:.9rem; }
.header-links a { color:var(--muted); text-decoration:none; }
.header-links a:hover, .header-links a.here { color:var(--text); }
.header-links .brand { color:var(--text); font-weight:700; margin-right:auto; }
.site-header .theme-toggle { position:absolute; top:50%; right:1.25rem;
  transform:translateY(-50%); }
@media (max-width:769px) {
  /* pdoc's own menu button sits at the top right, under the site header */
  #navtoggle { top:4.75rem; }
}
""" + NAV_CSS + """</style>"""

SHELL = """<!doctype html>
<html lang="en"><head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} — Codimate</title>
{theme_boot}
<style>
:root {{ color-scheme: light; --ink:#1b2332; --dim:#5b6478; --bg:#fff;
         --panel:#f5f6f8; --line:#e3e6ec; --link:#1f6feb; }}
:root[data-theme="dark"] {{ color-scheme:dark; --ink:#e8eef7; --dim:#9aa4b8;
  --bg:#12161f; --panel:#1a202b; --line:#2a3140; --link:#6ea8ff; }}
* {{ box-sizing: border-box; }}
body {{ margin:0; background:var(--bg); color:var(--ink); line-height:1.7;
        font:17px/1.7 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }}
header {{ border-bottom:1px solid var(--line); position:sticky; top:0;
          background:var(--bg); z-index:2; min-height:4rem; }}
.header-links {{ width:min(860px,92vw); min-height:4rem; margin:0 auto;
  padding-right:3.5rem; display:flex; flex-wrap:wrap; align-items:center;
  gap:1.25rem; font-size:.93rem; }}
.header-links a {{ color:var(--dim); text-decoration:none; }}
.header-links a.here, .header-links a:hover {{ color:var(--ink); }}
.header-links .brand {{ font-weight:700; color:var(--ink); margin-right:auto; }}
.theme-toggle {{ display:inline-flex; align-items:center; justify-content:center;
  width:2.5rem; height:2.5rem; flex:none; border:0; border-radius:50%; padding:0;
  background:transparent; color:var(--dim); cursor:pointer; }}
.theme-toggle:hover {{ background:var(--panel); color:var(--ink); }}
.theme-toggle:focus-visible {{ outline:2px solid var(--link); outline-offset:2px; }}
.theme-toggle svg {{ width:1.25rem; height:1.25rem; fill:none; stroke:currentColor;
  stroke-width:1.7; stroke-linecap:round; stroke-linejoin:round; }}
.theme-toggle .icon-sun {{ display:none; }}
:root[data-theme="dark"] .theme-toggle .icon-sun {{ display:block; }}
:root[data-theme="dark"] .theme-toggle .icon-moon {{ display:none; }}
.site-header .theme-toggle {{ position:absolute; top:50%; right:1.25rem;
  transform:translateY(-50%); }}
.layout {{ max-width:min(860px, 92vw); margin:0 auto; }}
main {{ min-width:0; max-width:76ch; margin-inline:auto; padding:2.5rem 0 5rem; }}
details {{ margin:1rem 0; }}
summary {{ cursor:pointer; color:var(--dim); }}
.contents {{ padding:1rem; background:var(--panel); border-radius:8px; }}
.contents ul {{ padding-left:1.2rem; }}
a {{ color:var(--link); }}
h1, h2, h3 {{ line-height:1.25; margin:2.2rem 0 .8rem; }}
h1 {{ font-size:2rem; margin-top:0; }}
h2 {{ font-size:1.4rem; padding-top:.6rem; border-top:1px solid var(--line); }}
h3 {{ font-size:1.1rem; }}
h1, h2, h3 {{ scroll-margin-top:8rem; }}
code {{ background:var(--panel); padding:.12em .35em; border-radius:4px;
        font-size:.88em; }}
pre {{ background:var(--panel); border:1px solid var(--line); border-radius:8px;
       padding:1rem; overflow-x:auto; }}
.codehilite {{ position:relative; }}
.copy {{ position:absolute; top:.5rem; right:.5rem; border:1px solid var(--line);
         background:var(--bg); color:var(--ink); border-radius:4px;
         padding:.25rem .5rem; cursor:pointer; }}
.codehilite pre {{ padding:1rem 5rem 1rem 1rem; }}
.next {{ border-top:1px solid var(--line); margin-top:3rem; padding-top:1rem;
         display:flex; justify-content:space-between; gap:1rem; }}
pre code {{ background:none; padding:0; font-size:.85rem; line-height:1.55; }}
table {{ border-collapse:collapse; width:100%; display:block; overflow-x:auto; }}
th, td {{ border:1px solid var(--line); padding:.5rem .7rem; text-align:left; }}
th {{ background:var(--panel); }}
blockquote {{ border-left:3px solid var(--line); margin:1rem 0; padding:0 1rem;
              color:var(--dim); }}
img {{ display:block; max-width:100%; height:auto; margin-inline:auto; }}
{nav_css}
{highlight}
</style>
</head><body>
<header class="site-header"><div class="header-links">{nav}</div>{theme_toggle}</header>
<div class="layout"><main>{toc}{body}{next_page}</main></div>
{theme_script}
<script>
document.querySelectorAll('.codehilite').forEach(block => {{
  const button = document.createElement('button');
  button.className = 'copy';
  button.type = 'button';
  button.textContent = 'Copy';
  button.setAttribute('aria-label', 'Copy code to clipboard');
  button.addEventListener('click', async () => {{
    try {{
      await navigator.clipboard.writeText(block.querySelector('code').textContent);
      button.textContent = 'Copied';
    }} catch {{
      button.textContent = 'Select code to copy';
    }}
    setTimeout(() => {{ button.textContent = 'Copy'; }}, 2000);
  }});
  block.appendChild(button);
}});
</script>
</body></html>
"""


def nav_html(here: str, depth: int) -> str:
    up = "../" * depth
    out = [f'<a class="brand" href="{up}index.html">Codimate</a>']
    for href, label in NAV:
        mark = ' class="here" aria-current="page"' if href == here else ""
        icon = (f'<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true">'
                f'{ICONS[label]}</svg>')
        out.append(f'<a href="{up}{href}"{mark} title="{label}" aria-label="{label}">'
                   f'{icon}<span class="nav-label">{label}</span></a>')
    return "\n".join(out)


def theme_toggle() -> str:
    return '''<button class="theme-toggle" type="button"
  aria-label="Switch to night mode" title="Switch to night mode" aria-pressed="false">
  <svg class="icon-moon" viewBox="0 0 24 24" aria-hidden="true">
    <path d="M20.9 13A9 9 0 0 1 11 3.1 9 9 0 1 0 20.9 13Z"/>
  </svg>
  <svg class="icon-sun" viewBox="0 0 24 24" aria-hidden="true">
    <circle cx="12" cy="12" r="4"/>
    <path d="M12 2v2m0 16v2
      M4.93 4.93l1.42 1.42m11.3 11.3 1.42 1.42
      M2 12h2m16 0h2
      M4.93 19.07l1.42-1.42m11.3-11.3 1.42-1.42"/>
  </svg>
</button>'''


def to_html(md_path: Path, dest: str, depth: int = 0) -> str:
    text = md_path.read_text()

    # A Markdown link between two source files has to become a link between
    # two built pages. Rewriting at this one point is why every guide can keep
    # linking the way it does when read on GitHub.
    def relink(m):
        image, label, target = m.group(1), m.group(2), m.group(3)
        # The README links to this repository by absolute URL, because PyPI
        # shows it on its own site where a relative path points nowhere. On
        # the built site those are pages and media it carries, so they are
        # taken back to repository paths and handled like any other.
        for prefix in OWN_URLS:
            if target.startswith(prefix):
                inside = ROOT / target[len(prefix):].split("#")[0]
                anchor = "#" + target.split("#", 1)[1] if "#" in target else ""
                if inside.exists():
                    target = os.path.relpath(inside, md_path.parent) + anchor
                break
        if target.startswith(("http", "#", "mailto:")):
            return m.group(0)
        anchor = ""
        if "#" in target:
            target, anchor = target.split("#", 1)
            anchor = "#" + anchor
        if image:
            source = (md_path.parent / target).resolve()
            if source.is_relative_to(DOCS / "media"):
                media = source.relative_to(DOCS / "media")
                return f"![{label}]({'../' * depth}media/{media}{anchor})"
        if target.startswith("api/") and target.endswith(".html"):
            return f"[{label}]({'../' * depth}{target}{anchor})"
        name = target.split("/")[-1]
        if name.endswith(".md"):
            page = PAGES.get((md_path.parent / target).resolve())
            if page:
                return f"[{label}]({'../' * depth}{page}{anchor})"
        # Anything else — a source file, an example folder — points at GitHub,
        # because the site does not carry the repository.
        rel = (md_path.parent / target).resolve().relative_to(ROOT)
        return f"[{label}](https://github.com/darhnoel/codimate/blob/main/{rel}{anchor})"

    text = re.sub(r"(!?)\[([^\]]+)\]\(([^)]+)\)", relink, text)

    converter = markdown.Markdown(
        extensions=["fenced_code", "codehilite", "tables", "toc",
                    "sane_lists", "md_in_html"],
        extension_configs={"codehilite": {"guess_lang": False},
                           "toc": {"toc_depth": "2-3"}},
    )
    body = converter.convert(text)
    guides = NAV
    position = next((i for i, (href, _) in enumerate(guides) if href == dest), None)
    links = []
    for i, prefix in (() if position is None or dest == "index.html" else
                      ((position - 1, "Previous: "), (position + 1, "Next: "))):
        if 0 <= i < len(guides):
            href, label = guides[i]
            links.append(f'<a href="{"../" * depth}{href}">{prefix}{label}</a>')
    next_page = '<div class="next">' + "".join(links) + "</div>" if links else ""
    toc = ('<details class="contents"><summary>Find a topic on this page</summary>'
           + converter.toc + "</details>"
           if dest == "reference.html" else "")
    title = re.search(r"^#\s+(.+)$", text, re.M)
    return SHELL.format(title=title.group(1) if title else "Codimate",
                        nav=nav_html(dest, depth), theme_toggle=theme_toggle(),
                        nav_css=NAV_CSS,
                        body=body,
                        highlight=HIGHLIGHT, toc=toc,
                        next_page=next_page, theme_script=THEME_SCRIPT,
                        theme_boot=THEME_BOOT)


def main() -> int:
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir()
    shutil.copytree(DOCS / "media", OUT / "media", dirs_exist_ok=True)
    # Google Search Console looks for this exact file at the site root.
    for proof in DOCS.glob("google*.html"):
        shutil.copy(proof, OUT / proof.name)

    for source, dest in PAGES.items():
        (OUT / dest).parent.mkdir(parents=True, exist_ok=True)
        depth = len(Path(dest).parts) - 1
        (OUT / dest).write_text(to_html(source, dest, depth))
        print(f"  {dest}")

    # The API reference, straight from the docstrings.
    subprocess.run(
        [sys.executable, "-m", "pdoc", "-o", str(OUT / "api"),
         "-d", "google", "--no-include-undocumented", "--no-show-source",
         "codimate", "codimate.scene", "codimate.layout",
         "codimate.preview", "codimate.index"],
        cwd=ROOT / "skins/python", check=True,
        env={**__import__("os").environ, "PYTHONPATH": str(ROOT / "skins/python")},
    )
    # pdoc renders its own shell, so the site nav is grafted on afterwards —
    # without it the API pages are a one-way door out of the site.
    #
    # It goes in a `<header>`, not a `<nav>`: pdoc styles bare `nav` as its
    # fixed full-height sidebar, so a `<nav>` here lands underneath that and is
    # never seen. `header` is the element pdoc's own layout leaves room for.
    for page in (OUT / "api").rglob("*.html"):
        depth = len(page.relative_to(OUT).parts) - 1
        bar = nav_html("api/codimate.html", depth)
        header = ('<header class="site-header"><div class="header-links">'
                  + bar + "</div>" + theme_toggle() + "</header>")
        page.write_text(page.read_text().replace("<body>", "<body>" + header, 1))
        html = page.read_text()
        html = html.replace("</head>", THEME_BOOT + PDOC_THEME + "\n</head>", 1)
        html = html.replace("</body>", THEME_SCRIPT + "\n</body>", 1)
        page.write_text(html)
    print("  api/")

    # GitHub Pages runs Jekyll over an artifact unless told not to, and Jekyll
    # drops files beginning with an underscore — which pdoc's search index and
    # assets use.
    (OUT / ".nojekyll").touch()

    print(f"\nbuilt {OUT.relative_to(Path.cwd())} — open site/index.html")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
