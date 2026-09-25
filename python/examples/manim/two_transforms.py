"""Manim's `TwoTransforms`, written in Codimate.

    python python/examples/manim/two_transforms.py

Manim has two calls that look the same on screen:

    self.play(Transform(a, b))              # `a` stays, wearing b's shape
    self.play(ReplacementTransform(a, b))   # `b` takes a's place in the scene

The difference is bookkeeping. After `Transform(a, b)` the thing on screen is
still `a`, so the next call is `Transform(a, c)` and the fade is `FadeOut(a)`.
After `ReplacementTransform(a, b)` the thing on screen is `b`, so it is
`ReplacementTransform(b, c)` and `FadeOut(c)`. Pick the wrong one and nothing
looks wrong — you just have a handle that no longer points at the picture.

**Neither exists here, because the name is the identity.** There is no registry
of objects to keep in step with the drawing, so there is nothing to get out of
step. Both of Manim's methods are this:

    scene.polygon("shape", outline(whatever_it_is_now))

What Codimate *does* make you choose is on the other side of the picture: keep
the name and it morphs, change the name and it cross-fades. That is a decision
about what the thing *is*, and this example shows both at once.
"""

import codimate as cm

import shapes

cm.canvas(1280, 720)

INK, PAPER, FAINT = "#58c4dd", "#e8eef7", "#8b96a8"
LEFT, RIGHT = (400.0, 360.0), (880.0, 360.0)
WALK = {"circle": shapes.circle, "square": shapes.square,
        "triangle": shapes.triangle}


def two_transforms(state, emit):
    for shape in ("square", "triangle"):     # Transform, then Transform again
        state["shape"] = shape
        state["beat"] += 1
        emit("transform")
    state["shape"] = None                    # FadeOut
    emit("gone")


def view(frame):
    scene = cm.Scene()
    state = frame.state
    shape = state["shape"]

    if shape:
        # One name the whole way through. Consecutive Scenes share it, so the
        # Engine has something to interpolate and the outline morphs.
        scene.polygon("kept", shapes.outline(WALK[shape], centre=LEFT),
                      closed=False) \
             .fill(INK, edge=INK, edge_w=4).on(opacity=0.7)

        # A new name every beat. Nothing is shared, so each one enters and the
        # one before it leaves — which is a cross-fade, not a morph.
        scene.polygon(("fresh", state["beat"]),
                      shapes.outline(WALK[shape], centre=RIGHT),
                      closed=False) \
             .fill(INK, edge=INK, edge_w=4).on(opacity=0.7)

    scene.text("kept_label", "one name — it morphs", size=26,
               at=cm.at(x=LEFT[0], top=590)).fill(PAPER)
    scene.text("fresh_label", "a new name each beat — it cross-fades", size=26,
               at=cm.at(x=RIGHT[0], top=590)).fill(FAINT)
    return scene


cm.explain(
    trace=cm.trace(two_transforms, {"shape": "circle", "beat": 0}),
    view=view,
    timing=cm.Timing(default=1.3, events={"gone": 1.0},
                     opening=0.8, final_hold=1.0),
).render("results/manim_two_transforms.mp4", fps=60, scale=1.5)

print("wrote results/manim_two_transforms.mp4")
