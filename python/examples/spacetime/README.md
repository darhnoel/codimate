# Curved Space — a lattice deformed by a mass inside it

```bash
.venv/bin/python python/examples/spacetime/main.py
```

A wireframe cube of 343 nodes and 882 edges, with every point moved to where it
would have *fallen* — toward a mass that circles inside it while the camera
sweeps right to left. Blue where the lattice is still nearly regular, green
where it is strongly deformed.

One minute, twenty orbits, and the deformation is computed from the mass's
position every frame rather than painted on.

## What it teaches

**Deformation as free fall, in closed form.** The displacement is not a guess
at how bent a line should look. Every lattice point is dropped from rest and
moved to where it gets to after one fixed stretch of time:

    x'' = -mu / x^2,   x(0) = r,   x'(0) = 0

Far out a point barely stirs in that time; close in it travels most of the way,
so the grid bunches exactly where the well is deep. It needs no numerical
solver — radial free fall from rest is a cycloid,

    d = (r/2)(1 + cos e)        t = sqrt(r^3 / 8mu) (e + sin e)

so the whole map is one inversion of `e + sin e`, monotone on `[0, pi]` and
tabulated once. `lattice.py` checks it is the real solution rather than
something shaped like it: a fall must reach the middle at exactly
`(pi/2) sqrt(r^3 / 2mu)` and be half way down when the cycloid says.

**The pile-up is the picture, not a bug.** Points close enough to complete the
fall all stop together on the body's edge. Two attempts were spent smoothing
that away before noticing it is what draws the lines converging into the mass.
The check now allows it and *bounds* it instead — it must stay an inner shell,
under 60% of the way out, or "flat far away" would not be shown at all.

**Colour is a real quantity.** The river model (Hamilton and Lisle): in
Gullstrand-Painlevé coordinates space falls inward at exactly the Newtonian
escape velocity, `v/c = sqrt(rs/r)`, reaching light speed at the horizon. So
the blue-to-green ramp is the inflow speed, not a measure of how bent a line
happens to look.

## Draw order, and the flicker

This is the part worth reading the code for, and it took four attempts.

The lattice flickered. Three explanations were offered and fixed before the
real one was found by reading `codimate-reconcile`:

```rust
let mut items: Vec<&Shape> = after.iter().chain(before.iter()).collect();
items.sort_by(|x, y| (x.layer, &x.item).cmp(&(y.layer, &y.item)));
```

**Draw order is resolved once per *segment*, not per frame.** A layer that
changes is therefore not a smooth reordering — it is a hard cut at a scene
boundary. At 48 scenes per three-second orbit that is sixteen cuts a second.

Every earlier attempt made it worse by being *more* honest. Quantising depth
into bands reorders the shapes that cross a band edge; ranking every shape by
depth each frame reorders nearly all of them.

**So the order is frozen: 1225 of the 1226 shapes take their layer once, at
import.** Nothing is lost. These lines are translucent and unfilled, so which
of two crossing lines draws first cannot be seen. The one shape that genuinely
needs its place is the mass, which is opaque — and being one shape, it moves
through the order alone instead of dragging a thousand with it.

Two smaller versions of the same mistake are fixed alongside it:

- **A moving ruler.** `near`/`far` were each frame's own min and max depth, so
  every line's brightness was renormalised against a range that shifted as the
  mass moved. They are now taken once, over every camera angle the film passes
  through.
- **A jumpy maximum.** Each edge took `max(heat)` over its 21 samples, and
  *which* sample was the maximum jumped as the mass slid, so the colour
  stepped. It uses the edge's own midpoint.

## Other things that were wrong first

**A body drawn smaller than it is.** The mass was drawn at a fixed 26 pixels
while everything collapsed onto its sphere at `SUN_R`, which projects to about
46 — so the collapsed shell sat *outside* the disc as a cage of chords wrapping
a ball half its size. It is drawn at its true projected radius now, and sorted
by its **near** surface rather than its centre, so it hides the front half of
that shell instead of letting it draw as scratches across the disc.

**Nine samples per edge was not enough.** An edge passing near the mass is bent
hard and unevenly, and a spline through nine samples cuts corners the real
curve does not have. Twenty-one.

**Face-on is the wrong angle.** Looking straight at the lattice, the depth
lines project to near-vertical and the cube reads as a flat grid with a tunnel
in it. A three-quarter view is what makes the deformation read as happening in
a volume.

## The shape of it

| file | what it knows |
| --- | --- |
| `lattice.py` | the fall, the river, the camera. No Codimate; checks itself. |
| `main.py` | the trace and the view. |

## What this is not

A depiction, not an embedding of the metric. The honest embedding is
`p -> p psi^2` in isotropic coordinates and it was tried first: the
displacement it gives tends to `rs/2` — a constant, not something that dies
away — so a distant mass shifts the whole lattice rigidly, and a mass moving
past would slide the entire grid about rather than deform it locally.
