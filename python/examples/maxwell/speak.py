#!/usr/bin/env python3
"""Speak this film's captions with Kiri TTS, then hear where each word falls.
Authoring-time only.

    python python/examples/maxwell/speak.py            # what would be done
    python python/examples/maxwell/speak.py --write    # record and hear what is missing

Two jobs, one per API call, both cached so an unchanged line is never paid for
twice:

1. **Speak.** One mp3 per caption in `audio/`, named by a hash of the text, voice
   and speed. What is sent is `lines.SPEAK` — the caption without word marks, and
   with symbols written out — never the marked text, which a speech model should
   not see. Written to `audio/narration.json` with how long each clip really runs.
2. **Hear.** Each clip goes back through Kiri's transcriber, which returns the
   words it heard with millisecond timings. Those are lined up against the
   caption's own words, character by character, because a transcript spells what
   it heard and not what was written. The result is `audio/timing.json`: for each
   caption, the moment each word is said, counted from the start of its own clip.
   `main.py` steps the bright mark on those moments, so it is on the word being
   spoken rather than near it.

The request, the cache key, the key lookup and the aligner are the ones in
`archimedes/narrate.py` and `archimedes/align.py`. The token is `KIRI_API_KEY`
in the environment or in the gitignored `.env` at the repository root.
"""

import argparse
import difflib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
AUDIO = HERE / "audio"
HEARD = AUDIO / "heard"

sys.path.insert(0, str(HERE.parent / "archimedes"))
sys.path.append(str(HERE.parent / "year"))
import align as hear  # noqa: E402
import caption  # noqa: E402
import narrate as kiri  # noqa: E402

import lines  # noqa: E402


def words(key: str) -> list:
    """The pieces the caption's mark steps over, exactly as the film sees them."""
    return [piece for piece, _ in caption.chunks(lines.SAY[key])]


def vocabulary() -> str:
    """The caption words, longest first, to bias the transcriber toward them."""
    every = {word for key in lines.SAY for word in words(key)}
    return ", ".join(sorted(every, key=len, reverse=True)[:100])


def suspect(entries) -> None:
    """Name any recording that does not say what it was asked to say: a clip that
    comes back much longer than its line, or that disagrees with it."""
    bad = []
    for entry in entries:
        got = json.loads((HEARD / f"{Path(entry['file']).stem}.json").read_text())
        said = "".join(entry["text"].split())
        back = "".join(got.get("text", "").split())
        grew = len(back) / max(len(said), 1)
        agrees = difflib.SequenceMatcher(None, said, back, autojunk=False).ratio()
        if grew > 1.35 or agrees < 0.55:
            bad.append((entry, grew, agrees))
    if not bad:
        return
    print(f"\n{len(bad)} recording(s) do not clearly say their line:")
    for entry, grew, agrees in bad:
        print(f"  ! x{grew:.2f} length, {agrees:.0%} agreement  {entry['text'][:46]}")
    print("\nListen to them. To redo one, delete its mp3 and its audio/heard/ entry.")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--voice", default="Oudom")
    parser.add_argument("--speed", type=float, default=1.0)
    parser.add_argument(
        "--write",
        action="store_true",
        help="actually call the API for anything missing",
    )
    args = parser.parse_args()

    AUDIO.mkdir(exist_ok=True)
    HEARD.mkdir(exist_ok=True)
    wanted = []
    for key in lines.SAY:
        text = lines.SPEAK[key]
        wanted.append(
            (key, text, AUDIO / f"{kiri.key(text, args.voice, args.speed)}.mp3")
        )
    unspoken = [item for item in wanted if not item[2].exists()]
    unheard = [item for item in wanted if not (HEARD / f"{item[2].stem}.json").exists()]

    if not args.write:
        print(
            f"{len(wanted)} captions: {len(unspoken)} not yet spoken, "
            f"{len(unheard)} not yet heard (voice {args.voice}, speed {args.speed}).\n"
        )
        for key, text, path in wanted:
            spoken = "·" if path.exists() else "+"
            heard = "·" if (HEARD / f"{path.stem}.json").exists() else "+"
            print(f"  speak {spoken}  hear {heard}  {key:9s} {text}")
        print("\nRe-run with --write to do the ones marked +.")
        return 0

    token = kiri.api_key() if unspoken or unheard else ""
    for i, (key, text, path) in enumerate(unspoken, 1):
        print(f"  speak [{i}/{len(unspoken)}] {key}: {text[:56]}")
        kiri.speak(text, path, args.voice, args.speed, token)

    manifest = [
        {"text": text, "file": path.name, "seconds": kiri.seconds(path)}
        for _, text, path in wanted
    ]
    (AUDIO / "narration.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    )

    bias = vocabulary()
    unheard = [item for item in wanted if not (HEARD / f"{item[2].stem}.json").exists()]
    for i, (key, text, path) in enumerate(unheard, 1):
        print(f"  hear  [{i}/{len(unheard)}] {key}")
        got = hear.listen(path, token, bias)
        (HEARD / f"{path.stem}.json").write_text(
            json.dumps(got, ensure_ascii=False, indent=2) + "\n"
        )

    timing, guessed = {}, []
    for key, text, path in wanted:
        got = json.loads((HEARD / f"{path.stem}.json").read_text())
        marks = hear.moments(got.get("words") or [], words(key))
        if marks is None:
            guessed.append(key)
        else:
            timing[text] = marks
    (AUDIO / "timing.json").write_text(
        json.dumps(timing, ensure_ascii=False, indent=2) + "\n"
    )

    total = sum(entry["seconds"] for entry in manifest)
    print(
        f"\n{len(manifest)} lines, {total:.1f}s of speech; "
        f"{len(timing)} timed word by word, {len(guessed)} left to the reading guess"
    )
    for key in guessed:
        print(f"  ? {key}")
    suspect(manifest)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
