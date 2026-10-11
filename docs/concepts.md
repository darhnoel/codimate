# How it works

Codimate connects **saved data** to **pictures**. It matches shapes between
pictures and animates whatever changed.

Read [First animation](tutorial.md) first if you have not run a program yet.
This page explains the decisions you make when building a larger explanation.

## The four pieces

| Piece | Your job | Example |
| --- | --- | --- |
| Algorithm | Calculate what happens and record important moments. | Increment a count, then `emit("counted")`. |
| View | Draw one saved moment as a `Scene`. | Turn the count into a bar height. |
| Motion | Choose the path for objects whose positions change. | Let swapped bars travel over each other. |
| Timing | Decide how long events take in the video. | Give `"swap"` 0.9 seconds. |

Motion is optional. The default path works for many explanations.

```python
explanation = cm.explain(
    trace=cm.trace(algorithm, initial_state),
    view=view,
    timing=cm.Timing(default=0.5),
)
```

Here `algorithm`, `initial_state`, and `view` refer to your own function,
data, and drawing function. This is a connection pattern, not a standalone program.

## State, events, and scenes are different

**State** is the data: a count, a list, a grid, or a simulation's values.
An **event** names something that happened and includes a saved state.
A **scene** is the drawing your view creates from that state.

```python
def count(state, emit):
    state["count"] += 1
    emit("counted", amount=1)
```

For this event, the view can read:

```python
value = frame.state["count"]       # the count after the increment
amount = frame.event.data["amount"]  # extra information about this event
finished = frame.is_("done")       # whether this event is named "done"
```

The first view call uses the initial state and has no event. Check
`frame.event` before accessing its data, or use `frame.is_()` and `frame.items()`,
which handle the opening picture for you.

Choose event names that describe your subject: `compare`, `swap`, `measure`,
`add_sample`. The name also lets you assign a duration in `Timing`.

## Names decide what moves

A shape name answers: **is this the same object as in the previous picture?**

| Between two scenes | Codimate's behavior |
| --- | --- |
| Same name, different position | Move the shape. |
| Same name, different size or color | Animate that change. |
| Name only in the new scene | Fade the shape in. |
| Name only in the old scene | Fade the shape out. |

Keep names stable. Use `"bar"`, not `f"bar-{count}"`, for one bar whose height
changes. Names must also be unique within a scene.

## Should a name follow an item or a position?

For a sorting animation, the **item** moves. For a heatmap, the **cell** stays
put while its value changes. Name the shapes accordingly.

| Subject | Name follows | Typical name |
| --- | --- | --- |
| Sorting bars | Individual items | `item.id` |
| Moving planets | Individual planets | `"earth"` |
| Heatmap | Fixed cells | `("cell", row, col)` |
| Growing tally | Categories | `"heads"`, `"tails"` |

`cm.items()` gives values stable identities. Two equal values still represent
two separate objects:

```python
values = cm.items([3, 1, 3])
# Each item has .value and a different .id.
```

Items compare by value, so ordinary sorting comparisons still work.
Create the items before recording the trace, not again inside the view.

## Example: two items exchanging places

Save this complete program as `swap.py` and run `python swap.py`.
The orange and blue circles exchange places, carrying their labels with them.

```python
import codimate as cm


def swap(values, emit):
    values.reverse()
    emit("swapped")


def view(frame):
    scene = cm.Scene()
    for slot, item in cm.row(frame.state, gap=180, size=120,
                             at=cm.at(y=360)):
        group = scene.group(item.id, slot)
        group.circle("body", r=50).fill(
            "orange" if item.value == "A" else "blue"
        )
        group.text("label", item.value, size=32).fill("white")
    return scene


cm.explain(
    trace=cm.trace(swap, cm.items(["A", "B"])),
    view=view,
    timing=cm.Timing(default=1.0, opening=0.8, final_hold=1.0),
).render("results/swap.mp4")
```

Now try naming groups by their slot index instead of `item.id`. Each circle
stays in its position while its color and label change. That is appropriate
for a fixed grid, but it tells a different story from moving items.

## Groups and slots

A **slot** is a place to put something. It does not draw anything.
A **group** gives several shapes a shared position and name prefix.

```python
bar = scene.group(item.id, slot)
bar.rect("body", h=item.value * 70, at=cm.at(bottom=0)).fill("blue")
bar.text("label", item.value, size=28, at=cm.at(top=20))
```

The bar and label share an item identity, so they travel together. Each child's
position is relative to the group's point, rather than the canvas origin.
See [Drawing and layout](drawing.md#use-local-coordinates-inside-a-group).

## What Codimate calculates, and what you calculate

You calculate the subject: a sorting decision, a planet's position, or a
floating object's equilibrium. Your view maps those results to shapes.
Codimate calculates the animation between the scenes you supply.

It cannot verify your physics or fix a misleading mapping from data to shapes.
Keep the view free of changes to the model: the same saved moment should
produce the same picture each time.

For continuous motion, record enough sample states to describe the path and use
`linear` motion to avoid repeatedly slowing down between samples.
[Motion recipes](recipes.md#choose-a-motion-path).

## Further reading

Use [Reference](reference.md) for signatures and defaults.
Engine contributors can read [the architecture decisions](adr/) for the
implementation model; animation authors do not need them to get started.
