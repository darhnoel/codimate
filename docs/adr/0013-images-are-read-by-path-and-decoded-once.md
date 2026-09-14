# ADR 0013 — Images, read by path and decoded once

**Status:** Proposed — 2026-09-14

## Context

Codimate cannot show a picture. `Geometry` is `Circle | Rect | Path | Text`, so
a screenshot, a figure from the paper being explained, a logo, or a photograph
has no way into a frame.

Everything else missing from the Authoring Surface has a workaround. An arc is
a `curve` through points on a circle — `pendulum` already approximates its
angle sector with sixteen straight segments. A ring is a stroked circle —
`dharma_wheel` still stacks four discs, and the comment explaining why stopped
being true when fill-plus-stroke landed. Dashes, gradients and edge routing are
cosmetic or composable.

A photograph cannot be approximated. It is the only remaining gap with no way
round it, in a library whose stated purpose is explainer video.

`fourier_einstein` is the shape of the problem: it reads `einstein.png` and
never shows it. The image is consumed at build time to extract contours, and
the portrait a viewer sees is traced, because tracing was the only way to get
anything of that picture onto the screen.

## Decision

**Add a fifth `Geometry`, `Image`, whose payload carries a path. The Engine
reads and decodes the file once and caches the pixels.**

    scene.image("figure", "paper/attention.png", w=520)
    scene.image("logo", "brand.png", at=cm.at(x=1180, top=24), w=90)

### The payload carries a path, not pixels

`text` already means "words, or a LaTeX source". It becomes "words, a LaTeX
source, or a file path" — a third reading of a field that already had two, and
the payload gains nothing.

The alternative is embedding the pixels. A one-megabyte PNG crossing the FFI
boundary once per Scene, for the seventy-four Scenes of an example like
`fourier_einstein`, is seventy-four megabytes of payload for one picture that
never changes. The payload is a flat struct of numbers and short strings by
deliberate choice (ADR 0008), and this would end that.

### Decoded once, cached by path

A 1,200-frame render must not decode the same PNG 1,200 times. The file is read
and decoded on first use and kept in a cache keyed by path — the same shape as
the formula glyph cache, which solves the identical problem for typeset
mathematics.

The cache must be `Send + Sync`, because frames rasterize in parallel (ADR
0011). A `OnceLock<Mutex<HashMap<String, Arc<Pixmap>>>>` matches what the
formula cache already does, and is on the hit path after the first frame.

**A consequence worth stating plainly: the file is read once per process and
never re-read.** Editing an image mid-session and re-rendering shows the old
one. That is the right trade for a render, and it will surprise somebody.

### Sizing, and the aspect ratio

`w` and `h` carry the drawn size, as for a rect. Giving neither draws the image
at its pixel size; giving one derives the other from the file's aspect ratio,
so a caller does not have to know the dimensions of their own asset to avoid
distorting it. Giving both stretches, because sometimes that is what is wanted
and an author who writes both has said so.

### A missing file is an error

`focus("typo")` is an error rather than a silent no-op, because a camera
pointing at whitespace is the failure that design exists to prevent. A missing
image is the same: a blank rectangle where a figure should be is worse than a
render that stops and says which path it could not open.

### Images transform properly, unlike text

`draw_pixmap` takes a `Transform`, so `.turn()` and `.grow()` work on an image
the way they work on a rect. Rotating text still moves the shape without
turning the glyphs. That asymmetry is not new, but images make it visible — the
first primitive where every part of the Handle does what it says.

### Two images tween only if they are the same file

Position, size and opacity interpolate like any other shape, so an image slides
and fades with no new machinery. When the path changes, the later image stands
for the whole segment — the rule `text` already follows when its content
changes, and `polygon` when its corner count does. Cross-fading two pictures is
achievable today with two shapes and opposing opacities, which is clearer than
inventing a blend nobody asked for.

### PNG first, JPEG with it

`png` is already a workspace dependency and decodes as well as encodes. JPEG
needs `jpeg-decoder`, a new dependency in the wheel — accepted, because
"put a photo in" is a main case and photographs are JPEG.

PNG gives straight alpha and tiny-skia wants premultiplied; the conversion
happens once, at decode, not per frame.

## Consequences

- **Assets live on the author's filesystem**, resolved relative to the working
  directory, and are not bundled into anything. A moved file breaks a render,
  the way a moved file breaks an `import`.
- **The wheel grows** by a JPEG decoder.
- **`KINDS` reaches eight**, which is the top of the range the architecture
  review quoted in ADR 0010 predicted the flat union would strain at. The next
  kind after this one should expect a harder argument than this one got.
- **Memory scales with distinct images, not with frames or Scenes**, because of
  the cache. Ten images at 4K is roughly 330 MB of decoded pixels, which is
  worth knowing before someone builds a slideshow.
- **`fourier_einstein` could show its source** beside the traced version, which
  is a better explanation than either alone.

## Alternatives rejected

**Embed pixels in the payload.** No filesystem access at render time, so a
render is reproducible from the payload alone. Rejected on size: the payload is
built per Scene and would carry the same megabyte once per event.

**Read the file in Python and pass a decoded buffer.** Keeps decoding out of
the Engine and reuses Pillow, which authors often have. Rejected for the same
size reason, and because it puts a decode step in the Authoring Surface that
ADR 0008 says belongs in the Engine.

**A `background` parameter on `render()` instead of a shape.** Much smaller —
one image, behind everything, no new kind. Rejected because it only serves the
one case and cannot move, scale, fade, or sit between two layers, and the
moment someone wants two pictures it is a dead end.

**Do nothing, and keep tracing.** `fourier_einstein` proves an image can be
turned into something Codimate already draws. Rejected because contour tracing
works on line art and not on photographs, and because "convert your screenshot
into paths first" is not an answer anyone will accept.
