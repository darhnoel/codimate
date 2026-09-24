#!/usr/bin/env python3
"""Lay the narration onto the rendered Khmer video. Authoring-time only.

    python python/examples/archimedes/mix.py

Reads `audio/cues.json`, which `main.py` writes as it renders: the cut it
rendered, and one entry per caption with the moment its recording should
start. Those offsets come from walking the same events the renderer walks, so
the sound cannot disagree with the picture.

Codimate has no audio channel yet (ADR 0007 is unbuilt), so this is a separate
ffmpeg pass rather than part of the render. The video stream is copied, never
re-encoded — only the audio is added.
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
AUDIO = HERE / "audio"
ROOT = HERE.parents[2]


def main() -> int:
    cues_file = AUDIO / "cues.json"
    if not cues_file.exists():
        raise SystemExit("no audio/cues.json — render `main.py km` first")
    written = json.loads(cues_file.read_text())
    cues = written["cues"]

    # Which cut these cues were walked for. There is more than one — with the
    # subtitle and without it — and laying a voice onto the wrong one would
    # sound right while showing the other film.
    video = ROOT / written["video"]
    if not video.exists():
        raise SystemExit(f"no {video} — render it first")

    # Written back over the render, so there is one file to watch rather than
    # a silent one and a narrated one side by side with the same thumbnail.
    # ffmpeg cannot read and write the same path, hence the temporary.
    out = video.with_name(f".{video.stem}-narrated.tmp.mp4")

    command = ["ffmpeg", "-y", "-v", "error", "-i", str(video)]
    for cue in cues:
        command += ["-i", str(AUDIO / cue["file"])]

    # Delay each clip to its cue, then sum them. `normalize=0` keeps every
    # clip at its recorded level; amix otherwise divides by the input count
    # and the narration fades as more clips are added.
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
        # No `-shortest`: the picture runs past the last clip by design — the
        # final hold — and truncating to the audio would cut the ending.
        str(out),
    ]
    subprocess.run(command, check=True)
    out.replace(video)

    print(f"wrote {video.relative_to(ROOT)} — {len(cues)} clips of narration")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
