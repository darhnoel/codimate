# Moving ball — one name, two positions

```bash
.venv/bin/python python/examples/moving_ball/main.py
```

This is the smallest useful explanation of Codimate's central idea. Python
changes a value and `cm.emit()` records the new moment. The view draws a circle
named `"ball"` in both moments, so Codimate knows it is the same thing and
connects its old and new positions with motion.

## What it teaches

The algorithm describes what changed:

```python
ball["x"] = 980
cm.emit("move")
```

The view describes what one moment looks like:

```python
scene.circle("ball", r=34, at=(frame.state["x"], 360))
```

There are no keyframes or animation commands. The shared name `"ball"` gives
the two circles one identity; their different positions imply the movement.

## Try changing

| Change | What happens |
|---|---|
| `980` to `640` | the journey becomes shorter |
| `default=2.0` to `0.5` | the same journey becomes faster |
| rename only one circle | the connection is lost; one shape fades into another |
