"""Shared setup for the self-checks.

These are plain `assert` tests with no framework. Each file runs on its own:

    python python/tests/test_layout.py

and the whole set runs with:

    python python/tests/run.py

They are also valid pytest modules if you would rather use that.
"""

import sys
import traceback
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))


def run(namespace) -> int:
    """Run every test_* in a module. Returns the number that failed."""
    tests = sorted(
        (name, fn)
        for name, fn in namespace.items()
        if name.startswith("test_") and callable(fn)
    )
    failed = 0
    for name, fn in tests:
        try:
            fn()
        except Exception:
            failed += 1
            print(f"FAIL  {name}")
            traceback.print_exc()
    if not failed:
        print(f"ok    {len(tests)} tests in {namespace['__name__']}")
    return failed
