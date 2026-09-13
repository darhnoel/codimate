"""Run the self-checks.

    python python/tests/run.py           # everything except rendering video
    python python/tests/run.py --all     # including it, before you commit

Two of these files render real video: `test_examples` renders all eight
examples, `test_docs` runs every complete program in the guide. Together they
take about two minutes when everything is stale — and everything goes stale
whenever `python/codimate/` is touched, because any change there *might* change
every video.

That is the right check to run before committing and the wrong one to run after
every edit, so it is opt-in. The fast checks take 0.04 seconds, which is the
difference between running them constantly and avoiding them.
"""

import importlib
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import support  # noqa: E402  (sets up the import path for `codimate`)

HERE = Path(__file__).resolve().parent

RENDERS_VIDEO = {"test_examples", "test_docs"}
everything = {"--all", "-a"} & set(sys.argv)

failed = skipped = 0
for path in sorted(HERE.glob("test_*.py")):
    if path.stem in RENDERS_VIDEO and not everything:
        skipped += 1
        continue
    failed += support.run(vars(importlib.import_module(path.stem)))

if skipped:
    print(f"\nskipped {skipped} file(s) that render video"
          " — run with --all before committing")
if failed:
    print(f"\n{failed} failed")
sys.exit(1 if failed else 0)
