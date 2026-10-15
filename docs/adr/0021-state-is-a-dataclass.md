# ADR 0021 — The state is written as a dataclass

**Status:** Accepted — 2026-10-15

## Context

Every page taught the state as a dict: `cm.trace(move, {"x": 100})`, then
`state["x"] = 540` in the algorithm and `frame.state["x"]` in the view. It works,
but it reads badly once a film has more than one or two fields, and it fails
quietly: `state["tpyed"] = ...` makes a new key, and the view goes on drawing the
old one. Nothing says which keys a film has, or where they start.

`cm.trace` never needed a dict. It deep-copies whatever it is given (or calls
`snapshot=`), so any plain object works, and examples such as `ripple` already
passed a class of their own.

## Decision

Documentation writes the state as a `@dataclass`:

```python
@dataclass
class Ball:
    x: float = 100

cm.trace(move, Ball())       # the algorithm writes ball.x, the view reads frame.state.x
```

The tutorial starts with it, the concepts, reference and recipes use it, the
`trace` docstring recommends it, and the agent skill's example follows it. A dict
is mentioned once, as fine for a quick sketch. A list of Items stays the state for
things that move; the dataclass is for state with named fields.

No code changes: this is a recommendation, not a requirement.

## Consequences

- Fields, their types and starting values are declared in one place, and a
  misspelled field is an `AttributeError` instead of a silent new key.
- Editors complete `frame.state.` and can jump to the field.
- The usual dataclass rule applies: a list or dict field needs
  `field(default_factory=...)`. The recipe says so.
- The state is deep-copied at every event, as before. A dataclass makes it easy
  to put large objects in it; the recipe says to keep it to plain data or pass
  `snapshot=`.
- Existing films that use dicts keep working unchanged.
