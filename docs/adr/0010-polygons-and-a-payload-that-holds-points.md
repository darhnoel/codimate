# ADR 0010 — Polygons, and the first payload field that is not a number

**Status:** Accepted — 2026-09-13

## Context

The Authoring Surface draws five kinds: `rect`, `circle`, `text`, `line`,
`formula`. There is no triangle, no polygon, no arbitrary path. `docs/drawing.md`
says so plainly and offers workarounds: thick lines for edges, or columns for a
fill.

The workarounds are in the repository and they are not design choices.
`bernoulli_lift` draws its wing as **a series of vertical columns**, because a
closed outline cannot be filled. Any arrow head, wedge, chevron or triangle in
a future example would need the same treatment.

The Engine could already draw one. `Geometry::Path` is fully supported and used
constantly: `line` is a path, formula glyphs are filled paths, and rect and
circle are *converted* to paths before rasterizing. Filling a closed polygon is
code that runs on every frame today.

So this is the fourth capability found sitting in the Engine with nothing
exposing it, after LaTeX, text measurement, and fill-plus-stroke. The others
were plumbing. This one is not, and that is why it needs a decision.

**A polygon has a variable number of points, and the payload is a flat struct
of scalars.** One `f32` can carry three meanings — `r` is a circle's radius, a
rect's corner radius, and a formula's reveal — because a number is a number. A
list of points cannot hide in a float.

ADR 0008 chose that flat payload deliberately: the Engine diffs it field by
field, with no per-kind special cases, which is what keeps the reconciler
small. An architecture review of this codebase predicted the union would strain
"around 8–10 kinds". The strain has arrived earlier and from a different
direction: not more kinds, but one kind that does not fit the shape of the
struct.

## Decision

**Add `points` to the payload, and one new kind, `polygon`.**

    points: Vec<f32>        // flat: [x0, y0, x1, y1, ...]

Flat rather than `Vec<(f32, f32)>` so it crosses the FFI boundary as a plain
sequence of numbers, and so the field-by-field diff still sees numbers.

Everything else is built **in Python, out of polygons** — no further kinds:

    scene.polygon(name, points, closed=True)   the primitive
    scene.arrow(name, start=, end=, head=)     one polygon: shaft and head
    cm.ngon(sides, r, at=)                     points for a regular polygon
    cm.star(points, r, inner=, at=)            points for a star

A triangle is `cm.ngon(3, ...)`. An arrow is an outline computed from two
endpoints. Neither needs the Engine to know it exists. This is the same move
`rect` and `circle` already make internally — they are paths by the time they
are drawn — pushed up to where an author can reach it.

### Two polygons only tween if they have the same number of points

Interpolating between a triangle and a pentagon has no obvious answer, and
inventing one produces a shape nobody asked for. When the counts differ the
later shape is used for the whole segment, which is exactly what `text` already
does when its content changes, and what a formula does when its LaTeX changes.

An author who wants a polygon to morph keeps its point count fixed and moves
the points — which is the same discipline as keeping a name stable to make a
thing travel.

### The camera transforms points too

`focus` frames a Scene by rewriting the flat payload. `points` is part of that
payload and is transformed with everything else, so a polygon pans and zooms
like any other shape and needs no special case.

## Consequences

- **The payload is no longer scalars only.** That is a real loss: it was
  serializable, diffable and describable in one sentence. It is still all three,
  but the sentence is longer. Any future non-scalar field should have to argue
  as hard as this one did.
- **`bernoulli_lift`'s column-fill and any future arrow head become one shape.**
  Existing examples are not rewritten here; the workarounds are removed when
  someone is already editing them.
- **Bounding boxes get more accurate.** `focus` measures a polygon from its
  actual points rather than a bounding rectangle it never had.
- **Nothing existing changes.** A `points` field defaulting to empty is ignored
  by every other kind, and the five current kinds take the same code path they
  did before.

## Alternatives rejected

**Encode points into the unused `text` field.** No payload change, and every
shape already carries an unused string. Rejected: stringly-typed geometry, and
it would make `text` mean "words, or a LaTeX source, or a coordinate list",
which is one meaning too many for a field that already has two.

**A `path` kind taking arbitrary Bézier segments.** More capable — curves, not
just straight edges. Rejected for now because it is a much larger surface to
design (how are control points authored? how do two paths tween?) and because
every concrete need so far — triangle, arrow, wedge, wing — is straight-edged.
`polygon` does not block it; a curved path would be a second kind alongside.

**Keep the workarounds.** They work, and `bernoulli_lift` proves a filled shape
is achievable without a polygon. Rejected because the cost is paid by every
author, forever, for a primitive every other animation tool has — and because
"draw your triangle as forty rectangles" is the kind of thing that makes a
library look unfinished, regardless of whether the output is correct.

**Ellipse, arc and sector as well.** Ellipse is a circle with two radii; arc and
sector need angles and are the natural way to draw a pie chart or an angle mark.
Deferred rather than rejected: each is a genuinely new kind rather than
something composable from points, and none has an example asking for it yet.
A regular polygon with enough sides approximates an ellipse, badly; that is not
an argument for shipping the approximation, only for waiting.
