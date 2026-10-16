# ADR 0022 — An exact orthographic camera

**Status:** Accepted — 2026-10-16

## Context

`science.Camera` (ADR 0019) is a perspective camera, and its answer for a flat
view was "a very large `distance` is an orthographic camera". That holds to about
one part in the distance, and it is how the kit's own test checks it
(`distance=1e9`).

Some films need the flat view to be the point, not an approximation. The depth
illusions in the examples repository (a wireframe, a cloud of dots or a helix
turning with no perspective, so the eye cannot tell which way it spins) each wrote
their own projection rather than use the Camera, because "stand a million units
back" reads as a workaround. Engineering drawings and maps want the same thing.

## Decision

`Camera(orthographic=True)`: every unit of space is `scale` pixels at any depth.
`distance` still places the camera, so `project` still returns a depth for draw
order. The camera's other answers follow the same view:

- `facing` and the shading's eye direction use the view direction itself, the
  same for every point, since the viewer is at infinity;
- `hidden_by` tests a line of sight parallel to the view;
- a sphere's outline is `r * scale` pixels, not shrunk by depth.

Two private helpers, `_per_unit(depth)` and `_toward_eye(point)`, carry the
difference, so no caller branches on the flag.

## Consequences

- An orthographic view is exact and says what it means:
  `science.Camera(azimuth=a, elevation=8, scale=180, orthographic=True)`.
- `dolly` keeps the lens and changes `scale`, so an orthographic dolly still
  shrinks things, though only because `scale` changed. A flat view has no
  pull-back of its own.
- Perspective cameras are unchanged.
