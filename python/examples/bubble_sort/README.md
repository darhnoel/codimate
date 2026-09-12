# Bubble sort — things that move

```bash
.venv/bin/python python/examples/bubble_sort/main.py
```

Four bars. The pair being compared turns orange; when they swap, they arc over
each other and land in each other's places.

## What it teaches

**Names follow the thing.** A bar is named after the value it represents, so
when value 3 moves from the first slot to the second, the Engine sees one shape
that changed position and slides it. Name it after the *slot* instead and
nothing would ever travel — the bars would just change height in place.

`cm.items([3, 1, 4, 2])` is what makes that possible. Two 3s in a list are two
different bars, and only an identity can say so. Items order by value, so the
algorithm stays ordinary Python:

```python
if values[j] > values[j + 1]:
```

**A thing made of parts is one group.** Each bar is a rectangle *and* a label,
drawn on one `scene.group(item.id, slot)`. They cannot come apart, because the
Engine sees them as `3/bar` and `3/label` — one name, two shapes.

**Motion is derived.** Nothing here says "move this bar over 0.9 seconds". The
view describes each moment; the arc comes from one rule:

```python
motion=[cm.Rule("*", position="lift_carry_drop", clearance=90)]
```

## Try changing

| Change | What happens |
|---|---|
| `cm.items([5, 2, 8, 1, 9])` | more bars, more events, same code |
| `cm.Rule("*", position="straight")` | bars slide through each other instead of arcing |
| `events={"swap": 1.5}` | swaps linger, comparisons stay quick |
| name the group `position` instead of `item.id` | nothing travels — see for yourself |
