#!/usr/bin/env python3
"""Hear where every word falls in its own recording. Authoring-time only.

    python python/examples/archimedes/align.py            # report only
    KIRI_API_KEY=... python .../align.py --write          # transcribe and write

The film runs a mark along the caption as it is spoken. Where that mark goes
was a *guess* — a word costs a fixed moment plus a little per letter — and a
guess is audibly wrong on a word the voice lingers on.

So each recording is sent back through Kiri, which returns the words it heard
with millisecond timings, and those timings are lined up against the caption
the recording was made from. The result is `audio/timing.json`: for each
caption, the moment each of its words is said, counted from the start of its
own clip.

**From the start of its own clip** is the whole design. An SRT of the finished
film was tried first and it goes stale the moment anything is re-timed — the
transcript supplied was of an earlier cut and its clock had drifted by up to
twenty-two seconds. A clip's own timings drift never: the film places the clip
and the words follow it.

Two more things this has to survive.

**The transcript is not the script.** It is what a machine heard — ដូច្នេះវា
came back as មិញនេះ វាគ — so words are matched by *character*, letting the long
agreeing runs carry the alignment, and any word it never found is filled in
between its neighbours. `custom_vocabulary` biases it toward the spellings the
captions actually use.

**Transcribing is not free.** Every answer is cached under `audio/heard/`, so
re-running after editing one caption costs one request, the same way
`narrate.py` does.
"""

from __future__ import annotations

import argparse
import difflib
import json
import re
import sys
import urllib.error
import urllib.request
import uuid
from pathlib import Path

HERE = Path(__file__).resolve().parent
AUDIO = HERE / "audio"
HEARD = AUDIO / "heard"
ENDPOINT = "https://api.kiritts.com/v1/audio/transcriptions"
MODEL = "kiristt"                # the transcriber; `kiritts` is the voice

sys.path.insert(0, str(HERE))
import narrate  # noqa: E402  — shares the key and the caption list
import vocabulary  # noqa: E402

# A short list of the words that actually get misheard beats a glossary, but
# we cannot know which those are until we have heard them. The longest words
# are the ones a model is least likely to guess, so those go first.
TERMS = 100


def worded(lang="km") -> dict:
    """What each spoken line is, split into the words the mark steps over.

    `narration.json` keys on the *spoken* form, which has no word marks in it
    — they are invisible and a speech model should never see them. The marks
    are what the film steps along, so they are looked back up here.
    """
    _, scenes = vocabulary.pick(lang)
    out = {}
    for _, subtitle in scenes.values():
        line = subtitle.format(**narrate.FILLINGS)
        out[vocabulary.spoken(line)] = line.replace(vocabulary.ZWSP, " ").split()
    return out


def terms() -> str:
    """The caption vocabulary, longest first, as a bias for the transcriber."""
    words = {word for line in narrate.captions()
             for word in vocabulary.plain(line).split()}
    ordered = sorted(words, key=len, reverse=True)[:TERMS]
    return ", ".join(ordered)


def multipart(fields: dict, name: str, blob: bytes) -> tuple[bytes, str]:
    """One file and some plain fields, as multipart/form-data.

    Written out by hand rather than pulling in a dependency for it: this is a
    boundary, some headers and a join, and the tool runs a handful of times.
    """
    edge = uuid.uuid4().hex
    out = bytearray()
    for key, value in fields.items():
        out += (f"--{edge}\r\nContent-Disposition: form-data; "
                f'name="{key}"\r\n\r\n{value}\r\n').encode()
    out += (f"--{edge}\r\nContent-Disposition: form-data; name=\"file\"; "
            f'filename="{name}"\r\n'
            f"Content-Type: audio/mpeg\r\n\r\n").encode()
    out += blob + f"\r\n--{edge}--\r\n".encode()
    return bytes(out), f"multipart/form-data; boundary={edge}"


