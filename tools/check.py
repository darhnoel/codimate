"""Every check a change to the Authoring Surface must pass, in order, stopping
at the first failure. Run it before every commit.

    python tools/check.py           # build what the checks need, then check
    python tools/check.py --fast    # skip rebuilding the extension and the engine

1. the generated files match the Interface Manifest
2. formatting and lint (cargo fmt, ruff)
3. the Rust workspace's tests, the Authoring Core's included
4. the Python Skin's tests (no video)
5. the JavaScript Skin's tests
6. every Conformance Example, in every language
7. the cost of crossing the Waist
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PY = str(ROOT / ".venv/bin/python")


def steps(fast: bool):
    yield "generated files", [PY, "tools/generate.py", "--check"]
    yield "cargo fmt", ["cargo", "fmt", "--check"]
    yield "ruff", [str(ROOT / ".venv/bin/ruff"), "check"]
    yield "rust tests", ["cargo", "test", "-q", "--workspace"]
    if not fast:
        yield "build the Python Skin", [str(ROOT / ".venv/bin/maturin"), "develop",
                                        "--release", "-q"]
        yield "build the JavaScript Skin", [PY, "tools/build_js.py"]
    yield "python tests", [PY, "python/tests/run.py"]
    yield "javascript tests", ["node", "--test", "skins/js/test/skin.test.ts"]
    yield "conformance", [PY, "tools/conformance.py"]
    yield "waist cost", [PY, "tools/bench_waist.py"]


def main(argv: list[str]) -> int:
    env = {**os.environ, "VIRTUAL_ENV": str(ROOT / ".venv")}
    for name, command in steps("--fast" in argv):
        print(f"── {name}", flush=True)
        done = subprocess.run(command, cwd=ROOT, env=env, capture_output=True,
                              text=True)
        if done.returncode != 0:
            print((done.stdout + done.stderr).strip()[-3000:])
            print(f"\nFAILED: {name}")
            return 1
    print("\nevery check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
