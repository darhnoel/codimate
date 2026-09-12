"""Run every self-check.

    python python/tests/run.py
"""

import importlib
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import support  # noqa: E402  (sets up the import path for `codimate`)

HERE = Path(__file__).resolve().parent

# The example checks render real video, so they take about 15 seconds. Skip
# them with --fast while iterating on the library itself.
SLOW = {"test_examples", "test_docs"}
fast = "--fast" in sys.argv

failed = 0
for path in sorted(HERE.glob("test_*.py")):
    if fast and path.stem in SLOW:
        print(f"skip  {path.stem} (--fast)")
        continue
    failed += support.run(vars(importlib.import_module(path.stem)))

if failed:
    print(f"\n{failed} failed")
sys.exit(1 if failed else 0)
