"""Motion and timing: bars swapped by a lift-carry-drop path, a card on an
arc, labels that spring in turn, and events with their own durations."""

import sys

import codimate as cm


def sort(items, emit):
    items[0], items[2] = items[2], items[0]
    emit("swap", items=[items[0], items[2]])
    items.reverse()
    emit("flip")


def view(frame):
    scene = cm.Scene()
    for slot, item in cm.row(frame.state, gap=40, size=120):
        bar = scene.group(("bar", item.id), slot)
        bar.rect("fill", h=item.value * 40, at=cm.at(bottom=0)).fill("#2997ff")
        bar.text("label", item.value, size=24, at=cm.at(top=12))
    swapped = frame.is_("swap")
    scene.rect("card", w=160, h=90, at=(1100 if swapped else 180, 120)).fill("#ff9f0a")
    return scene


motion = [
    cm.Rule(("bar", "*", "fill"), position="lift_carry_drop", clearance=90),
    cm.Rule("card", position="arc", bend=-0.4, ease="out"),
    cm.Rule(("bar", "*", "label"), ease="spring", stagger=0.1, order="centre",
            bounciness=0.5),
]
timing = cm.Timing(default=0.8, events={"swap": 1.5}, opening=0.4, final_hold=1.0)
cm.explain(trace=cm.trace(sort, cm.items([1, 4, 2, 5])), view=view, motion=motion,
           timing=timing).export_web(sys.argv[1])
