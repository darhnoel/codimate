# Otsu's Method — a threshold chosen by the histogram

```bash
.venv/bin/python python/examples/otsu/main.py
```

Needs `scikit-image` and `Pillow`, which is also how the arithmetic is checked:
`otsu.py` compares its own threshold with `skimage.filters.threshold_otsu` on
import. Both say 107.

Four panels — the picture, the mask, the histogram, the score curve — revealed
one at a time. The threshold is a single number in the trace, so every panel is
a pure function of it and none of them can drift out of step with the others.

## What it teaches

**An image is pixels, and Codimate cannot compute pixels.** `scene.image`
places and fades a file; it does not threshold one. So a sliding threshold is a
*sequence of PNGs*, written before the render and named by their threshold.
They are regenerated every run, because `scene.image` reads a file once per
process and caches it by path — a stale PNG would be drawn for the whole film
without complaint (ADR 0013). The same caching is why the mask *cuts* between
thresholds instead of cross-fading, which here is right: a mask that dissolved
into the next mask would show grey pixels that are in neither.

**Fixed point counts are what let a shape change size.** The two class bands
grow and shrink as T slides, and they only tween because `otsu.band` resamples
to a fixed 96 points however wide the band is. Sliced from the histogram
instead, they would gain and lose points and snap from one shape to the next
(ADR 0010). The score curve is 256 points for the same reason.

**The one-pass trick is the method.** `compute_between_class_variance_curve`
takes running sums of the histogram, so both classes' weight and mean at *every*
cut come out of one sweep of 256 numbers rather than 256 passes over a hundred
thousand pixels. The film says this after the picture has already made the
point, not before.

## Three jumps that had to be sampled

A position tweens in a **straight line**, so a large jump draws a chord instead
of a path. The threshold jumped in three places, and each one looked wrong in
its own way:

| where | what it looked like |
| --- | --- |
| end of the first sweep (222 → 107) | the line races back across the histogram and the bands re-split at speed |
| before the score chart arrives (107 → 30) | the same race, under everything else fading up |
| end of the second sweep | the marker travels in a straight line *through* the curve rather than along it |

All three are now `settle()`, which hands the threshold over eighteen steps.
The marker stays on the curve because it is placed from the curve at each step.

## Why two beats per section

A shape entering or leaving a Scene fades, and the fade takes the **whole**
beat — so a panel appearing at the top of a five-second section spends five
seconds arriving. Every section is therefore a short `swap` beat where the
panels change, then a long one where nothing changes at all and the section
simply holds.

## The shape of it

| file | what it knows |
| --- | --- |
| `otsu.py` | the histogram, the curve, the PNGs. No Codimate; checks itself against scikit-image. |
| `main.py` | the trace and the view. |

## What to try changing

- The picture. `load_image` returns `skimage.data.coins()`; anything 8-bit grey
  works, and the last section — where one global threshold loses the dimly lit
  coins — is a property of *that* picture, so it may stop being true.
- `SWEEP`. Coarser and the search reads as steps; finer and it costs frames
  without reading any smoother.
