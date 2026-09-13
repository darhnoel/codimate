"""The timeline panel: three segments, and the playhead sitting in one."""

import codimate as cm
from story import SEGMENTS, TOTAL
from theme import ACCENT, DIM, INK, LIVE, TRACK


def track_x(t):
    return TRACK.left + TRACK.w * (t / TOTAL)


def draw(scene, clock, active):
    start = 0.0
    for index, (name, duration) in enumerate(SEGMENTS):
        left, right = track_x(start), track_x(start + duration)
        here = index == active
        band = scene.group(("segment", index), at=((left + right) / 2, TRACK.y))
        band.rect("box", w=right - left - 4, h=TRACK.h).fill(ACCENT if here else DIM)
        band.text("name", name, size=22).fill(INK if here else "grey")
        band.text("span", f"{start:.1f}s", size=17,
                  at=cm.at(top=TRACK.h / 2 + 12)).fill("grey")
        start += duration

    scene.text("end", f"{TOTAL:.1f}s", size=17,
               at=cm.at(x=track_x(TOTAL), top=TRACK.bottom + 12)).fill("grey")

    # One group, so the stem and its reading can never drift apart.
    head = scene.group("playhead", at=(track_x(clock.t), TRACK.y))
    head.line("stem", start=(0, -TRACK.h / 2 - 12), end=(0, TRACK.h / 2 + 4),
              w=3.0).fill(LIVE)
    head.text("t", f"t = {clock.t:.2f}s", size=24,
              at=cm.at(bottom=-TRACK.h / 2 - 18)).fill(LIVE)
