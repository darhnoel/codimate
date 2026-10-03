# ADR 0019 — A science kit that draws flat

**Status:** Accepted — 2026-10-02

## Context

A long run of science explainers — buoyancy, the photoelectric effect, Hawking
radiation, atoms shaking with heat, microwaves on water, the Earth and Moon and
Sun, the Cavendish experiment — kept writing the same dozen things again.
Each one did it a little differently, and each copy carried a bug the others
had already fixed:

- **A projection from space to screen** exists three times in the repository
  already, and again in every 3D sketch: an orthographic orbit camera in
  `helical_solar_system`, a hard-coded isometric one in `rubiks_cube`, a
  perspective one in `spacetime`. They differ, but they are the same
  arithmetic, and none is shared. One `Camera` is all three: isometric by
  default, orthographic with a very large `distance`, perspective otherwise.
- **A label on a plate**, seven times. Its first version centred the plate and
  its text on two different points, so the text sat high with a gap below it.
- **A glowing dot**, five times; **a bracket on a measured span**, **an arrow
  whose length is a force**, and **a wave**, three each — and `archimedes`
  still keeps private copies of the bracket and the arrow.

ADR 0016 had already named the gap and left it open: *"a projection helper is
possible … but '3D axes' implies depth sorting the Engine does not have."* And
`CONTEXT.md` lists 3D rendering, stateful particles and physics simulation as out
of scope. Any answer has to stay inside both.

The threshold used was evidence rather than appetite. Nothing was extracted that
had not appeared in several pieces of real work and survived being fixed there.

## Decision

**`codimate.science` — an optional kit in the Authoring Surface, with no change to
the Engine.** No new kind, no payload field, no Rust. It is arithmetic that ends
in the shapes that already exist, following `cm.axes` exactly: each piece works
out pixels and hands them back, and where it draws it draws ordinary shapes under
names the author chose the prefix of. That keeps what ADR 0016 protected — a
name here is the identity the reconciler runs on, so a curve or a face the author
cannot name is one they cannot animate, focus or aim a rule at.

It has two halves.

**Marks** (flat): `tag` (a label on a plate, with `leader` and `edge` for
pointing at it), `Glow`, `wave`, `Bracket`, `ForceScale`.

**Space** (three dimensions, drawn flat): `Camera`, `Sphere`, `Light`, `Orbit`,
`World`, `RingArrows`.

It is a submodule, `from codimate import science`, rather than thirteen more names
in `cm.*`: nothing in it is needed to make a film.

### 3D is projection, and the Engine stays 2D

The Engine is not given a camera or a depth buffer. A `Camera` turns a point in
space into a pixel; a `World` turns spheres into polygons or lines, sorts every
face of every body far to near, and gives each its own layer. Nothing is rendered
in three dimensions. It is *drawn flat in the right order*, which is a smaller
promise, and the limit ADR 0016 stated still holds and is kept: painter's order is
correct when shapes admit a separating plane. Convex bodies that stay apart do.
Bodies that interpenetrate, or anything not a sphere, need a depth buffer this does
not have, and the docs say so.

### Simulation stays in the Algorithm

Nothing in the kit holds state. `Orbit.point(angle)` is closed-form — it solves
Kepler's equation for an angle the *trace* supplies — and a `Sphere` is a value the
view is handed, not an object that moves itself. So "physics simulation" stays
where `CONTEXT.md` puts it: in the Algorithm, sampled into Trace Events. The kit
only says where a thing is *at* an angle.

### The decisions that were forced, not chosen

Each of these was a mistake in a sketch first. They are here because they are what
a reasonable first version gets wrong.

- **One moment per frame.** Draw order is resolved once per segment (found by
  `examples/spacetime` the hard way). A world whose depth order changes — any
  turning ball — must emit a moment every frame, or its faces re-sort in jumps and
  a ring tears across a planet. `World` documents it; the example sets
  `Timing(default=1 / fps)`.
- **Perspective stretches spheres.** At a wide field of view a sphere far from the
  middle of the picture is an egg (silhouette ratio 1.36, against 1.02 from further
  back). The camera is defined by `distance` and `scale`, defaults to a long
  lens, and says so. `dolly` pulls back with the same lens, so things shrink;
  `moved(distance=)` keeps their size and only changes perspective. They are
  different shots and the names say which.
