#!/usr/bin/env python3
"""Speak this example's Khmer captions with Kiri TTS. Authoring-time only.

    python python/examples/archimedes/narrate.py            # what would be said
    KIRI_API_KEY=... python .../narrate.py --write          # actually record

One mp3 per caption, in `audio/`, named by a hash of the text, voice and
speed. An unchanged line is never sent twice, so re-running after editing one
caption costs one request rather than twenty-two.

Writes `audio/narration.json`: the caption, what was said, its file, and how
long the speech actually runs. **That last number is the point.** The film
paces itself off a guess at reading speed, and real speech has its own length;
feeding the measured duration back is what keeps the voice and the picture
together.

Only subtitles are spoken. Titles are labels on the picture, not narration.

The captions are read out of `vocabulary.py` as data rather than scraped as
text, and formatted with the same numbers `main.py` puts on screen — a line
saying "{left:.0f}%" has to be spoken as "9%", and the cache has to be keyed
on what was really said.

ZWSP word marks are stripped before sending: they are for `chunks()`, and a
speech model should see ordinary text.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
AUDIO = HERE / "audio"
ENDPOINT = "https://api.kiritts.com/v1/audio/speech"

sys.path.insert(0, str(HERE))
import vocabulary  # noqa: E402
import world as W  # noqa: E402

# The same numbers the picture shows, so the voice and the screen agree.
FILLINGS = {"left": 100 * W.LEFT_OF_IT}


def captions(lang: str = "km") -> list[str]:
    """Every subtitle, in the order it is spoken, without repeats.

    A line the film shows twice is read once — the film holds a repeated
    caption rather than saying it again, so there is nothing to record twice.
    """
    _, scenes = vocabulary.pick(lang)
    seen, out = set(), []
    for _, subtitle in scenes.values():
        spoken = vocabulary.spoken(subtitle).format(**FILLINGS)
        if spoken and spoken not in seen:
            seen.add(spoken)
            out.append(spoken)
    return out


def key(text: str, voice: str, speed: float) -> str:
    return hashlib.sha1(f"{voice}|{speed}|{text}".encode()).hexdigest()[:12]


def seconds(path: Path) -> float:
    out = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration",
         "-of", "csv=p=0", str(path)],
        capture_output=True, text=True, check=True).stdout.strip()
    return round(float(out), 3)


def speak(text: str, path: Path, voice: str, speed: float, token: str) -> None:
    """One request, on the standard library.

    This is a POST and a file write, not worth a dependency in a tool that
    runs a handful of times.
    """
    import urllib.error
    import urllib.request

    body = json.dumps({
        "model": "kiritts",
        "input": text,
        "voice": voice,
        "response_format": "mp3",
        "speed": speed,
    }).encode()

    request = urllib.request.Request(
        ENDPOINT, data=body, method="POST",
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
            # Cloudflare sits in front of this API and blocks urllib's default
            # "Python-urllib/3.x" outright — as a 403 carrying "error code:
            # 1010", which reads like a rejected key and is not one.
            "User-Agent": "codimate-narrate/1.0",
            "Accept": "*/*",
        })

    try:
        with urllib.request.urlopen(request, timeout=300) as reply:
            audio = reply.read()
    except urllib.error.HTTPError as failure:
        detail = failure.read()[:400].decode("utf8", "replace")
        raise SystemExit(f"Kiri refused ({failure.code}): {detail}") from None

    # Never leave a truncated or error-page mp3 behind: the cache keys on the
    # text, so a bad file would be silently reused for ever.
    if len(audio) < 1024:
        raise SystemExit(f"suspiciously small reply for {text!r}")
    path.write_bytes(audio)


def api_key() -> str:
    """The Kiri token, from the environment or from a gitignored `.env`."""
    token = os.environ.get("KIRI_API_KEY")
    if token:
        return token
    env = HERE.parents[2] / ".env"
    if env.exists():
        for line in env.read_text().splitlines():
            name, _, value = line.partition("=")
            if name.strip() == "KIRI_API_KEY":
                return value.strip().strip("\"'")
    raise SystemExit(
        f"no Kiri key. Either:\n"
        f"  echo 'KIRI_API_KEY=sk-...' >> {env}\n"
        f"  KIRI_API_KEY=sk-... python narrate.py --write")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--voice", default="Oudom")
    parser.add_argument("--speed", type=float, default=1.0)
    parser.add_argument("--write", action="store_true",
                        help="actually call the API for anything missing")
    args = parser.parse_args()

    lines = captions()
    AUDIO.mkdir(exist_ok=True)
    wanted = [(text, AUDIO / f"{key(text, args.voice, args.speed)}.mp3")
              for text in lines]
    missing = [(text, path) for text, path in wanted if not path.exists()]

    if not args.write:
        print(f"{len(lines)} captions, {len(missing)} not yet spoken "
              f"(voice {args.voice}, speed {args.speed}).\n")
        for text, path in wanted:
            print(f"  {'·' if path.exists() else '+'} {text}")
        print("\nRe-run with --write to record the missing ones.")
        return 0

    token = api_key() if missing else ""
    for i, (text, path) in enumerate(missing, 1):
        print(f"  [{i}/{len(missing)}] {text[:60]}")
        speak(text, path, args.voice, args.speed, token)

    manifest = [{"text": text, "file": path.name, "seconds": seconds(path)}
                for text, path in wanted]
    (AUDIO / "narration.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    spoken = sum(entry["seconds"] for entry in manifest)
    print(f"\n{len(manifest)} lines, {spoken:.1f}s of speech, "
          f"written to {AUDIO.name}/narration.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
