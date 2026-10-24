"""Run the self-checks.

    python skins/python/tests/run.py                    # no video; a second or two
    python skins/python/tests/run.py --all              # also runs the guide's programs

One of these files renders real video: `test_docs` runs every complete program in
the guide. That is the right check to run before committing and the wrong one to run
after every edit, so it is opt-in.

The examples are in their own repository, https://github.com/darhnoel/codimate-examples,
with their own checks (every example builds, every formula typesets, an opt-in render).
"""

import importlib
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import support  # noqa: E402  (sets up the import path for `codimate`)

HERE = Path(__file__).resolve().parent

RENDERS_VIDEO = {"test_docs"}
everything = {"--all", "-a"} & set(sys.argv)

# Example names after the flag narrow the render to those. Passed by env var
# because these files are imported, not launched, so they cannot read argv
# without agreeing on a parser.
only = [a for a in sys.argv[1:] if not a.startswith("-")]
if only:
    os.environ["CODIMATE_ONLY"] = ",".join(only)

failed = skipped = 0
for path in sorted(HERE.glob("test_*.py")):
    if path.stem in RENDERS_VIDEO and not everything:
        skipped += 1
        continue
    # Narrowing to an example is about that example, not about the guide.
    if only and path.stem == "test_docs":
        skipped += 1
        continue
    failed += support.run(vars(importlib.import_module(path.stem)))

if skipped:
    print(f"\nskipped {skipped} file(s) that render video"
          " — run with --all before committing")
if failed:
    print(f"\n{failed} failed")
sys.exit(1 if failed else 0)
