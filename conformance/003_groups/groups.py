"""Bars standing on Slots, each a Group of a bar and its label, and a nested
Group (a card with a badge) that moves as one thing between moments."""

import sys

import codimate as cm


def walk(state, emit):
    state["values"] = [3, 5, 2]
    state["card"] = 900
    emit("grow")


def view(frame):
    scene = cm.Scene()
    for i, value in enumerate(frame.state["values"]):
        slot = cm.Slot(x=200 + i * 160, y=560, w=120, h=120, anchor="bottom")
        bar = scene.group(("bar", i), slot)
        bar.rect("fill", h=value * 60, at=cm.at(bottom=0)).fill("#2997ff")
        bar.text("value", value, size=24, at=cm.at(top=12))
    card = scene.group("card", at=(frame.state["card"], 300), w=240)
    card.rect("back", h=140).fill("#1c1c1e", edge="#3a3a3c", edge_w=2)
    badge = card.group("badge", at=(80, -50))
    badge.circle("dot", r=18).fill("#ff453a")
    badge.text("n", "!", size=20)
    return scene


start = {"values": [1, 2, 3], "card": 700}
cm.explain(trace=cm.trace(walk, start), view=view).export_web(sys.argv[1])
