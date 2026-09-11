"""Run every self-check.

    python python/tests/run.py
"""

import importlib
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import support  # noqa: E402  (sets up the import path for `codimate`)

HERE = Path(__file__).resolve().parent
failed = 0

for path in sorted(HERE.glob("test_*.py")):
    failed += support.run(vars(importlib.import_module(path.stem)))

if failed:
    print(f"\n{failed} failed")
sys.exit(1 if failed else 0)
