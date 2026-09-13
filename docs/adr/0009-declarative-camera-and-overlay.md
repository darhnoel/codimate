# ADR 0009 — A camera you point at things, and an overlay that ignores it

**Status:** Accepted — 2026-09-13

## Context

Some explanations are bigger than the frame. A six-by-six attention matrix is
legible at 1280x720; a twelve-by-twelve one is not. A Galton board is a wide
picture in which the interesting thing is one falling ball. The usual answer is
a camera — pan and zoom the view over a larger world.

`helical_solar_system` already has one. `space.py` carries an orthographic
camera that travels with the Sun, projects world coordinates onto the screen,
and streams the star field the other way for parallax. It is written by hand,
in Python, and every shape's position passes through `project()` before it
reaches a Scene. It works, and it is evidence the need is real rather than
theoretical.

Two things make a conventional camera API wrong here.

**It would be the one imperative thing left.** Every animation engine exposes
the camera as a movement — `camera.animate.move_to(x, y)`, `camera.zoom(2)`.
Codimate has no way to say that. The author describes a moment; the Engine
derives the movement by diffing consecutive moments (ADR 0008). A camera that
had to be *moved* would be the only part of the system that works the other way
round.

**Coordinates go stale.** `camera(at=(412, 338), zoom=2.5)` is correct until
somebody nudges the layout, and then it is silently wrong — pointing at
whitespace beside the thing it used to frame. This is the same failure the
Slot system exists to prevent for shapes.

There is also a consequence that is easy to miss until it is drawn. A camera
scales *everything in its space*. Zoom 2.5x on a matrix cell, and the title at
y=52 and the caption plate at y=648 are pushed off the top and bottom edges —
not clipped artfully, gone — and at 2.5x the text would be unreadable anyway.
In this project a caption is on screen during every beat, so a camera without
an answer for narration would break the narration the first time it fired.

## Decision

### A camera is aimed by name, not by coordinate

    scene.focus("cat")                    # frame this shape
    scene.focus(("cell", 2, 1), pad=60)   # any Item name, with breathing room
    scene.focus("The", "cat", "sat")      # several — the union of their boxes
    scene.focus()                         # the whole canvas again

The Engine knows where every shape is and how big it is, so it computes the
bounding box, adds padding, and derives the position and zoom itself. The
author never writes a camera coordinate.

Between one moment and the next those derived numbers differ, and the existing
tween interpolates them. **Smooth camera movement falls out of the diff**, the
same way a bar sliding between slots does. No new machinery, and no new idea
for the author: you say what to look at, and looking is animated for you.

Aiming by name also means the camera survives a layout change. Move the shape
and the camera follows it, because the name is the thing that is stable —
exactly the property that makes identity-keyed motion work.

### An overlay is the part the camera does not touch

    hud = scene.overlay()
    hud.text("title", "Scaled dot-product attention", x=640, y=52)

An overlay is a Group whose contents are positioned against the screen rather
than the world. The diagram pans and zooms underneath; titles, narration and
the head strip stay where they are put.

The default is the right way round: a shape is in the world unless it says
otherwise, because in an explainer most of the frame is diagram and only a
little is narration.

`overlay` rather than `hud`: the first reviewer of this design asked what a HUD
was. It is aviation and games jargon, and this library is meant to be read by
people who have flown neither. "Overlay" is an ordinary word that implies both
"on top" and "separate from what is underneath".

### Framing has a floor, not just a ceiling

`focus()` on a three-pixel dot is a 200x zoom. The limit is expressed as a
minimum framed size — "never frame tighter than 240px" — rather than a maximum
zoom, because it is a statement about the picture rather than about the
arithmetic, and it stays correct when the canvas changes size.

### The payload carries a camera per scene

Python hands the Engine one flat list of Shapes per Trace Event. A scene
becomes a camera and a list of Shapes. The camera is resolved on the Rust side,
where the bounding boxes are, and applied as a transform composed with
`pixel_scale` at render.

It is deliberately **not** smuggled in as another `kind` in the Shape union. A
camera is not a shape: it has no position of its own, nothing draws it, and
treating it as one would put a special case in the field-by-field diff that
every real shape would then have to step around.

## Consequences

- **The camera cannot be pointed at nothing.** `focus("typo")` is an error, not
  a silent no-op, because a camera pointing at whitespace is the failure this
  design exists to prevent.
- **Text and formula bounding boxes are required**, so `focus` depends on the
  measurement exposed in `codimate-render` (`measure_text`, `measure_formula`).
  This design was not implementable before those existed.
- **Everything in the world scales, including stroke widths and text.** That is
  what a camera does, and a camera that scales some things and not others is
  harder to predict than one that is uniform. `overlay()` is the escape hatch,
  and it is the right one for the case that actually comes up — narration.
- **`cm.measure` keeps returning world-space sizes.** A box sized to its text
  stays correct under zoom, because both scale together.
- **`focus` belongs wherever the diff lives.** It is scene semantics, not
  marshalling. Placing it in `codimate-py` would deepen the problem that crate
  already has — the reconciliation lives there and should not.

## Alternatives rejected

**`camera(at=, zoom=)` with explicit coordinates.** Simple to implement and
familiar from every other engine. Rejected because the coordinates go stale
silently, and because it is a movement rather than a description — the one
imperative corner in a declarative system.

**A screen-space layer range (`layer >= 100` ignores the camera).** No new API
at all. Rejected because it overloads `layer`, which already means draw order,
and because the rule is invisible: a reader cannot tell from the call that the
shape is pinned.

**Camera applies to groups only, top-level shapes stay fixed.** Also needs no
new API, and puts the common case in the right place by accident. Rejected as
surprising: whether a shape moves would depend on whether it happened to be
wrapped, which is a layout decision, not a camera one.

**Automatically following the Trace Event** — framing whatever just changed,
the way screen recorders follow the cursor. Attractive, and cheap once `focus`
exists, since `frame.event` already knows what happened. Deferred rather than
rejected: an automatic camera that guesses wrong is motion sickness, and that
judgement needs a working `focus()` to make.

**Doing nothing.** The strongest alternative, and it held for a while. The
attention example changes scope four times — one row, the row's output, the
matrix, two matrices — and does it by *swapping content* rather than moving a
camera, which directs the viewer's attention instead of dragging it. That is
often the better explainer. The camera earns its place where the content cannot
be swapped: one ball among many in a Galton board, or a matrix too large to
show whole.
