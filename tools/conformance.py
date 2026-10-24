"""Run every Conformance Example in every language and compare its film data,
byte for byte, with the expected file beside it (ADR 0031).

    python tools/conformance.py            # check
    python tools/conformance.py --bless    # write a missing expected.film.json
                                           # from the Python example
    python tools/conformance.py 001        # only examples whose folder starts so

A language whose runtime is missing is reported, not silently passed.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = ROOT / "conformance"
EXPECTED = "expected.film.json"

# Each language: the file suffix of its examples, and how to run one.
LANGUAGES = {
    "python": (".py", lambda script, out: [sys.executable, str(script), str(out)]),
    "typescript": (".ts", lambda script, out: ["node", str(script), str(out)]),
}


def run(script: Path, language: str, out: Path) -> "str | None":
    """Run one example; its film data, or None with the error printed."""
    _, command = LANGUAGES[language]
    done = subprocess.run(command(script, out), cwd=script.parent,
                          capture_output=True, text=True)
    if done.returncode != 0:
        last = (done.stderr or done.stdout).strip().splitlines()[-8:]
        print(f"    {language}: failed\n" + "\n".join("      " + line for line in last))
        return None
    return out.read_text()


def first_difference(a: str, b: str) -> str:
    at = next((i for i, (x, y) in enumerate(zip(a, b)) if x != y), min(len(a), len(b)))
    lo, hi = max(0, at - 40), at + 40
    return f"at byte {at}: …{a[lo:hi]}… vs …{b[lo:hi]}…"


def main(argv: list[str]) -> int:
    bless = "--bless" in argv
    only = [a for a in argv if not a.startswith("--")]
    failures = 0
    for folder in sorted(p for p in EXAMPLES.iterdir() if p.is_dir()):
        if only and not any(folder.name.startswith(o) for o in only):
            continue
        print(folder.name)
        expected = folder / EXPECTED
        with tempfile.TemporaryDirectory() as tmp:
            for language, (suffix, _) in LANGUAGES.items():
                scripts = sorted(folder.glob(f"*{suffix}"))
                if not scripts:
                    print(f"    {language}: no example")
                    failures += 1
                    continue
                if language == "typescript" and shutil.which("node") is None:
                    print(f"    {language}: node is not installed")
                    failures += 1
                    continue
                data = run(scripts[0], language, Path(tmp) / f"{language}.json")
                if data is None:
                    failures += 1
                    continue
                if not expected.exists():
                    if bless and language == "python":
                        expected.write_text(data)
                        print(f"    {language}: wrote {EXPECTED}")
                        continue
                    print(f"    {language}: no {EXPECTED} (run with --bless)")
                    failures += 1
                    continue
                want = expected.read_text()
                if data == want:
                    print(f"    {language}: ok")
                else:
                    print(f"    {language}: DIFFERS {first_difference(data, want)}")
                    failures += 1
    print("\nall examples agree" if failures == 0 else f"\n{failures} failure(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
