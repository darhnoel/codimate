# Codimate explaining Codimate

```bash
.venv/bin/python python/examples/explain_codimate/main.py
```

Three stored pictures of a bubble sort, and the formula that computes every
frame between them.

## What it teaches

**The whole model, on one screen:**

```text
local = (t - start) / duration
value = before + (after - before) * ease(local)
```

A frame is never played forward. Ask for time `t` and the Engine finds the
segment containing it, turns `t` into a local 0..1, eases that, and
interpolates. Any moment can be rendered in any order, and twice the same way.

**It is self-referential on purpose.** The trace records 28 instants; the
playhead gliding between them is moved by exactly the formula it is drawing.

**It does not keep a second copy of the truth.** The curve is `cm.ease` — the
Engine's own easing, called into rather than re-typed. An earlier version had
its own `ease_in_out`, which meant the diagram would have quietly started lying
the day the Rust curve changed.

**Both identity modes appear.** The segment boxes and the curve are named after
their *place* and never move; the playhead and the bars are named after the
*thing* and travel.

## Try changing

| Change | What happens |
|---|---|
| `TICKS = 6` | the playhead visibly steps — you can see the samples |
| `TICKS = 60` | smoother, and no truer: the frames between were always computed |
| a different `MOMENTS` table | the bars explain a different algorithm |
