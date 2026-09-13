# Neural network — things that stay put

```bash
.venv/bin/python python/examples/neural_net/main.py
```

A signal moving forward through a 3-4-2 network. Neurons charge as they fire;
pulses run down the live edges.

## What it teaches

**Names follow the place.** Nothing on screen moves house, so neurons are named
`("neuron", layer, index)` — there is no `cm.items()` anywhere in this file.
This is the other half of the decision `bubble_sort` shows: a grid, a heatmap,
a matrix multiply and a network all want identity keyed to position, because
the cell *is* the thing and the value is just its state.

**Travel comes from the trace, not from a request.** The only movers are the
pulses, and the view never mentions movement:

```python
here = at[src] if net.signal_at == "source" else at[dst]
scene.circle(("pulse", src, dst), r=9, at=(here.x, here.y))
```

The algorithm emits `send` and then `arrive`. The pulse exists in both moments
at two positions, so the Engine makes it travel. Change the trace and the
motion changes with it.

**No motion rules at all.** A straight line is the default, which is what a
signal down a wire should do.

## Try changing

| Change | What happens |
|---|---|
| `LAYERS = (4, 6, 3)` | a bigger network, no other edits |
| `events={"arrive": 1.2}` | signals take their time crossing |
| name a neuron after its value instead of its place | neurons start swapping seats |
