"""Placement and Queries: a row of bars, a column of labels, a polygon and a
star from their corners, a box sized to its words, a dot eased along a path."""

import sys

import codimate as cm


def walk(values, emit):
    values.sort()
    emit("sort")


def view(frame):
    scene = cm.Scene()
    for slot, value in cm.row(frame.state, gap=30, size=90, within=cm.Slot(
            x=420, y=360, w=760, h=600)):
        bar = scene.group(("bar", value), slot)
        bar.rect("fill", h=value * 50, at=cm.at(bottom=0)).fill("#2997ff")
    for i, slot in enumerate(cm.column(3, gap=20, size=(220, 50), at=cm.at(x=1050))):
        scene.rect(("row", i), h=slot.h, w=slot.w, at=slot).fill("#1c1c1e")
        scene.text(("label", i), f"row {i}", size=22, at=(slot.x, slot.top + 25))
    scene.polygon("hex", cm.ngon(6, 50, at=(950, 600), turn=30)).fill("#30d158")
    scene.polygon("star", cm.star(5, 50, at=(1150, 600))).fill("#ffd60a")
    w, h = cm.measure("Bubble sort", 30)
    scene.rect("tag", w=w + 24, h=h + 12, at=(640, 60)).fill("#3a3a3c")
    scene.text("tag text", "Bubble sort", size=30, at=(640, 60))
    scene.circle("dot", r=8, at=(100 + 300 * cm.ease(0.3), 680)).fill("white")
    return scene


cm.explain(trace=cm.trace(walk, [4, 1, 3, 2]), view=view).export_web(sys.argv[1])