- **A geodesic mesh, not latitude and longitude.** A lat/long wireframe seen near
  edge-on shows both poles as dense tufts that read as a peanut. Finer lines do not
  help: the poles are the problem. A subdivided icosahedron has no special points.
- **Faces are shaded opaque.** Mixing a face's colour toward a shadow colour looks
  the same on black as shading by opacity, and does not let everything behind show
  through on any other background.
- **Facing and occlusion are tested exactly.** "Normal points along the viewing
  axis" is only right for a camera at infinity. "Which half of its own orbit is this
  on" is only right for a point on the body's surface — for a ring far outside the
  body it wrongly hid about half of what the body never covered. `facing` and
  `hidden_by` are a real perspective test and a real ray against the sphere.
- **An orbit is an ellipse with the sun at a focus**, and `point` takes mean
  anomaly, so equal steps of time cover unequal ground and the body quickens near
  the sun without the author doing anything.
- **A label's plate and its text share one centre**, resolved once. `top=` means
  two different centres for two boxes of different height.
- **One scale for every force arrow in a film**, with a visible break where one is
  cut at its cap — kept from `archimedes`, where the arrows are the argument.

### What was left out

Left out for want of evidence, not for want of use:

- **The caption plate** (a plate sized to the line, a mark stepping word by word)
  exists in `archimedes`, a simpler one in `spacetime`, and a copy of the first in
  a sketch. It is not science, and wants its own home rather than this one.
- Nested-scale zooming ("powers of ten"), damped oscillation and grid diffusion
  each appeared once. Once is not a shape yet.
- Containers, thresholds and graphs: `cm.axes` already covers the graph, and the
  rest are particular to one topic.
- Molecules and the torsion balance are domain, not kit.

## Consequences

- **Several private copies can go**, and none have in this change: the bracket and
  arrow in `archimedes`, and the projection in `helical_solar_system`,
  `rubiks_cube` and `spacetime`. The first two are straight replacements. The
  projections are not: each of those examples has its own argument for its draw
  order (the cube's took four attempts, the lattice's froze its order at import),
  and `World` re-ranks every moment, which is what turning spheres need and what
  a lattice moving at sixteen moments a second must not do. Moving one means
  checking that argument first, not swapping a call.
- **The kit makes a dense scene easy**, and a dense scene is expensive. Every face
  is a named shape, a world is a few thousand a frame, and `render()` writes an
  index of every shape of every moment — 470 MB beside a 1.5 MB video for
  `examples/orbits`, and 1.3 GB for a denser sketch. `render(index="auto")`
  now skips an index past 50 MB, and says so.
  `Sphere.detail` defaults to the coarsest mesh that does not show facets.
- **The layer budget is one integer per face.** `World.draw` returns the next free
  layer, and `ABOVE` is above any world.
- **A new surface to keep.** The guards: no function takes more than five
  arguments (a look is a frozen dataclass, as `Axes.looks` is), and every helper
  hands back pixels or draws named shapes. The kit has its own checks, and the
  tests that matter were each shown to fail when the thing they guard is broken.
- `examples/orbits` renders the whole kit end to end, so `test_examples` covers it.

## Alternatives rejected

**A depth buffer in the Engine.** The honest answer to interpenetrating solids, and
a much larger decision: `KINDS` is full, the rasteriser would grow a z plane, and
none of the spheres here needed it. Painter's order per face is exact for them.

**Put the pieces in `layout.py` and `cm.*`.** It is where `axes` went. But `axes`
is one name in everyone's film, and this is thirteen in some people's. A submodule
costs one import line and keeps the core's namespace the size it is.

**A lens model — `focal` and `distance` — for the camera.** The camera most
people reason about is "one unit is this many pixels at the thing I am looking at".
`scale` says that directly, and `dolly` gives the pull-back a lens model would have
made natural, without making every author do the division.

**Shade by opacity**, as the sketches did. It works on black. It stops being true
the day the background is not.

**Keep copying.** What happened. Each copy fixed its own bug, and the next one
began without the fix.
