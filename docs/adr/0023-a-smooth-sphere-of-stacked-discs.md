# ADR 0023 — A smooth sphere of stacked discs

**Status:** Accepted — 2026-10-16

## Context

A `Sphere` in `"solid"` style (ADR 0019) is an icosphere: 1,280 faces at the
default detail, each projected, shaded, named and sorted far to near, about half
of them drawn. That is right for one planet. For a scene of many balls it is the
whole cost of the film. A cluster of forty balls in the examples repository
(`about-codimate/blobs`) was about 6,300 shapes a frame; it took over a minute to
build at 60 fps and still showed faint facets.

Drawing each ball instead as about thirty stacked circles, each smaller, nearer
the light and brighter, was tried in that example. It read as perfectly smooth,
cost 1,280 shapes for the forty balls, and built in under ten seconds. Its
shading was hand-tuned for one light, fixed on screen.

## Decision

`Sphere(style="smooth")` draws a ball as stacked discs, shaded by the same
`Light` as `"solid"`:

- `detail` is the number of discs, 32 by default; 8 shows bands like contour
  lines.
- Each disc shows as a crescent on the side away from the light. It takes the
  shade the light gives the ball's surface in the middle of that crescent:
  `ambient + (1 - ambient) * max(0, n · light)`, mixed toward `shadow`, the
  solid style's own formula. So any light works, a moving one included, and a
  `"solid"` and a `"smooth"` ball of one colour under one light match in tone.
- The discs share the ball's depth and stack in order, so a ball sorts among
  other bodies, smooth or meshed, as one thing.

## Consequences

- Many lit balls are cheap and smooth: a molecule, a cluster of cells, a heap of
  particles.
- A smooth ball has no surface. `spin` and `tilt` show nothing, and there is no
  mottle or limb darkening; those stay with `"solid"` and `"star"`.
- Balls are ordered as wholes. Two that sink deep into each other are drawn one
  simply in front, where a mesh would at least interleave their faces. Neither
  is right, since there is no depth buffer (ADR 0016).
- The shading is a model of a lit ball, not a lit ball: the terminator is soft
  where a real one is sharp. At the sizes these films use, it reads as a sphere.
