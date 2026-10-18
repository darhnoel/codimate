# ADR 0027 — A marker that draws on

**Status:** Accepted — 2026-10-18

## Context

Codimate can mark a word only by putting a plain rectangle behind it. The
rectangle resizes between moments, but the words are a single text shape whose
content is taken from the start of each move, so when the highlight moves to a
wider or narrower word the letters can overflow the band. This is a property of
the Engine's text tween, not of the highlight, and it stays for every label.

The Remotion rough-notation highlight is a better model: a hand-drawn,
translucent stroke over any region, drawn on with a progress from 0 to 1 and
with the words sitting over it. It is not tied to text, so the same marker can
sit on a number, a diagram or an empty patch of the screen.

## Decision

A marker is a polygon in the Scene payload, built in Python. It needs no new
Engine shape kind and no Rust change.

- `scene.marker(name, *, w, h, at=None)` draws a rough stroke over the box
  `w` by `h`. It is centred on `at`, is slightly translucent (opacity 0.85,
  bright on the black background) and sits on layer 9, under the words on
  layer 10.
- **Draw-on.** `.write(reveal=0..1)` draws the stroke on from its left end. The
  points are regenerated from the reveal, so the outline always has the same
  number of points, and the Engine tweens one stroke into the next, as it does
  for any polygon of the same size.
- **Angle and direction.** The stroke is laid out along its own x axis, so
  `.turn(degrees)` sets both its angle and the direction it draws.
- **Shape.** Drawn as a hand would: straight along the top and bottom, a
  slight tilt (0.5 to 1.5 degrees), and ends cut at a slant like a chisel-tip
  pen with three small bumps biting in. The tilt and bumps are seeded from the
  shape's name, so they stay put from moment to moment. Earlier versions
  wandered every corner at random (a jagged zigzag), were a clean
  parallelogram (a sticker), and then bent their edges in a slow wave, which
  still read as a zigzag along the middle.
- **One colour.** The stroke is one shape of one even colour. A version with
  denser ink bands at each end read as extra edges and was dropped.
- **Turning.** `.turn(degrees)` is laid out in Python, about the marker's
  centre, on top of the hand's tilt.
- **Colour.** `.fill(name)` maps six highlighter names to hex in Python:
  yellow `#ffec4f`, orange `#ffa33a`, pink `#ff8fc7`, green `#8ee88a`, blue
  `#7fd1ff` and purple `#c9a3ff`. Any `#rrggbb` also works. On a marker these
  names mean the highlighter colours, not the engine's named colours.
- **Highlighting a word.** `scene.highlight(name, words, *, size, at)` draws a
  marker over the words' measured box, padded by a quarter of `size` on every
  side, and the words as a separate text shape. The words are near-black
  (`#171717`) while marked, and `write(reveal=...)` on the marker blends them
  back to their own colour as it rubs out, in step with the stroke. Their own
  colour is white unless `words.fill(...)` sets it, which is how a film on a
  white page keeps them dark. Each word needs its own name,
  and the words never change text under it, so they cannot overflow the marker.
  Moving the highlight means one word's marker rubs out and another draws on,
  with both changed in the same moment's progress.
- **Easing.** The draw-on uses the Engine's own easing between moments. A spring
  curve is not added here.

## Consequences

- No Engine change. Colour names and the reveal are Python-side, and the
  renderer already draws polygons.
- A marker draws on and off only between moments: the Engine tweens between
  them, so a single moment shows a fixed stroke.
- At 0.85 opacity the colours read bright on black. A plain `marker` over
  text does not recolour that text; only a `highlight` links its words.
- The shape is a simplification of rough-notation's stroke, not a port.
