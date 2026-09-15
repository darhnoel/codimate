# ADR 0014 — SVG is imported as geometry, not pasted as a picture

**Status:** Accepted — 2026-09-15

## Context

An Explanation Author cannot bring existing artwork into a video. A logo, an
icon, a diagram exported from Mermaid or Graphviz, an illustration drawn in
Figma — none of it has a way in. The only route today is redrawing it in
Python, shape by shape.

ADR 0013 proposes raster images for this. That answers the photograph and the
screenshot, and it is the right answer for those, but it treats every picture
as an opaque rectangle of pixels. Most of what an author wants to import is
vector art, and vector art does not have to arrive opaque.

**The conversion already exists.** `codimate-math` depends on `usvg`, and
`svg_to_paths` / `collect_nodes` already walk an SVG tree, apply each node's
absolute transform, and emit core `Path`s with multi-contour support. It runs
on every formula — the LaTeX pipeline is `mitex → typst → SVG → usvg → Path`
(ADR 0005). Importing an SVG is the same stage-3 call with a file's contents
instead of typst's output.

This is the sixth capability found finished in the Engine with nothing exposing
it, after LaTeX, text measurement, fill-plus-stroke, polygons and curves.

## Decision

**`scene.svg(name, path, size=)` imports vector art as native geometry.**

    scene.svg("logo", "brand.svg", size=90, at=cm.at(x=1180, top=24))
    scene.svg("chart", "flow.svg", size=(900, 420))

Because the result is geometry rather than pixels, it inherits everything the
Engine already does: it tweens, `.fill()` recolours it, `.grow()` and `.turn()`
transform it, `focus()` frames it, and the pen can draw it on.

### One Shape, expanded by the Engine

An imported drawing is a hundred paths, but the author names it once and thinks
of it as one thing. That is exactly what `formula` does: one Shape carrying
LaTeX in `text`, expanded into many `PathNode`s at diff time, and an author who
writes `scene.formula("eq", ...)` never meets a glyph.

`svg` follows it: the payload carries a file path in `text`, and the Engine
expands. `text` already meant "words, or a LaTeX source"; it becomes "words, a
LaTeX source, or a file path". The payload gains no field.

### It keeps its own colours

`usvg::Path::fill()` gives each path its own paint, and an imported logo should
look like the logo. `.fill(colour)` overrides every path with one colour, which
flattens it to a silhouette deliberately.

This needs a way to say *"nothing was said about colour"*, because
`_Shape.color` defaults to `"white"` and an author who genuinely wants a white
silhouette writes the same thing. **`scene.svg` sets `color=""`**, an empty
string being something no author produces by accident, and the Engine reads it
as "use the file's own fills".

That is a third meaning for `color` in a payload ADR 0010 already strained
over. It buys the thing the alternative cannot recover: under a
formula-style monochrome rule, importing a brand logo gives a white blob and
there is no way to get the colours back, whereas here flattening is one call
away.

### The pen draws it left to right

`formula` already sweeps a reveal edge across its glyphs, sorted by left edge,
so an equation writes itself on. An SVG expansion gets the same treatment, so
`.write(reveal=, pen=)` draws a logo on stroke by stroke — the most valuable
thing an import gets from being geometry rather than a picture.

Document order — the order the artist saved the paths in — was the alternative,
and for hand-built artwork it is better: it draws the way it was made. It was
rejected because most first imports come out of a tool rather than a designer's
hand, and there the saved order is arbitrary. A Mermaid flowchart emits its
arrows before its boxes, so document order draws two arrows hanging in space
and then boxes around them. Left-to-right's failure is "slightly mechanical";
document order's is "visibly broken", and a boring failure beats a broken one.

If a file turns up that wants the designer's order, that is `order=` on
`.write()`, added then.

### `size` is a box to fit inside

SVG dimensions are arbitrary — a 24-unit icon and a 1000-unit diagram both mean
"this big relative to itself" — so the author must say how big, and one number
has to mean something predictable.

`size` is the box the drawing fits inside, aspect always preserved: `size=90`
fits it in 90×90, `size=(900, 420)` in that box. A single dimension was
rejected because it fails for the other orientation — height-only runs a wide
Graphviz diagram off both edges, width-only runs a tall flowchart off the top —
and the fiddling that follows is the layout arithmetic Slots exist to abolish.

