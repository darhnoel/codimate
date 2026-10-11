# Recipes

Use these small changes after completing [First animation](tutorial.md).
Unless marked otherwise, the snippets assume you already have `import codimate
as cm`, an algorithm, and a `view(frame)` function.

## Highlight the item involved in an event

Pass information with the event after updating your state:

```python
emit("counted", fruit=fruit)
```

Inside the view, check for the opening picture before reading event data:

```python
active = frame.event.data.get("fruit") if frame.event else None
color = "orange" if fruit == active else "blue"
```

For algorithms using `cm.items`, pass an `items` list instead:

```python
emit("compare", items=[values[j], values[j + 1]])
```

The view can then use `active = frame.items()` and `item in active`.
Both event data and state are snapshots; create your item identities before
recording the trace.

## Give an event more time

Pass this timing object to `cm.explain`:

```python
timing = cm.Timing(
    default=0.4,
    events={"compare": 0.6, "swap": 0.9},
    opening=0.8,
    final_hold=2.0,
)
```

The keys must match names passed to `emit()`. `default` applies to other
events. `opening` and `final_hold` let viewers read the first and last pictures.

## Choose a motion path

Motion changes the route of shapes whose positions already differ between scenes.
Pass a list of rules to `cm.explain(motion=...)`:

```python
motion = [cm.Rule("*", position="lift_carry_drop", clearance=90)]
```

| Path | Use it for |
| --- | --- |
| `straight` | Distinct steps that ease in and out; the default. |
| `linear` | Continuously sampled motion, such as an orbit. |
| `fall` | A falling trajectory. |
| `lift_carry_drop` | Items traveling over neighboring items. |

Rules match shape names; the first matching rule wins. `"*"` matches all shapes.
A rule cannot move a shape whose position is unchanged.

## Add a formula

Install Typst and confirm the executable is available:

```bash
typst --version
```

On macOS with Homebrew, install it with `brew install typst`.
Then add a formula to your view:

```python
scene.formula("force", r"F = ma", size=36, at=(640, 100)).fill("white")
```

The `r` prefix preserves LaTeX backslashes. For example:

```python
scene.formula("ratio", r"\frac{917}{1000} = 0.917", size=36,
              at=(640, 100)).fill("white")
```

Formulas use a LaTeX subset. If a command is rejected, simplify the expression
and consult [the reference](reference.md#drawing).

## Add narration or sound

Supply an existing audio file with an event:

```python
emit("explain", sound="audio/explain.mp3")
```

`render()` starts the clip at that event's start and mixes it into the video.
The path is relative to the working folder. Keep audio files available when
rendering; a missing file causes an error.

Audio does not automatically set event duration. Assign enough time for the
recording through `Timing(events={"explain": ...})`.
The reference describes [audio inspection and mixing](reference.md#looking-at-what-you-made).

## Frame the camera around an object

After drawing the named object in your view:

```python
scene.focus("planet", pad=40, least=240)
```

Codimate fits the camera around it. `pad` adds breathing room; `least` prevents
very small objects from filling the whole picture.
For text that should stay fixed while the camera moves, use an overlay:

```python
caption = scene.overlay()
caption.text("title", "Orbit", size=32, at=(640, 60)).fill("white")
```

## Inspect a moment before exporting the whole video

Keep the result of `cm.explain` in a variable called `explanation`, as in the
tutorial. Temporarily replace its `render()` call with:

```python
explanation.frame_at(1.5, "results/check.png")
```

This writes one PNG at 1.5 seconds. To find a useful time, inspect the event schedule:

```python
print(explanation.timeline())
```

For interactive inspection, restore the script's `render()` call and run:

```bash
python -m codimate.preview counter.py
```

The previewer captures the explanation instead of exporting the full video.
See [Previewer reference](reference.md#the-previewer) for controls and options.

## Find a larger worked example

The [examples repository](https://github.com/darhnoel/codimate-examples) contains
complete source and a README for each explanation. Start with a subject whose
structure matches yours:

| You are making | Example |
| --- | --- |
| A sorting explanation | [Bubble sort](https://github.com/darhnoel/codimate-examples/tree/main/bubble_sort) |
| A floating-object explanation | [Archimedes](https://github.com/darhnoel/codimate-examples/tree/main/archimedes) |
| An orbit diagram | [Orbits](https://github.com/darhnoel/codimate-examples/tree/main/orbits) |
| A field and equation explanation | [Maxwell](https://github.com/darhnoel/codimate-examples/tree/main/maxwell) |

Copy the organization, not unexplained constants. Your algorithm and geometry
still need to describe your own subject correctly.
