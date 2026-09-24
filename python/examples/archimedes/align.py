#!/usr/bin/env python3
"""Read a transcript of the narration and learn how fast each word is said.

    python python/examples/archimedes/align.py            # report only
    python python/examples/archimedes/align.py --write    # write the shares

The film paces its running mark by *guessing*: a word costs a fixed moment
plus a little per letter. That is a fair guess and it is still the fallback,
but a transcript with timestamps knows better — it knows which word the voice
lingers on.

So this takes an SRT of the narration, lines the words up against the captions
in `vocabulary.py`, and writes `audio/timing.json`: for each caption, what
share of its own length each word is worth. Shares, not seconds, and that is
the point — the film has been re-rendered a dozen times and every render moves
the clock, so absolute times go stale immediately. A *share* survives, and the
recording's own measured length is what it gets scaled onto.

Two things make this harder than it sounds, and both are handled here.

**The transcript is not the script.** It is what a machine heard, so it spells
things its own way — អណ្តែត for អណ្ដែត, សំបក for សម្បក. Matching word to word
would fail on those; matching *characters* and letting the long agreeing runs
carry the alignment does not. It currently agrees on about 96% of them.

**The transcript's phrases straddle the captions.** An SRT of the finished
film cuts where the speaker pauses, not where a caption ends, so one cue often
holds the end of one line and the start of the next — with the film's silence
between them, inside the cue. Left alone, that silence is read as a very slow
word. Any gap longer than `GAP` is therefore clipped: it is the film's pause,
not the voice's.
"""

from __future__ import annotations

import argparse
import difflib
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
AUDIO = HERE / "audio"

sys.path.insert(0, str(HERE))
import vocabulary  # noqa: E402
import world as W  # noqa: E402

FILLINGS = {"left": 100 * W.LEFT_OF_IT}

GAP = 0.40          # longer than this between two words is the film's silence
LEAST = 4           # fewer matched words than this and the guess is better

# The order the film speaks its captions in. A line the film holds rather than
# re-reads is spoken once, so it appears once.
ORDER = ("hook hook2 simpler waterbox displace neutral why_neutral ice lighter "
         "push more release shrink stop why derive percent steel steelforce "
         "sinks question hollowing rising floats average chart law said").split()


def cues(path: Path):
    """The SRT, as `(start, end, text)`, with silence and markup dropped."""
    out = []
    for block in re.split(r"\n\s*\n", path.read_text().lstrip("﻿").strip()):
        lines = [ln for ln in block.splitlines() if ln.strip()]
        if len(lines) < 3 or "-->" not in lines[1]:
            continue
        start, _, end = lines[1].partition(" --> ")
        text = re.sub(r"<[^>]+>", "", " ".join(lines[2:])).strip()
        if text and not text.startswith("["):
            out.append((seconds(start), seconds(end), text))
    return out


def seconds(stamp: str) -> float:
    hours, minutes, rest = stamp.strip().split(":")
    return int(hours) * 3600 + int(minutes) * 60 + float(rest.replace(",", "."))


def lettered(spans):
    """Every character of the transcript, with the moment it is spoken.

    Character by character rather than word by word, because the alignment
    that follows is a character alignment — a transcript that spells a word
    differently still agrees on most of its letters.
    """
    letters, clock = [], []
    for start, end, text in spans:
        words = text.split()
        total = sum(len(word) for word in words) or 1
        at = start
        for word in words:
            span = (end - start) * len(word) / total
            for i, letter in enumerate(word):
                letters.append(letter)
                clock.append(at + span * i / max(len(word), 1))
            at += span
    return "".join(letters), clock


def script(lang="km"):
    """The captions the film speaks, in order, each as its list of words."""
    _, scenes = vocabulary.pick(lang)
    said, out = "", []
    for key in ORDER:
        line = scenes[key][1].format(**FILLINGS)
        if out and line == out[-1][2]:
            continue                                   # held, not said again
        words = line.replace(vocabulary.ZWSP, " ").split()
        out.append((key, words, line, len(said)))
        said += "".join(words)
    return out, said


def shares(words, at, where, clock):
    """What share of its caption's length each word is worth.

    `None` when the transcript does not cover the line well enough to be
    worth trusting over the guess.
    """
    marks, i = [], at
    for word in words:
        found = next((where[x] for x in range(i, i + len(word)) if x in where),
                     None)
        marks.append(None if found is None else clock[found])
        i += len(word)

    known = [(n, t) for n, t in enumerate(marks) if t is not None]
    if len(known) < LEAST:
        return None

    # Start to start, with the film's own pauses clipped out of it.
    steps = {a: min(t - was, GAP)
             for (a, was), (_, t) in zip(known, known[1:])}
    typical = sorted(steps.values())[len(steps) // 2] if steps else GAP
    spread = [steps.get(n, typical) for n in range(len(words) - 1)] + [typical]
    whole = sum(spread)
    return [step / whole for step in spread]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("srt", nargs="?",
                        default=AUDIO / "meatika-archimedes-km.srt", type=Path)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()

    if not args.srt.exists():
        raise SystemExit(f"no transcript at {args.srt}")

    letters, clock = lettered(cues(args.srt))
    lines, said = script()
    match = difflib.SequenceMatcher(None, said, letters, autojunk=False)
    where = {i + d: j + d
             for i, j, n in match.get_matching_blocks() for d in range(n)}

    print(f"{args.srt.name}: {len(letters)} characters, "
          f"{match.ratio():.0%} of the script agrees with it\n")

    timing, guessed = {}, []
    for key, words, line, at in lines:
        got = shares(words, at, where, clock)
        if got is None:
            guessed.append(key)
        else:
            timing[vocabulary.spoken(line)] = [round(s, 5) for s in got]
        print(f"  {'·' if got else '?'} {key:12} {len(words):3} words"
              f"{'' if got else '   — guessed, too little matched'}")

    if guessed:
        print(f"\n{len(guessed)} line(s) keep the reading guess: "
              f"{', '.join(guessed)}")
    if not args.write:
        print(f"\n{len(timing)} line(s) would be timed. Re-run with --write.")
        return 0

    (AUDIO / "timing.json").write_text(
        json.dumps(timing, ensure_ascii=False, indent=2) + "\n")
    print(f"\nwrote {AUDIO.name}/timing.json — {len(timing)} lines timed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