"One number, or a pair" is the third use of a shape authors have already met in
`cm.row(size=)` and `.grow()`.

Deliberate distortion is not available. Stretching a logo is nearly always a
mistake, and `.grow((2.0, 0.5))` is still there for anyone who means it.

### A missing file errors; text is refused

A missing or unreadable file is an error naming the path, for the reason
`focus("typo")` is an error: a blank space where the logo should be is worse
than a render that stops and says why.

**Labels are translated, not shaped.** `usvg` is built with
`default-features = false`, so it drops `<text>` while parsing; the elements
are read from the source in a second pass and become `Geometry::Text`
primitives, shaped at draw time by the same `codimate_glyph::shape` call that
draws every `scene.text`.

Enabling `usvg`'s own `text` feature was the obvious route and was rejected. It
brings a second font database and a second shaper, so the same string in the
same font would be shaped one way from `scene.text` and another from an import,
inside one frame. It also would not have helped: the first real file tried
asks for `system-ui`, which no embedded face provides, so its labels would be
substituted either way — and a Khmer wireframe rendered correctly *because* the
import went through the pipeline that already carries Noto Sans Khmer.

What is still refused is layout this cannot read: a `<tspan>`, a `<textPath>`,
or a label under a transform. The last is not a shortcut — the renderer cannot
turn glyphs at all, which is the same limitation `.turn()` has on `scene.text`,
so a rotated axis label has no correct rendering to fall back on. The error
names the label so the author knows which one to outline.

## Consequences

- **Two imports, for two kinds of picture.** SVG for vector art, and ADR 0013's
  raster images for photographs and screenshots, which cannot be vectorised.
  Neither replaces the other.
- **`KINDS` reaches eight**, the bottom of the "8–10 kinds" the architecture
  review quoted in ADR 0010 predicted the flat union would strain at. If ADR
  0013's images land too it is nine, and that should be the moment the
  prediction is re-examined rather than waved past.
- **The pen moved fields, which is the strain showing.** `w` is a formula's pen
  width and an imported SVG's fit box, so `.write(pen=)` cannot simply write to
  `w` any more — it asks the shape what kind it is and puts an SVG's pen in
  `size`, which text and formula use and `svg` does not. Found by building it:
  the first pen test fitted the artwork into a three-pixel box. Reusing fields
  per kind is the payload's design, but this is the first time the reuse became
  visible from Python rather than staying inside the Engine.
- **Gradients flatten.** `Style` carries a flat colour, so a gradient fill
  becomes one of its stops. Better than refusing the file; worth stating so it
  is not discovered.
- **The file is read once per process**, cached by path exactly as formula
  glyphs are, because a 1,200-frame render must not parse the same SVG 1,200
  times. Editing the file mid-session and re-rendering shows the old one.
- **No new dependency**, no new payload field, no new `Geometry` variant. The
  reveal machinery is now shared: `formula` and `svg` both hand a list of
  outlines to one `revealed()`, which was previously the formula branch inline.
  `pendulum` renders byte-identical across that refactor.

## Alternatives rejected

**Render SVG output instead.** `render("out.svg")` for embedding in a web page.
Rejected because it bypasses the rasterizer entirely — it is a second renderer
wearing a feature's clothes, and the Engine's whole job is per-frame
rasterization.

**Honour the SVG's own animation.** SMIL and CSS animation in the file,
replayed. Rejected outright: it would be a second source of motion competing
with the reconciler, which is the imperative corner in a declarative system that
ADR 0009 rejected for cameras and ADR 0008 rejected for everything.

**Convert the SVG to paths in Python and hand over points.** No Engine change,
and `polygon`/`curve` could carry the result. Rejected because it puts geometry
in the Authoring Surface that ADR 0008 places in the Engine, and because the
conversion already exists in the Engine and would be duplicated.

**Treat an SVG as a raster image** by rendering it to pixels at import. One
mechanism instead of two, once ADR 0013 exists. Rejected because it throws away
everything that makes vector art worth importing: no tween, no recolour, no
pen, and pixelation under `focus()`.

**Do nothing, and redraw artwork in Python.** Honest, and what every example
does today. Rejected as the deferred-work answer rather than the design answer
— but see the status. This is Proposed, and no file yet exists that wants it.
