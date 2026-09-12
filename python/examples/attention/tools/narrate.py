#!/usr/bin/env python3
"""Speak this example's captions with Kiri TTS. Authoring-time only.

    python tools/narrate.py                      # show what would be spoken
    KIRI_API_KEY=... python tools/narrate.py --write

One mp3 per caption, in `audio/`, named by a hash of the text, voice and
speed. An unchanged line is never sent twice, so re-running after editing one
caption costs one request, not twenty.

Writes `audio/narration.json`: the caption, its file, and how long the speech
actually runs. That last number is the point — the beats currently pace
themselves off a guess at reading speed, and real speech has its own length.
Feeding the measured duration back is what keeps voice and picture together.

ZWSP word marks are stripped before sending: they are for `view.chunks`, and a
speech model should see ordinary text.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
MAIN = HERE.parent / "main.py"
AUDIO = HERE.parent / "audio"
ENDPOINT = "https://api.kiritts.com/v1/audio/speech"
ZWSP = "​"

# Captions are the only Khmer string literals in the file.
KHMER_LITERAL = re.compile(r"'([^'\n]*[ក-៿][^'\n]*)'")


SPOKEN_FORMS = AUDIO / "spoken.json"
BRACKETED = re.compile(r"\s*\([^)]*\)")
MATH = re.compile(r"\$[^$]+\$")


def for_speech(caption: str, strict: bool = True) -> str:
    """The caption as it should be *said*, not as it is drawn.

    Two things on screen must not be read aloud:

    `$...$` is LaTeX. Sent as written, the voice says "dollar backslash sqrt
    brace d underscore k". Each one needs a spoken form, written by hand in
    `audio/spoken.json` — there is no reliable way to say an equation aloud in
    another language automatically.

    `(Token)`, `(Query)` and so on are glosses for the reader's eye. Spoken,
    they interrupt the sentence.

    Both change the speech without changing the picture, so the mark spreads
    the voice's time over pieces that were never said — a small wobble inside
    a line. It cannot accumulate: every caption is timed and cued on its own,
    so each line still starts and ends with its own voice.
    """
    forms = json.loads(SPOKEN_FORMS.read_text()) if SPOKEN_FORMS.exists() else {}

    def say_math(found):
        latex = found.group()[1:-1].strip()
        if not forms.get(latex):
            if not strict:
                # A dry run is for *reading* the script, so show the gap rather
                # than refusing. Only an actual recording insists.
                return f"<<{latex}>>"
            entry = json.dumps({latex: "…"}, ensure_ascii=False, indent=2)
            raise SystemExit(
                f"no spoken form for the equation {latex!r}.\n"
                f"Add it to {SPOKEN_FORMS.name}:\n{entry}")
        return forms[latex]

    spoken = MATH.sub(say_math, caption)
    spoken = BRACKETED.sub("", spoken)
    return " ".join(spoken.split())


def captions() -> list[str]:
    """Every caption, in the order it is spoken, without duplicates."""
    seen, out = set(), []
    for source in KHMER_LITERAL.findall(MAIN.read_text()):
        # The file is read as text, so a literal reaches us in *source* form:
        # `\\sqrt` here is two characters, where the running program sees one.
        # Evaluating the literal gives the string main.py actually builds, which
        # is what the manifest must be keyed on for the render to find it.
        text = ast.literal_eval("'" + source + "'")
        spoken = text.replace(ZWSP, "")
        if spoken not in seen:
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
    """One request, on the standard library — this is a POST and a file write,
    not worth a dependency in a tool that runs a handful of times."""
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
            # 1010", which reads like a rejected key and is not one. Any real
            # User-Agent gets through to the actual reply.
            "User-Agent": "codimate-narrate/1.0",
            "Accept": "*/*",
        })

    try:
        with urllib.request.urlopen(request, timeout=300) as response:
            audio = response.read()
    except urllib.error.HTTPError as failure:
        # The body is where the useful message lives; without this the tool
        # raises a bare status and the reason stays hidden.
        detail = failure.read()[:400].decode("utf8", "replace")
        raise SystemExit(f"Kiri refused ({failure.code}): {detail}") from None

    # Never leave a truncated or error-page mp3 behind: the cache keys on the
    # text, so a bad file would be silently reused for ever.
    if len(audio) < 1024:
        raise SystemExit(f"suspiciously small reply for {text!r}: {audio[:200]!r}")
    path.write_bytes(audio)


def api_key() -> str:
    """The Kiri token, from the environment or from a gitignored `.env`.

    A `.env` at the repository root is the convenient place — it survives
    across shells and never reaches a commit. The environment still wins, so
    CI or a one-off run can override it without editing a file.
    """
    token = os.environ.get("KIRI_API_KEY")
    if token:
        return token

    env = HERE.parents[3] / ".env"
    if env.exists():
        for line in env.read_text().splitlines():
            name, _, value = line.partition("=")
            if name.strip() == "KIRI_API_KEY":
                return value.strip().strip("\"'")

    raise SystemExit(
        f"no Kiri key. Either:\n"
        f"  echo 'KIRI_API_KEY=sk-...' >> {env}\n"
        f"  KIRI_API_KEY=sk-... python tools/narrate.py --write")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--voice", default="Nita")
    parser.add_argument("--speed", type=float, default=1.0)
    parser.add_argument("--write", action="store_true",
                        help="actually call the API for anything missing")
    args = parser.parse_args()

    lines = captions()
    AUDIO.mkdir(exist_ok=True)

    # Keyed on what is *said*, so changing how an equation is pronounced
    # re-records, and changing only a bracketed gloss does not.
    wanted = [(text, for_speech(text, strict=args.write)) for text in lines]
    wanted = [(text, spoken, AUDIO / f"{key(spoken, args.voice, args.speed)}.mp3")
              for text, spoken in wanted]
    missing = [(spoken, path) for _, spoken, path in wanted if not path.exists()]

    if not args.write:
        print(f"{len(lines)} captions, {len(missing)} not yet spoken.\n")
        for text, spoken, _ in wanted:
            if spoken != text:
                print(f"  drawn : {text}\n  spoken: {spoken}\n")
        print("\nRe-run with --write (and KIRI_API_KEY set) to generate.")
        return 0

    token = api_key()

    for i, (spoken, path) in enumerate(missing, 1):
        print(f"[{i}/{len(missing)}] {spoken[:44]}")
        speak(spoken, path, args.voice, args.speed, token)

    manifest = [
        {"text": text, "spoken": spoken, "file": path.name,
         "seconds": seconds(path)}
        for text, spoken, path in wanted
    ]
    (AUDIO / "narration.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2))

    total = sum(entry["seconds"] for entry in manifest)
    print(f"\n{len(manifest)} clips, {total:.1f}s of speech "
          f"-> {AUDIO.name}/narration.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
