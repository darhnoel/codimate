#!/usr/bin/env python3
"""Speak this film's Khmer captions with Kiri TTS. Authoring-time only.

    python python/examples/year/speak.py            # what would be said
    python python/examples/year/speak.py --write    # record what is missing

The same job as `archimedes/narrate.py`, whose request, cache key and key
lookup it borrows: the token is `KIRI_API_KEY` in the environment or in the
gitignored `.env` at the repository root, one mp3 per caption in `audio/` named
by a hash of the text, voice and speed, and an unchanged line is never sent
twice.

It writes `audio/narration.json` — each caption, its file, and how long the
speech really runs. `main.py` stretches the word-by-word mark over that length,
so the picture is held for the voice rather than for a guess at reading speed.
The film is the same without the file; it just reads silently.

The ZWSP word marks are stripped before sending: they are for the caption's
mark, and a speech model should see ordinary text.
"""

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
AUDIO = HERE / "audio"

sys.path.insert(0, str(HERE.parent / "archimedes"))
import narrate as kiri  # noqa: E402

import lines  # noqa: E402


def spoken(line: str) -> str:
    """What is said for a caption: its words, without the marks."""
    return line.replace("​", "")


def captions() -> list:
    """Every caption, in the order the film shows them."""
    return [spoken(line) for line in lines.SAY.values()]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--voice", default="Oudom")
    parser.add_argument("--speed", type=float, default=1.0)
    parser.add_argument("--write", action="store_true",
                        help="actually call the API for anything missing")
    args = parser.parse_args()

    AUDIO.mkdir(exist_ok=True)
    wanted = [(text, AUDIO / f"{kiri.key(text, args.voice, args.speed)}.mp3")
              for text in captions()]
    missing = [(text, path) for text, path in wanted if not path.exists()]

    if not args.write:
        print(f"{len(wanted)} captions, {len(missing)} not yet spoken "
              f"(voice {args.voice}, speed {args.speed}).\n")
        for text, path in wanted:
            print(f"  {'·' if path.exists() else '+'} {text}")
        print("\nRe-run with --write to record the missing ones.")
        return 0

    token = kiri.api_key() if missing else ""
    for i, (text, path) in enumerate(missing, 1):
        print(f"  [{i}/{len(missing)}] {text[:60]}")
        kiri.speak(text, path, args.voice, args.speed, token)

    manifest = [{"text": text, "file": path.name, "seconds": kiri.seconds(path)}
                for text, path in wanted]
    (AUDIO / "narration.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    total = sum(entry["seconds"] for entry in manifest)
    print(f"\n{len(manifest)} lines, {total:.1f}s of speech, "
          f"written to {AUDIO.name}/narration.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
