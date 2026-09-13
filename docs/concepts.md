# How Codimate Thinks

Chapter 3 of [the guide](../README.md#more). By now you have built an animation
and know what you can draw. This chapter is about why the library is shaped the
way it is, and it is worth reading before you write anything substantial.

Codimate is fastest when you do not start by drawing frames.

Start with the concept:

```text
State → Algorithm → Trace → View → Scene → Timing → Video
```

```python
cm.explain(
    trace=merge_sort(cm.items([38, 27, 43, 3, 9, 82, 10, 15])),
    view=merge_sort_view,
    motion=[cm.Rule("*", position="lift_carry_drop")],
    timing=cm.Timing(default=0.5, events={"merge": 0.9}),
).render("merge_sort.mp4")
```

Change the data or the algorithm and the video regenerates from the concept,
instead of being manually re-edited. That is the whole point: the video cannot
drift from the code it describes, because a human never decides what the
picture shows.

## Why This Works

The One Law is still:

```text
f(t) → Scene
```

The authoring model explains where `f` comes from:

```text
algorithm:  State → Trace
view:       State + Trace Event → Scene
motion:     which items travel, and along what path
timing:     Trace Event → duration
```

Nothing you write knows what a frame is. The Engine composes those four pieces
into a pure function of time, and a frame is computed from scratch at `t` —
never played forward, never accumulated. Any moment can be rendered in any
order, and twice the same way.

## Motion Is Derived, Not Authored

This is the idea that makes the Python surface small.

In a keyframe tool you say *"move this bar from here to there over 0.6s."* In
Codimate you never say that. You describe two moments, and the Engine works out
what changed:

```text
scene at event i        scene at event i+1
     3 at x=420  ─────▶  3 at x=550     "3 travelled"
     1 at x=550  ─────▶  1 at x=420     "1 travelled"
     4 at x=680  ─────▶  4 at x=680     "4 stayed"
```

Whatever differs becomes a tween — position, size, colour, opacity. Whatever
matches holds. A shape present in only one of the two moments fades.

**The cost of this trade is identity.** The Engine can only pair shapes it can
recognise across two moments, and the only thing it has to go on is the name
you gave them.

## Identity: The One Decision

Every shape carries a name — the identity of the thing you are talking about,
not an arbitrary id.

```python
scene.group(item.id, slot)              # keyed by the thing
scene.group(("cell", row, col), slot)   # keyed by the place
```

| Keyed by | The Engine sees | The viewer sees |
|---|---|---|
| **the thing** | item 3 moved from x=420 to x=550 | the bar **slides across** |
| **the place** | slot 0 stayed at x=420 and got shorter | bars **morph in place** |

Both are correct for different explanations. Sorting, routing, and anything
where a *thing* travels wants identity keyed to the thing. A heatmap, a
progress bar, or a grid of cells wants identity keyed to the position, because
the cell is the thing and the value is just its state.

The Engine cannot check this. A wrong choice is not an error, it is a
confusing video. When an explanation reads badly, this is the first thing to
look at.

A corollary: **names must be stable.** If a name changes between moments, the
Engine sees one thing leave and a different thing arrive, and fades them
instead of moving anything.

`cm.items()` exists for the first case. A bare `3` cannot be an identity —
two 3s in a list are two different bars, and only a wrapper can say so. An
Item compares by value, so the algorithm stays ordinary Python, and is equal
by id, so it survives the snapshot taken at every event. The second case needs
nothing: `("cell", row, col)` is already unique and already stable.

## Groups: A Thing Made Of Several Shapes

A bar is a rectangle *and* a label. If those are two independent names, they
are two independent things, and a motion rule can pull them apart — the bar
arcs over while the label slides underneath.

A Group makes that unrepresentable:

```python
bar = scene.group(item.id, slot)
bar.rect("bar", h=item.value * 70, at=cm.at(bottom=0))
bar.text("label", item.value, at=cm.at(top=20))
```

The Engine sees `3/bar` and `3/label` — one name, two shapes. Whatever the
group does, both do.

**Inside a Group, `0` is the Group's own point.** Nothing is inherited from the
parent and nothing is decided implicitly; `bottom=0` and `top=20` say exactly
where they mean, and the same rule holds at any depth of nesting.

## The Four Pieces

### Algorithm — State → Trace

Write the real logic. Call `emit()` after you change your data; the Trace
records a snapshot of the result.

```python
@cm.trace()
def bubble_sort(values):
    for i in range(len(values)):
        for j in range(len(values) - 1 - i):
            cm.emit("compare", items=[values[j], values[j + 1]])
            if values[j] > values[j + 1]:
                values[j], values[j + 1] = values[j + 1], values[j]
                cm.emit("swap", items=[values[j], values[j + 1]])
```

The View does not invent concept logic. If the picture needs to know something,
the Algorithm should have emitted it.

### View — State + Trace Event → Scene

One moment, one picture. Pure: no timing, no memory of the moment before, no
knowledge of frames.

```python
def bars(frame):
    scene = cm.Scene()
    active = frame.items()
    for slot, item in cm.row(frame.state, gap=40):
        bar = scene.group(item.id, slot)
        bar.rect("bar", h=item.value * 70, at=cm.at(bottom=0)) \
           .fill("orange" if item in active else "blue")
        bar.text("label", item.value, at=cm.at(top=20))
    return scene
```

`cm.row()` divides the canvas into one **Slot** per item. A Slot is a place,
not a shape — nothing draws it, and it holds no identity. Slots are where
things sit; Items are what things are. Keeping those separate is what lets a
bar move from one Slot to another without becoming a different bar.

The View runs **once per Trace Event**, not once per frame. A 60-second video
is 1800 frames but perhaps 50 events, so the View can be as slow as it likes.
That asymmetry is the reason Python is the authoring language at all.

### Motion — which items travel, and how

Timeless. Patterns matched against shape names, first match wins. A shape in
a group is named `group/child`, so `"3/*"` targets one group.

```python
motion=[
    cm.Rule("*", position="lift_carry_drop", clearance=90),
]
```

Motion contains no durations. It also cannot make something move that did not
move — a rule only shapes the path of a shape whose position actually changed.

### Timing — Trace Event → duration

Pacing lives here and nowhere else. If a duration appears in your algorithm or
your view, it is in the wrong place.

```python
timing=cm.Timing(
    default=0.55,
    events={"swap": 0.9, "done": 0.6},
    opening=0.8,
    final_hold=1.2,
)
```

Durations are keyed by event **name**, which is why naming events well pays off
twice.

## Good Trace Events

Good events come from the concept:

- `compare(a, b)`
- `swap(a, b)`
- `choose_pivot(index)`
- `compute_cell(row, col)`
- `fire_to_hidden(hidden)`

Weak events come from video editing:

- `show_box_at_frame_12`
- `move_thing_for_two_seconds`
- `make_it_blue_now`
- `keyframe_3`

If the event would still make sense in a *written* explanation of the concept,
it probably belongs in the Trace.

## Build A New Explanation

1. Name the concept.
2. Write the Algorithm as real logic.
3. `emit()` the moments worth showing, and name them for what they mean.
4. Decide what each thing on screen **is** — that decides your names, and
   whether you need `cm.items()`.
5. Write the View for one moment.
6. Add Motion only where a straight line is not enough.
7. Put durations in Timing.

## Guardrails

- The final animation still obeys `f(t) → Scene`.
- The Algorithm produces the Trace; the View does not invent concept logic.
- The View is pure — same frame in, same Scene out, every time.
- Motion has no duration. Timing owns duration.
- Names are stable and mean something; a thing made of parts is one Group.
- Renderer and export concerns stay in the Engine, out of the authoring model.

## Under The Hood

The Rust crates are the Engine, not a second way to author explanations. The
split is per-event versus per-frame — see
[ADR 0008](./adr/0008-python-authoring-surface.md) for why, and
[CONTEXT.md](../CONTEXT.md) for the vocabulary.