def listen(clip: Path, token: str, bias: str) -> dict:
    """What Kiri heard in `clip`, with a start and end for every word."""
    body, kind = multipart(
        {"model": MODEL, "language": "km", "response_format": "verbose_json",
         "timestamp_granularities": "word", "custom_vocabulary": bias},
        clip.name, clip.read_bytes())
    request = urllib.request.Request(
        ENDPOINT, data=body, method="POST",
        headers={"Authorization": f"Bearer {token}", "Content-Type": kind,
                 "User-Agent": "codimate-align/1.0", "Accept": "*/*"})
    try:
        with urllib.request.urlopen(request, timeout=300) as reply:
            return json.loads(reply.read())
    except urllib.error.HTTPError as failure:
        detail = failure.read()[:400].decode("utf8", "replace")
        raise SystemExit(f"Kiri refused ({failure.code}): {detail}") from None


def moments(words, said):
    """When each word of `said` is spoken, from the transcript of its clip.

    Matched by character, because the transcript spells what it heard rather
    than what was written. A word the transcript never found is placed between
    the two it did, which is where it was.
    """
    letters, clock = [], []
    for heard in words:
        text = re.sub(r"\s+", "", heard.get("word", ""))
        start, end = float(heard["start"]), float(heard["end"])
        for i, letter in enumerate(text):
            letters.append(letter)
            clock.append(start + (end - start) * i / max(len(text), 1))
    if not letters:
        return None

    script = "".join(said)
    match = difflib.SequenceMatcher(None, script, "".join(letters),
                                    autojunk=False)
    where = {i + d: j + d
             for i, j, n in match.get_matching_blocks() for d in range(n)}

    at, marks = 0, []
    for word in said:
        found = next((where[x] for x in range(at, at + len(word))
                      if x in where), None)
        marks.append(None if found is None else clock[found])
        at += len(word)
    if sum(mark is not None for mark in marks) < max(2, len(said) // 3):
        return None

    # Fill the holes by sharing the gap between the neighbours that were found.
    known = [i for i, mark in enumerate(marks) if mark is not None]
    for i, mark in enumerate(marks):
        if mark is not None:
            continue
        before = max((k for k in known if k < i), default=None)
        after = min((k for k in known if k > i), default=None)
        if before is None:
            marks[i] = marks[after] * i / max(after, 1)
        elif after is None:
            marks[i] = marks[before]
        else:
            step = (marks[after] - marks[before]) / (after - before)
            marks[i] = marks[before] + step * (i - before)
    return [round(mark, 3) for mark in marks]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true",
                        help="transcribe anything not already heard")
    args = parser.parse_args()

    spoken = AUDIO / "narration.json"
    if not spoken.exists():
        raise SystemExit("no audio/narration.json — run narrate.py first")
    recordings = json.loads(spoken.read_text())
    HEARD.mkdir(parents=True, exist_ok=True)

    missing = [entry for entry in recordings
               if not (HEARD / f"{Path(entry['file']).stem}.json").exists()]
    if not args.write:
        print(f"{len(recordings)} recordings, {len(missing)} not yet heard.\n")
        for entry in recordings:
            known = (HEARD / f"{Path(entry['file']).stem}.json").exists()
            print(f"  {'·' if known else '+'} {entry['seconds']:5.2f}s  "
                  f"{entry['text'][:62]}")
        print("\nRe-run with --write to transcribe the missing ones.")
        return 0

    token = narrate.api_key() if missing else ""
    bias = terms()
    for i, entry in enumerate(missing, 1):
        print(f"  [{i}/{len(missing)}] {entry['text'][:56]}")
        got = listen(AUDIO / entry["file"], token, bias)
        (HEARD / f"{Path(entry['file']).stem}.json").write_text(
            json.dumps(got, ensure_ascii=False, indent=2) + "\n")

    words = worded()
    timing, guessed = {}, []
    for entry in recordings:
        got = json.loads((HEARD / f"{Path(entry['file']).stem}.json").read_text())
        said = words.get(entry["text"])
        if not said:
            guessed.append(entry["text"][:40])
            continue
        marks = moments(got.get("words") or [], said)
        if marks is None:
            guessed.append(entry["text"][:40])
        else:
            timing[entry["text"]] = marks

    (AUDIO / "timing.json").write_text(
        json.dumps(timing, ensure_ascii=False, indent=2) + "\n")
    print(f"\n{len(timing)} line(s) timed, {len(guessed)} left to the guess")
    for line in guessed:
        print(f"  ? {line}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
