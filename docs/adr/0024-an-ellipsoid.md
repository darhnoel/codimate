# ADR 0024 — An ellipsoid

**Status:** Accepted — 2026-10-16

## Context

The science kit's only body was a `Sphere`: one radius. Much of what a science
film shows is round but not a sphere: cell nuclei and eggs, molecules' electron
clouds, a planet flattened by its spin. The `"smooth"` style (ADR 0023) was first
tried on a cluster of nuclei, whose real shapes were mostly egg-shaped.

Two ways were open: give `Sphere` an optional `radii=`, or add a body of its own.
Much of `Sphere` is built on its single radius: an orbit hiding behind it, ring
arrows riding its equator, `on_equator`, the axis that `spin` and `tilt` turn.
Each would have to decide what "its radius" means.

## Decision

A separate `Ellipsoid(centre, radii, color, style, turn, detail)`: a unit ball
stretched by three radii along x, y and z, then turned by `turn` degrees about x,
then y, then z. The `World` draws it with the spheres and sorts them together.

Both bodies now hand the `World` the same two things: `_place(unit)`, a point on
the surface in a direction of the unit mesh, and `_stretch()`, the matrix that
turns the unit ball into the body (for a sphere, its radius).

- **`"solid"`, `"flat"`, `"wire"`** are the sphere's mesh, placed through
  `_place`. Face normals already come from the placed faces, so the shading is
  right for the stretched shape with no new code.
- **`"smooth"`** works on the unit ball. Seen through the stretch, the outline is
  the great circle square to `stretch⁻¹ · eye`, the discs are circles in that
  plane, and the stretch carries every disc to the same ellipse on screen: a
  circle grown to the ellipse's two axes and turned to its angle. Each disc is
  shaded at the middle of its crescent by the true surface normal there,
  `stretch⁻ᵀ · spot`. For a sphere every one of these reduces to the old circle,
  and the sphere's output is unchanged.
- No `"star"`: its mottle turns with `spin`, which an ellipsoid does not have.

## Consequences

- A squished ball costs what a round one does: 32 shapes smooth, the mesh solid.
- In perspective the outline of an ellipsoid is not exactly an ellipse; the discs
  use the projection at the centre, which at these films' distances cannot be
  told apart. With an orthographic camera it is exact.
- Where a single radius is needed, an orbit hiding behind it or a wire mesh's
  depth fade, the largest radius stands in.
