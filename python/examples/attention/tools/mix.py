#!/usr/bin/env python3
"""Lay the narration onto the rendered video.

    python tools/mix.py

Reads `audio/cues.json`, which `main.py` writes as it renders: one entry per
caption, with the moment its recording should start. Those offsets come from
walking the same events the renderer walks, so the sound cannot disagree with
the picture.

Codimate has no audio channel yet (ADR 0007 is unbuilt), so this is a separate
ffmpeg pass rather than part of the render. The video stream is copied, never
re-encoded — only the audio is added.
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
AUDIO = HERE.parent / "audio"
ROOT = HERE.parents[3]
VIDEO = ROOT / "results" / "attention.mp4"

# Written back over the render, so there is one file to watch rather than a
# silent one and a narrated one sitting side by side with the same thumbnail.
# ffmpeg cannot read and write the same path, hence the temporary.
OUT = ROOT / "results" / ".attention-narrated.tmp.mp4"


def main() -> int:
    cues_file = AUDIO / "cues.json"
    if not cues_file.exists():
        raise SystemExit("no audio/cues.json — render main.py first")
    cues = json.loads(cues_file.read_text())
    if not VIDEO.exists():
        raise SystemExit(f"no {VIDEO} — render main.py first")

    command = ["ffmpeg", "-y", "-v", "error", "-i", str(VIDEO)]
    for cue in cues:
        command += ["-i", str(AUDIO / cue["file"])]

    # Delay each clip to its cue, then sum them. `normalize=0` keeps every
    # clip at its recorded level; amix otherwise divides by the input count and
    # the narration fades as more clips are added.
    delays = "".join(
        f"[{i}:a]adelay={round(cue['start'] * 1000)}:all=1[d{i}];"
        for i, cue in enumerate(cues, start=1)
    )
    mix = "".join(f"[d{i}]" for i in range(1, len(cues) + 1))
    graph = f"{delays}{mix}amix=inputs={len(cues)}:normalize=0[a]"

    command += [
        "-filter_complex", graph,
        "-map", "0:v", "-map", "[a]",
        "-c:v", "copy", "-c:a", "aac", "-b:a", "160k",
        # No `-shortest`: the picture runs past the last clip by design (the
        # final hold), and truncating to the audio would cut the ending.
        str(OUT),
    ]
    subprocess.run(command, check=True)
    OUT.replace(VIDEO)

    print(f"wrote {VIDEO.relative_to(ROOT)} — {len(cues)} clips of narration")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
