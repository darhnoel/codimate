"""Bubble sort, end to end.

    python python/examples/bubble_sort/main.py

Four pieces: the algorithm, the view, the motion, the timing.
Nothing here knows what a frame is.
"""

import codimate as cm


# --- the algorithm: ordinary Python, with emit() where something happens ----


@cm.trace()
def bubble_sort(values):
    n = len(values)
    for i in range(n):
        for j in range(n - 1 - i):
            cm.emit("compare", items=[values[j], values[j + 1]])
            if values[j] > values[j + 1]:
                values[j], values[j + 1] = values[j + 1], values[j]
                cm.emit("swap", items=[values[j], values[j + 1]])
    cm.emit("done")


# --- the view: what one moment looks like ----------------------------------


def bars(frame):
    scene = cm.Scene()
    active = frame.items()
    done = frame.is_("done")

    scene.text("title", "Bubble Sort", x=cm.width() / 2, y=90, size=40, color="grey")

    for slot, item in cm.row(frame.state, gap=40):
        # The name is the ITEM, not the position — so a bar travels when it
        # moves. Naming it after the position would make it morph in place.
        bar = scene.group(item.id, slot)
        bar.rect(
            "bar",
            h=item.value * 70,
            bottom=0,
            color="green" if done else "orange" if item in active else "blue",
        )
        bar.text("label", item.value, top=20, size=32)

    return scene


# --- motion and timing ------------------------------------------------------

cm.explain(
    trace=bubble_sort(cm.items([3, 1, 4, 2])),
    view=bars,
    motion=[cm.Rule("*", position="lift_carry_drop", clearance=90)],
    timing=cm.Timing(default=0.55, events={"swap": 0.9, "done": 0.6}),
).render("results/bubble_sort.mp4")

print("wrote results/bubble_sort.mp4")
