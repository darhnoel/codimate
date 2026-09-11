"""Every code block in the guide still runs.

The examples have `test_examples.py`. The documentation had nothing: its code
was only ever checked by hand, which means it could drift from the library
between one release and the next and nobody would know until a reader hit it.

Blocks are **discovered**, not listed, so a new chapter is covered the day it
is written.

Blocks come in three kinds and are treated differently:

* a **complete program** imports codimate and calls `.render(`, so it is run
  as written — which is why `results/` gains the guide's own videos after a
  full test run;
* a **signature sketch** like `rect(name, h=, w=)` is deliberately not valid
  Python and is skipped;
* everything else is a **fragment**: it cannot run, because it refers to names
  it never defines, but it can be compiled, which catches the typo that would
  otherwise waste a reader's afternoon.

Two fragment shapes need a little help before they will compile, and both are
normal ways to write documentation: a call shown without its receiver
(`.render(...)` alone) gets a placeholder receiver, and a block ending in a
colon — one line of a `for` shown as the line you are meant to change — gets a
body. The syntax inside them is checked either way.
"""

import re
import subprocess
import sys
from pathlib import Path

import support  # noqa: F401  (puts `codimate` on the import path)

ROOT = Path(__file__).resolve().parents[2]
PAGES = sorted([ROOT / "README.md"] + list((ROOT / "docs").glob("*.md")))
BLOCK = re.compile(r"```python\n(.*?)```", re.S)

# Notation a reference page legitimately contains, none of which is Python:
#   rect(name, h=, w=)          parameter names with no values
#   explain(*, trace, view)     a keyword-only marker, invalid in a call
#   ... -> Explanation          a return annotation
#   <anchors>                   a placeholder for a group of parameters
# Listed rather than inferred from "it did not compile", so a real typo in a
# signature still fails instead of being quietly excused.
SKETCH = re.compile(r"=\s*[,)\n]|\(\*,|, \*,|\)\s*->|<\w+>")


def _kind(code):
    if "import codimate" in code and ".render(" in code:
        return "program"
    return "sketch" if SKETCH.search(code) else "fragment"


def _blocks(page):
    return BLOCK.findall(page.read_text())


def test_the_guide_has_code_to_check():
    found = {p.name: len(_blocks(p)) for p in PAGES if _blocks(p)}
    assert found, "no python blocks found — has the layout changed?"
    assert "tutorial.md" in found, f"the tutorial has no code: {found}"


def test_every_complete_program_in_the_guide_runs():
    """Run as written, from the repository root, exactly as a reader would."""
    ran = 0
    for page in PAGES:
        for i, code in enumerate(_blocks(page), 1):
            if _kind(code) != "program":
                continue
            script = ROOT / f"_doccheck_{page.stem}_{i}.py"
            script.write_text(code)
            try:
                done = subprocess.run([sys.executable, str(script)], cwd=ROOT,
                                      capture_output=True, text=True)
            finally:
                script.unlink()
            assert done.returncode == 0, (
                f"{page.name} block {i} failed:\n{done.stderr[-1500:]}")
            ran += 1
    assert ran >= 2, f"only {ran} complete programs found in the guide"


def test_every_fragment_in_the_guide_is_valid_python():
    """A fragment cannot be run — it refers to names it never defines — but it
    can still be parsed, which catches the typo that would waste a reader's
    afternoon."""
    for page in PAGES:
        for i, code in enumerate(_blocks(page), 1):
            if _kind(code) != "fragment":
                continue
            # `.render(...)` shown without its receiver is still worth
            # checking inside the brackets.
            standalone = "\n".join("_" + ln if ln.startswith(".") else ln
                                    for ln in code.split("\n"))
            body = [ln for ln in standalone.split("\n") if ln.strip()]
            if body and body[-1].rstrip().endswith(":"):
                standalone += "\n    pass"
            try:
                compile(standalone, f"{page.name}:{i}", "exec")
            except SyntaxError as e:
                raise AssertionError(
                    f"{page.name} block {i} is not valid Python: {e}") from None


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
