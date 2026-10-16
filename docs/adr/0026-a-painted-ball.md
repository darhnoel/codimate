# ADR 0026 — A painted ball

**Status:** Accepted — 2026-10-16

## Context

A ball can be drawn three ways in the science kit, and two of them show their
construction. `"solid"` is a mesh: faint facets, and about 640 shapes a ball.
`"smooth"` (ADR 0023) is stacked discs: each disc is one flat colour, so the
shading steps from band to band instead of flowing, and more discs only make
the steps smaller. Tried on a pile of balls dropping onto a floor, both were
judged not good enough to publish, the stepped gradient most of all.

The Engine fills a shape with one colour. It can also place an image.

## Decision

`style="painted"`, for `Sphere` and `Ellipsoid`: the ball is one picture.

- The shading is painted pixel by pixel with numpy: the solid style's light
  (ambient, mixed toward `shadow`), a soft sheen where the surface faces
  halfway between the light and the eye, a slightly darker rim, supersampled
  for a clean edge, transparent outside the ball.
- The light is given in the picture's own terms (across, up, toward the eye),
  rounded to 0.02, and each look is painted once and kept on disk under
  `~/.cache/codimate/balls`, written whole or not at all. A light that is still
  relative to the camera reuses one painting for the whole film.
- The `World` places the picture at the body's centre, sizes it to the body,
  and for an `Ellipsoid` stretches and turns it to the outline the `"smooth"`
  style already works out (`_unit_view`, now shared by both).

numpy and Pillow are an **optional extra**, `codimate[painted]`, not a
dependency: Codimate's only dependency stays `imageio-ffmpeg`. Without them the
style raises an `ImportError` that names the extra; nothing else changes.

## Consequences

- A ball is one shape with a truly smooth gradient. The dropping-balls film
  went from 24 seconds to render with stacked discs to 4 with paintings.
- A light that moves relative to the camera (a planet round a sun, seen by a
  still camera) asks for a new painting at each new angle. They are cached and
  quick, but it is not free; `"smooth"` and `"solid"` remain for that.
- An egg is the painted ball stretched, so its shading leans a little where the
  stretch is strong. The exact shading of an egg is `"solid"` or `"smooth"`.
- The picture has no surface: `spin` and `tilt` show nothing, as for `"smooth"`.
- This is not gradient fills in the Engine. Skia's radial gradients would let
  any circle be filled with one; that is Rust and payload work, kept for if it
  is ever wanted beyond balls.
