# Dharmachakra — rotation without a rotate

```bash
.venv/bin/python python/examples/dharma_wheel/main.py
```

The wheel of the Noble Eightfold Path, turning. The spoke reaching the top
brightens, and its factor is named below in Khmer.

> **Please check the Khmer.** The eight factors are given in the Pali terms as
> written in Khmer script (`សម្មាទិដ្ឋិ`, `សម្មាសង្កប្បៈ`, …), with the English
> beside each in a comment in `main.py`. They are religious terms and I am not
> a Khmer speaker — a wrong vowel sign would be easy to miss and worth fixing.

## What it teaches

*The step size.* **Codimate has no rotation, and does not need one.** There is no `rotate=`
anywhere in the API. The trace says where the spokes are every 15 degrees:

```python
@cm.trace()
def turn(wheel):
    for step in range(SPOKES * PER_SPOKE):
        wheel.angle = (step + 1) * STEP
        cm.emit("turn")
```

The view draws each spoke as a line at its current angle, and the Engine works
out every frame in between — the same machinery that slides a bar from one slot
to another. **Rotation is just position over time.** Anything you can describe
as a position at each moment, Codimate can animate.

**Two things make it turn smoothly**, and only one of them is obvious.

*The `linear` motion path.* Every path eases in and out by default, which is
right when each event is a distinct step and wrong for something mid-journey at
every event. Measured on this wheel, the eased version's frame-to-frame motion
swings over a **6.6x** range — surge, stall, surge — while `linear` holds a
**1.2x** spread. That is the judder, and no step size fixes it:

```python
motion=[cm.Rule("*", position="linear")]
```

**Why 15 degrees.** The Engine interpolates a line's endpoints in a straight
line, so between two samples a spoke tip travels a chord rather than an arc.
That is a real approximation, and the step size decides whether it shows:

| step | tip dips by |
|---|---|
| 45° | 7.6% — 13.5px, a visible wobble |
| 30° | 3.4% — 6.1px, still noticeable |
| **15°** | **0.9% — 1.5px, invisible** |
| 10° | 0.4% — 0.7px |

15° also divides evenly into the 45° between spokes, so a spoke lands exactly
on the marker rather than near it.

**Two tricks draw the whole thing**, out of nothing but filled circles and
thick lines — see [`wheel.py`](wheel.py):

*Rings come from discs.* A circle can only be filled, so a banded ring with a
dark edge on both sides is four stacked discs: dark, gold, dark, background.
The rim and the hub are both built that way.

*A thick line is a rotated rectangle.* `line(start, end, w=26)` strokes a path,
so a short span at a large width draws a block square to the spoke at any
angle. That is how the spokes taper — a broad shaft from the hub, a narrower
one out to the rim — and how the diamond ornament sits where they meet. No new
shape kind was needed for either.

**What is still out of reach.** The lotus finials are three circles apiece
rather than a drawn petal, and the hub is a three-lobed approximation of a
triskelion. Curves that are not circles need arbitrary paths, which the
Authoring Surface does not expose.

## Try changing

| Change | What happens |
|---|---|
| `STEP = 45` | the wobble in the table above, now visible |
| `SPOKES = 12` | a twelve-spoke wheel; `PER_SPOKE` adjusts itself |
| `position="straight"` | the default easing — the judder, plainly visible |
| `FACTOR_SECONDS = 3.0` | each factor held twice as long; the wheel slows to match |
| two turns in `turn()` | the path recited twice |

Note that `FACTOR_SECONDS` is the knob, not the per-event duration. A factor is
named while its spoke crosses the top, which takes `PER_SPOKE` events, so the
Timing follows from how long you want to read it:

```python
STEP_SECONDS = FACTOR_SECONDS / PER_SPOKE
```
