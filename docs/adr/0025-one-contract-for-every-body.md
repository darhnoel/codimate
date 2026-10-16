# ADR 0025 — One contract for every body, and four new ones

**Status:** Accepted — 2026-10-16

## Context

The science kit's 3D half lived in one file, `space.py`, which by the time an
`Ellipsoid` joined the `Sphere` held the vector maths, the `Camera`, the
icosphere, the `Light`, both bodies, orbits, the `World` and ring arrows. The
`World` knew it was drawing a ball: it read `body.r`, built the icosphere itself
and flipped each face outward against the centre on every draw. A rod, an
arrow, a tube or a box (the next shapes science films ask for: bonds between
atoms, vectors in space, strands and knots, crystals and containers) would each
have meant another special case inside it.

## Decision

**Split by what each part is for**, flat under `science/`: `_vectors.py` (the
arithmetic, private), `camera.py`, `light.py`, `_meshes.py` (the icosphere and
the builders shapes are made from, private), `bodies.py`, `orbits.py`,
`world.py`. The public names (`science.Camera`, `science.Sphere`, …) are
unchanged.

**One contract.** Every body gives the `World`:

- `centre`, `color`, `style`, and `reach`, how far its farthest point is from its
  centre, which stands in wherever one radius is needed;
- `_mesh()`: points, faces wound outward, and edges, for the mesh styles;
- a round body also gives `_stretch()` and `levels` for the `"smooth"` style.

The icosphere's faces are wound outward once, when it is built; turning or
stretching it by positive amounts keeps them so. The `World` never asks which
kind of body it has.

**Four bodies on it**, each just a class in `bodies.py`:

- `Cylinder(start, end, r, caps="flat" | "round")`, the round one a capsule;
- `Arrow(start, end, r, head, spread)`, a shaft and a cone;
- `Tube(points, r, closed)`, a ring swept along a path by parallel transport,
  any twist left at a closed loop's join spread evenly round it;
- `Box(centre, size, turn)`.

Rods and arrows share one builder, `lathe`: a profile of (distance along,
radius) turned about the axis, so an arrow is the profile shaft, out to the
cone's rim, in to the tip.

## Consequences

- The split changed no drawn shape: 7,823 shapes across three cameras, every
  style, eggs, an orbit and ring arrows came out identical before and after.
- A new shape is one class; the `World` stays as it is.
- The four new bodies draw `"solid"`, `"flat"` or `"wire"`. `"smooth"` is for
  round bodies; a rod as stacked shaded stripes is possible later.
- They are not convex in general (a tube, an arrow), and the `World` sorts face
  by face with no depth buffer, so where a body passes through itself or
  another, the order can be wrong (ADR 0016).
- Tests check each new mesh by its signed volume, which is right only if every
  face is wound outward: a box exactly, a cylinder to its polygon, a closed
  tube to within 3% of a torus.
