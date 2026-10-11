# First animation

Build a chart that grows as a Python loop updates its data.
By the end, you will know where to put your logic, drawing, and timing.

**Before you start:** install Codimate with `python -m pip install codimate`.
Use any working folder; a repository checkout is unnecessary.

## 1. Record a few changes

Start a file named `counter.py`. This function is ordinary Python:

```python
def count(state, emit):
    for value in range(1, 6):
        state["count"] = value
        emit("counted")
```

`state` is a dictionary containing the current count. `emit` is a function
Codimate passes to your algorithm. Calling it saves a moment worth showing.

**Change the state first, then call `emit()`.** Otherwise the saved moment
contains the old value.

The event name `"counted"` describes what happened. It is separate from the
names of shapes you will draw.

## 2. Draw one saved moment

Add a function called `view`:

```python
def view(frame):
    scene = cm.Scene()
    value = frame.state["count"]
    scene.rect("bar", w=160, h=max(value * 70, 1),
               at=cm.at(x=640, bottom=550)).fill("blue")
    scene.text("label", f"Count: {value}", size=36,
               at=(640, 610)).fill("white")
    return scene
```

A `Scene` is the picture for this moment. `frame.state` contains a saved copy
of your data, not the dictionary after the whole loop has finished.

| Line | Meaning |
| --- | --- |
| `"bar"` | Keep this shape's identity across moments. |
| `w=160` | Make the bar 160 coordinate units wide. |
| `h=max(value * 70, 1)` | Map the count to height; keep zero visible as a thin bar. |
| `cm.at(x=640, bottom=550)` | Center it horizontally at 640 and keep its bottom at 550. |
| `.fill("blue")` | Color the rectangle blue. |
| `scene.text(...)` | Place a label below the bar. |

The default canvas is 1280 × 720. Its top-left corner is `(0, 0)`;
x increases to the right and y increases downward.

You draw the starting picture and the picture after each event. Codimate
works out the pictures between them. You do not write a frame-by-frame loop.

## 3. Assemble and run the complete program

Replace `counter.py` with this complete version. The first two functions are
the ones above; the final lines connect them and set the pace.

```python
import codimate as cm


def count(state, emit):
    for value in range(1, 6):
        state["count"] = value
        emit("counted")


def view(frame):
    scene = cm.Scene()
    value = frame.state["count"]
    scene.rect("bar", w=160, h=max(value * 70, 1),
               at=cm.at(x=640, bottom=550)).fill("blue")
    scene.text("label", f"Count: {value}", size=36,
               at=(640, 610)).fill("white")
    return scene


explanation = cm.explain(
    trace=cm.trace(count, {"count": 0}),
    view=view,
    timing=cm.Timing(default=0.5, opening=0.3, final_hold=1.0),
)
explanation.render("results/counter.mp4")
```

```bash
python counter.py
```

**Expected result:** `results/counter.mp4` contains a blue bar growing from
0 through 5, with a changing count label. The file is relative to the folder
where you ran the command. No terminal output is required for success.

If the command fails, check [Troubleshooting](troubleshooting.md).

## 4. Understand the data flow

```text
count() changes the data
          ↓ emit("counted")
trace saves a copy of the data
          ↓
view(frame) draws that saved moment
          ↓
Codimate animates the changes between pictures
          ↓
results/counter.mp4
```

`cm.trace(count, {"count": 0})` runs `count` once. It records the initial count
and five events. Your view is then called for the initial state and each event.

The loop does not pause for half a second after each count. It runs normally.
`Timing(default=0.5)` gives each event half a second **in the video**.
`opening` holds the initial picture; `final_hold` holds the last picture.

## 5. Make one change at a time

Try these edits, then run `python counter.py` again:

| Edit | Expected change |
| --- | --- |
| Change `"blue"` to `"orange"` | The bar changes color. |
| Change `value * 70` to `value * 40` | The chart is shorter. |
| Change `default=0.5` to `default=1.0` | Each count takes longer. |
| Change `final_hold=1.0` to `final_hold=3.0` | The final count stays visible longer. |

Keep `"bar"` as the shape name. A name identifies the object; it should not
include the count or change whenever the object's value changes.

## 6. Extend it to two bars

Once the single bar makes sense, replace the file with this version.
It alternates between two counters and highlights the one just updated.

```python
import codimate as cm

FRUITS = ("apples", "oranges")


def count(state, emit):
    for step in range(8):
        fruit = FRUITS[step % 2]
        state[fruit] += 1
        emit("counted", fruit=fruit)


def view(frame):
    scene = cm.Scene()
    active = frame.event.data["fruit"] if frame.event else None
    for slot, fruit in cm.row(FRUITS, gap=100, size=160,
                              at=cm.at(y=550)):
        bar = scene.group(fruit, slot)
        value = frame.state[fruit]
        color = "orange" if fruit == active else "blue"
        bar.rect("body", h=max(value * 70, 1),
                 at=cm.at(bottom=0)).fill(color)
        bar.text("label", f"{fruit}: {value}", size=28,
                 at=cm.at(top=20)).fill("white")
    return scene


explanation = cm.explain(
    trace=cm.trace(count, {"apples": 0, "oranges": 0}),
    view=view,
    timing=cm.Timing(default=0.5, opening=0.3, final_hold=1.0),
)
explanation.render("results/fruit.mp4")
```

**Expected result:** two labeled bars grow to 4. The most recently updated
bar is orange. The output is now `results/fruit.mp4`.

Three new tools do the work:

- `emit("counted", fruit=fruit)` records extra event data for highlighting.
- `cm.row(...)` supplies one place, called a **slot**, for each bar.
- `scene.group(fruit, slot)` keeps each bar and its label together.

Inside a group, coordinates are local. `bottom=0` puts the bar on the group's
anchor point; `top=20` puts the label's top 20 units below that point.

The opening picture has no event, so `frame.event` is `None`. That is why the
highlighting line checks it before reading event data.

## Next steps

Read [How it works](concepts.md) to learn why names decide what moves.
Use [Drawing and layout](drawing.md) to place your own diagrams, or jump to
[a recipe](recipes.md) for a specific task.
