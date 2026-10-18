# ADR 0028 — Rules set feel and timing, not only the path

**Status:** Accepted — 2026-10-18

## Context

A `Rule` chose one of four paths for a shape's position, and nothing else.
Everything a shape does moved together across the whole beat: position on the
Engine's one ease-in-out, size, colour, opacity and turn linearly. A row of bars
could not cascade, nothing could overshoot or settle, and nothing could curve
except by lifting and dropping. The Engine core already had `ease_in`,
`ease_out` and `back`, unused.

## Decision

A rule warps a matching shape's **time**, and that one warp reaches everything
the shape does: position, size, colour, opacity, turn, a marker's reveal.

- **`ease=`** — `linear`, `in`, `out`, `in_out` (cubic), `back`, `spring`
  (with `bounciness`, 0..1, scaled to land exactly on 1) and `bounce`.
- **`delay=`, `stagger=`** — fractions of the beat, so they mean the same at
  any `Timing`. Shapes a rule matches start `stagger` apart in `order` —
  `screen` (left to right, then top to bottom), `reverse`, `name`, `centre`
  (from the middle out) — all take the same time, and the last finishes as the
  beat ends. A stagger too wide for that is narrowed, never run over. Only
  shapes that change in the beat take a place in the queue.
- **`position="arc"`** — a quadratic curve through a control point `bend` of
  the distance off the middle of the line.
- `ease=` replaces the built-in ease of `straight` and `arc`, which become
  pure geometry. On `linear`, `fall` and `lift_carry_drop` it warps their time.
- **Defaults do not move.** A rule with none of these renders exactly as
  before, which a test checks frame by frame.

How: `Primitive::ease(TimeCurve)` already retimed a primitive's transform,
style and geometry together; it is now public, and `build_segment` gives each
changing shape its clock. The rule crossing from Python grows from three
fields to five — pattern, path, numeric options, ease, order — still a tuple,
so the bindings need no conversion code.

## Consequences

- Cascades, overshoot, springs and arcs need no new shape kinds and no
  per-property plumbing.
- One warp per shape: position cannot ease differently from colour in the
  same rule. Separate properties would need separate rules per property,
  deferred until a film needs it.
- `ease="in_out"` (cubic) is a little smoother than the default curve
  (quadratic), which stays as it was so existing films do not shift.
