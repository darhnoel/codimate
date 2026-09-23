#!/usr/bin/env python3
"""Mark Khmer word boundaries in this example's words. Authoring-time only.

    python python/examples/archimedes/segment.py            # report only
    python python/examples/archimedes/segment.py --write    # rewrite in place

Khmer does not separate words with spaces, so a line cannot run word by word
without knowing where its words are. Finding that out needs a dictionary and a
Viterbi search — far too much to carry into a render.

So it happens here, once, and the answer is written into the line itself as
U+200B ZERO WIDTH SPACE, the character Khmer already uses for a word boundary.
`main.chunks()` splits on it at render time. The video needs no segmenter, no
dictionary and no model: the text arrives already knowing where its own words
end. The segmenter stays in the lab; only its output ships.

ZWSP is invisible, so the lines still read exactly as written — and a line that
has never been through here still runs, one orthographic cluster at a time,
which is choppier but not broken.

The segmenter is vendored in a sibling repository rather than published, so
point at it if it is not where it usually lives:

    --segmenter ~/Developments/khmerime-lab/tools/dict-segment/vendor
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ZWSP = "​"
HERE = Path(__file__).resolve().parent
WORDS = HERE / "vocabulary.py"

DEFAULT = Path.home() / "Developments/khmerime-lab/tools/dict-segment/vendor"

# A quoted string holding Khmer. Khmer is the only non-ASCII prose in the file
# and it is always in double-quoted literals.
KHMER_LITERAL = re.compile(r'"([^"\n]*[ក-៿][^"\n]*)"')

# Khmer punctuation that belongs to the word before it: a full stop, the
# repetition sign, and so on. The rule below would otherwise break in front
# of them.
CLINGS_LEFT = set("?!,.;:។៕ៗ»")


def load_segmenter(vendor: Path):
    sys.path.insert(0, str(vendor))
    try:
        from khmer_segmenter import KhmerSegmenter  # type: ignore
    except ImportError as exc:
        raise SystemExit(
            f"no khmer_segmenter under {vendor}\n"
            "Pass --segmenter <path to the vendor directory>."
        ) from exc

    data = vendor / "khmer_segmenter" / "dictionary_data"
    return KhmerSegmenter(
        str(data / "khmer_dictionary_words.txt"),
        str(data / "khmer_word_frequencies.json"),
    )


def _khmer(text: str) -> bool:
    return any("ក" <= ch <= "៿" for ch in text)


def mark(text: str, segmenter) -> str:
    """Insert ZWSP between Khmer words, and nowhere else.

    A break only ever goes *between two Khmer words*. Everything else —
    brackets, digits, Latin, a `{placeholder}` — stays welded to its
    neighbour, because the mark that runs along the line stops on each piece,
    and stopping on a lone brace is not reading.
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
    parser.add_argument("--segmenter", type=Path, default=DEFAULT)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()

    source = WORDS.read_text()
    segmenter = load_segmenter(args.segmenter)
    changed = 0

    def replace(found: re.Match) -> str:
        nonlocal changed
        before = found.group(1)
        after = mark(before, segmenter)
        if after != before:
            changed += 1
            words = [w for w in after.replace(" ", ZWSP).split(ZWSP) if w]
            print(f"  {len(words):2d} words  {' | '.join(words)[:86]}")
        return f'"{after}"'

    updated = KHMER_LITERAL.sub(replace, source)
    if not args.write:
        print(f"\n{changed} line(s) would change. Re-run with --write.")
        return 0

    WORDS.write_text(updated)
    print(f"\nmarked {changed} line(s) in {WORDS.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
