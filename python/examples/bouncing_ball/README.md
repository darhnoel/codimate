# Bouncing ball — gravity decides the motion

```bash
.venv/bin/python python/examples/bouncing_ball/main.py
```

This example adds gravity and collision response to `moving_ball`. Python
updates velocity and height thirty times per second. If an impact happens
between two samples, it solves when the ball actually touches the floor,
reverses its velocity at that instant, and simulates the rest of the frame.
Codimate connects the resulting positions.

```python
impact_velocity = velocity + GRAVITY * time_to_impact
rebound_velocity = -RESTITUTION * impact_velocity
```

`RESTITUTION = 0.72` means the ball leaves the floor at 72% of its impact
speed. Because kinetic energy depends on speed squared, it keeps about 52% of
its kinetic energy after each collision. The rest represents energy lost to
sound, heat, and deformation.

Once an impact is too slow to produce a visible bounce, the ball settles on
the floor. This avoids the tiny numerical jitter that otherwise continues
forever in a sampled simulation.

The physics decides where the ball is; Codimate does not fake the bounce. Every
scene still contains the same `"ball"`, so its changing position becomes one
continuous movement. `linear` interpolation is used because the closely spaced
samples already contain the acceleration.
