# Dharmachakra — rotation without a rotate

```bash
.venv/bin/python python/examples/dharma_wheel/main.py
```

The wheel of the Noble Eightfold Path, turning. A fixed marker sits on the rim
at the top; the spoke passing it brightens, and its factor is named below.

## What it teaches

**Codimate has no rotation, and does not need one.** There is no `rotate=`
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

**Rings out of discs.** A circle can only be filled, so the rim is a gold disc
with a ground-coloured disc on top of it, and the hub is the same trick. The
marker is a third — a notch punched out of the rim. Layers do the rest.

**The marker never moves.** It is drawn at a fixed point and the wheel turns
underneath it, which is both simpler and truer to what a wheel is.

## Try changing

| Change | What happens |
|---|---|
| `STEP = 45` | the wobble in the table above, now visible |
| `SPOKES = 12` | a twelve-spoke wheel; `PER_SPOKE` adjusts itself |
| `default=0.6` in `Timing` | a slow, contemplative turn |
| two turns in `turn()` | the path recited twice |
