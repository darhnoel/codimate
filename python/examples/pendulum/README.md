# Pendulum — continuous motion from sampled physics

```bash
.venv/bin/python python/examples/pendulum/main.py
```

A pendulum released at 50 degrees swings under gravity while a little damping
gradually reduces its amplitude. The movement comes from the pendulum equation,
not from an animation chosen to look pendulum-like.

## What it teaches

**The simulation and the animation have different jobs.** `Pendulum.step()`
updates angle and angular velocity using ordinary Python. It knows nothing
about Codimate or pixels. The view converts the current angle into the bob's
position and draws one moment.

```python
x = pivot_x + rod_length * math.sin(angle)
y = pivot_y + rod_length * math.cos(angle)
```

**Continuous-looking motion can come from discrete moments.** The trace samples
the simulation thirty times per second:

```python
for _ in range(round(DURATION / DT)):
    pendulum.step()
    cm.emit("tick")
```

Codimate interpolates between those positions. Each interpolation is a chord,
not a mathematically exact circular arc, but the largest inward error here is
about a quarter of one canvas pixel. Dense samples make the distinction
invisible.

**Use `linear` when the state already contains the physics.** Default motion
eases into and out of every event. That is useful for distinct algorithm steps,
but a sampled simulation is always mid-journey. Linear interpolation avoids
adding thirty tiny accelerations and stops every second:

```python
motion=[cm.Rule("*", position="linear")]
```

**Names keep the pendulum assembled.** `"string"` and `"bob"` identify the
same objects in every moment. Their endpoints are interpolated together, so
the string remains attached to the bob between samples.

**One mark can carry two measurements.** The filled sector follows the rod, so
its size and side show the instantaneous angle. Its colour comes from the
pendulum's remaining mechanical energy: green at the initial amplitude,
shifting toward red as damping reduces the equivalent turning angle. The colour
scale spans the amplitude loss visible during this ten-second lesson, while the
number beside the sector reports its current angle directly. The sector
collapses at each centre crossing, but its colour continues its gradual change.

## Try changing

| Change | What happens |
|---|---|
| `DAMPING = 0.0` | the ideal pendulum keeps almost the same amplitude |
| starting angle `10.0` | the small-angle approximation becomes very accurate |
| starting angle `120.0` | the nonlinear swing becomes visibly slower |
| `LENGTH_METRES = 2.0` | a longer physical pendulum swings more slowly |
| `DT = 1 / 10` | the straight chords and coarse simulation become easier to see |
| remove `position="linear"` | the bob surges and stalls at every sampled moment |
