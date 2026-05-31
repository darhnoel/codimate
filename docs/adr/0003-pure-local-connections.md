# ADR 0003 — Connections are pure & local; automatic arrangement is deferred to a layout engine

**Status:** Accepted — 2026-06-01

## Context
The driving use case is diagramming — e.g. animating the "Attention is all you need" figure: connecting boxes, fan-in (several arrows into one box), bent/routed arrows (residuals), and animated flow ("firing"). Every mature diagram tool (Graphviz, draw.io, yEd) does this with **automatic** port distribution, obstacle-avoiding routing, and box placement.

But automatic arrangement requires a **global pass** over the whole scene — and, because Codimate shapes animate, potentially **per frame**. That contradicts the One Law: in `f(t) → Scene`, each Node resolves purely from *its own* data. An arrow whose geometry depends on *sibling* arrows or *other* boxes is no longer local or pure, and animated re-routing has a known stability/jitter problem.

## Decision
**Connections are anchor-based and pure-local:**
- A **Connection** links exactly **two** Anchors (pairwise). "Many arrows at one box" = multiple pairwise Connections sharing that box's edge.
- An **Anchor** is a point on a shape's boundary, resolvable at `t` as an `Animated<Vec2>` derived from the shape's own animated geometry — so a Connection tracks its shapes as they animate.
- Fan-in/out uses **explicit Ports** (`bottom_port(i, n)` — "slot i of n", evenly divided).
- Bends use **explicit manual waypoints** (`start → waypoints → end`); straight is the no-waypoint case.
- The line is **stroked** (width + color) — this also gives boxes real borders. Arrowhead via `.arrow()` at the target end.
- Flow animation is a **Pulse**: a dot travels along the path via `point_at(progress) → Animated<Vec2>` (arc-length). The line + arrowhead stay fully drawn; the Pulse is an overlay, not a reveal.
- Construction is **clone-based**: keep the shape in a variable, add `.clone()` to the scene, connect via its anchors (anchors are `Arc`-backed, so the clone and the variable share the same animated values).

**All automatic arrangement is deferred** to a future global **layout engine** in `codimate-layout`:
- automatic port allocation ("just connect them, they spread themselves"),
- automatic orthogonal routing (arrows that dodge boxes),
- automatic box placement.

These three are the *same* global-pass problem, and the manual primitives above are exactly the substrate the engine emits into (an auto-router just *computes* the waypoints/ports you would otherwise write).

## Consequences
- Ships now, stays pure and golden-testable, and can draw the Transformer figure by hand today.
- The layout engine becomes a well-scoped future project that upgrades ergonomics **without changing the Connection primitive** — it just generates ports/waypoints/positions.
- New capability pulled in: **stroke** (line-with-width), shared by Connections and box borders.
- Deferred: named-handle construction (clone-based for now), both-ends arrowheads, glow-segment pulses, continuous looping (Layer 3).
- The user's `cssbox`/Tailwind instinct splits into two separate future efforts — **style tokens/themes** (styling) and this **layout engine** (positioning) — to be grilled on their own. A full CSS *engine* is rejected for the same reason as Bevy/nannou in [ADR 0001](0001-rendering-backend.md): it is stateful/retained and fights `f(t) → Scene`.
