#!/usr/bin/env python3
"""Mark Khmer word boundaries in this example's captions. Authoring-time only.

    python tools/segment_captions.py --check     # report, change nothing
    python tools/segment_captions.py --write     # rewrite main.py in place

Khmer does not separate words with spaces, so a caption cannot be revealed
word by word without knowing where its words are. Finding that out needs a
dictionary and a Viterbi search — far too much to carry into a render.

So it happens here instead, once, and the answer is written into the caption
itself as U+200B ZERO WIDTH SPACE — the character Khmer already uses to mark a
word boundary. `view.chunks()` splits on it at render time. The video needs no
segmenter, no dictionary and no model; the text arrives already knowing where
its own words end. Same split the `khmerime-lab` dict-segment tool makes for
the same reason: the model stays in the lab, only its output ships.

ZWSP is invisible, so the captions still read exactly as written.

The segmenter is vendored in a sibling repository rather than published, so
point at it:

    --segmenter ~/Developments/khmerime-lab/tools/dict-segment/vendor
"""

from __future__ import annotations

import argparse
import ast
import re
import sys
from pathlib import Path

ZWSP = "​"
HERE = Path(__file__).resolve().parent
MAIN = HERE.parent / "main.py"

DEFAULT_SEGMENTER = Path.home() / "Developments/khmerime-lab/tools/dict-segment/vendor"

# A quoted string holding Khmer. Captions are the only Khmer in the file, and
# they are always single-quoted literals.
KHMER_LITERAL = re.compile(r"'([^'\n]*[ក-៿][^'\n]*)'")


def load_segmenter(vendor: Path):
    sys.path.insert(0, str(vendor))
    try:
        from khmer_segmenter import KhmerSegmenter  # type: ignore
    except ImportError as exc:  # pragma: no cover - depends on a sibling repo
        raise SystemExit(
            f"no khmer_segmenter under {vendor}\n"
            "Pass --segmenter <path to the vendor directory>."
        ) from exc

    data = vendor / "khmer_segmenter" / "dictionary_data"
    return KhmerSegmenter(
        str(data / "khmer_dictionary_words.txt"),
        str(data / "khmer_word_frequencies.json"),
    )


# Khmer punctuation that belongs to the word before it: a full stop, the
# repetition sign, and so on. These are Khmer characters, so the rule below
# would otherwise break in front of them.
CLINGS_LEFT = set("?!,.;:។៕ៗ»")


def _khmer(text: str) -> bool:
    return any("\u1780" <= ch <= "\u17ff" for ch in text)


def mark(text: str, segmenter) -> str:
    """Insert ZWSP between Khmer words, and nowhere else.

    A break only ever goes *between two Khmer words*. Everything else —
    brackets, quotes, digits, Latin — stays welded to its neighbour, because
    the mark that runs along the caption stops on each piece, and stopping on
    a lone "(" is not reading.

    The segmenter returns those as separate tokens, so `sqrt(d_k)` came back as
    six of them and the mark stopped six times inside one symbol.
    """
    pieces: list[str] = []

    for token in segmenter.segment(text.replace(ZWSP, "")):
        piece = " " if token.isspace() else token
        breaks = (
            pieces
            and piece != " "
            and pieces[-1] != " "
            and piece not in CLINGS_LEFT
            and _khmer(piece)
            and _khmer(pieces[-1])
        )
        if breaks:
            pieces.append(ZWSP)
        pieces.append(piece)

    return "".join(pieces)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--segmenter", type=Path, default=DEFAULT_SEGMENTER)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()

    source = MAIN.read_text()
    segmenter = load_segmenter(args.segmenter)

    changed = 0

    def replace(match: re.Match) -> str:
        nonlocal changed
        before = match.group(1)
        after = mark(before, segmenter)
        if after != before:
            changed += 1
            words = after.replace(" ", ZWSP).split(ZWSP)
            print(f"  {len([w for w in words if w]):2d} words  "
                  f"{' | '.join(w for w in words if w)[:88]}")
        return f"'{after}'"

    updated = KHMER_LITERAL.sub(replace, source)

    if not args.write:
        print(f"\n{changed} caption(s) would change. Re-run with --write.")
        return 0

    MAIN.write_text(updated)
    print(f"\nmarked {changed} caption(s) in {MAIN.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
