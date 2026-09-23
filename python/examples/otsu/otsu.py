"""Otsu's method, and the pictures it needs. No Codimate in this file.

Everything here is arithmetic on a histogram plus a few PNGs written to disk,
which is the seam worth splitting on: it can be checked without rendering
anything, and it is checked against scikit-image's own implementation.
"""

import pathlib

import numpy as np
from PIL import Image
from skimage import data

# Written next to the video, which is not committed. Regenerated every run,
# because `scene.image` reads a file once per process and caches it by path —
# a stale PNG would be drawn for the whole film without complaint (ADR 0013).
CACHE = pathlib.Path("results/otsu")

LEVELS = 256


def load_image():
    """scikit-image's coins: 303x384, 8-bit grey, and unevenly lit.

    The uneven lighting is not a flaw for this example, it is the point of the
    last section — the coins at the bottom sit on a brighter background than
    the ones at the top, which is exactly where one global threshold struggles.
    """
    return data.coins()


def compute_histogram(image):
    """How many pixels at each of the 256 grey levels."""
    return np.bincount(image.ravel(), minlength=LEVELS).astype(np.float64)


def apply_threshold(image, t):
    """Otsu's convention, and scikit-image's: foreground is *above* t."""
    return image > t


def compute_between_class_variance_curve(hist):
    """`w0 w1 (mu0 - mu1)^2` at every threshold, in one pass.

    Otsu's insight is that this needs no search over pixels. Running sums of
    the histogram give both classes' weight and mean at every cut at once, so
    the whole curve costs one sweep of 256 numbers rather than 256 passes over
    a hundred thousand pixels.
    """
    levels = np.arange(LEVELS, dtype=np.float64)
    total = hist.sum()

    below = np.cumsum(hist)                      # pixels at or under t
    above = total - below
    weight_sum = np.cumsum(hist * levels)
    whole_sum = weight_sum[-1]

    # Empty classes have no mean; their variance is zero either way.
    busy = (below > 0) & (above > 0)
    w0 = np.where(busy, below / total, 0.0)
    w1 = np.where(busy, above / total, 0.0)
    mu0 = np.divide(weight_sum, below, out=np.zeros(LEVELS), where=below > 0)
    mu1 = np.divide(whole_sum - weight_sum, above,
                    out=np.zeros(LEVELS), where=above > 0)
    return np.where(busy, w0 * w1 * (mu0 - mu1) ** 2, 0.0)


def compute_otsu_threshold(hist):
    """The threshold that separates the two classes best."""
    return int(np.argmax(compute_between_class_variance_curve(hist)))


def class_split(hist, t):
    """`(w0, w1, mu0, mu1)` at one threshold, for putting numbers on screen."""
    levels = np.arange(LEVELS, dtype=np.float64)
    low, high = hist[:t + 1], hist[t + 1:]
    total = hist.sum()
    return (low.sum() / total, high.sum() / total,
            float((levels[:t + 1] * low).sum() / max(low.sum(), 1)),
            float((levels[t + 1:] * high).sum() / max(high.sum(), 1)))


def band(hist, lo, hi, points):
    """A filled outline of the histogram over `[lo, hi]`, as `points + 2` xy.

    The count is *fixed* whatever `lo` and `hi` are, which is what lets the two
    class bands tween as the threshold slides. A shape that gained or lost
    points as it grew would stop interpolating and snap instead (ADR 0010), so
    the band is resampled rather than sliced.
    """
    lo, hi = float(lo), float(max(hi, lo + 1e-6))
    xs = np.linspace(lo, hi, points)
    ys = np.interp(xs, np.arange(LEVELS), hist)
    return ([(lo, 0.0)] + [(float(x), float(y)) for x, y in zip(xs, ys)]
            + [(hi, 0.0)])


def _png(path, array):
    path.parent.mkdir(parents=True, exist_ok=True)
    Image.fromarray(array).save(path)
    return str(path)


def write_pictures(image, thresholds):
    """Every picture the film shows, as PNGs, keyed by what they are.

    A mask per threshold, because an image is pixels: the Engine can place and
    fade one but cannot compute it, so a sliding threshold is a sequence of
    files rather than one file that changes.
    """
    out = {"grey": _png(CACHE / "grey.png", image)}
    for t in thresholds:
        mask = apply_threshold(image, t)
        out[("mask", t)] = _png(CACHE / f"mask{t:03d}.png",
                                (mask * 255).astype(np.uint8))
    return out


def write_foreground(image, t):
    """The coins kept, the background dropped — and the other way round."""
    mask = apply_threshold(image, t)
    return (_png(CACHE / f"fore{t:03d}.png", np.where(mask, image, 0)),
            _png(CACHE / f"back{t:03d}.png", np.where(mask, 0, image)))


def _agrees_with_scikit_image():
    """The method, checked against the library that ships it."""
    from skimage import filters

    image = load_image()
    hist = compute_histogram(image)

    mine, theirs = compute_otsu_threshold(hist), filters.threshold_otsu(image)
    assert mine == int(theirs), (mine, theirs)

    # The curve really is `w0 w1 (mu0 - mu1)^2`, computed the slow honest way
    # at a few thresholds and compared with the one-pass version.
    curve = compute_between_class_variance_curve(hist)
    for t in (40, 107, 180):
        w0, w1, mu0, mu1 = class_split(hist, t)
        assert abs(curve[t] - w0 * w1 * (mu0 - mu1) ** 2) < 1e-6, t

    # It peaks where it should, and the peak is a real maximum.
    assert curve[mine] == curve.max()
    assert curve[mine] > curve[mine - 20] and curve[mine] > curve[mine + 20]

    # Bands keep their point count however wide they are — the thing that
    # lets them tween as the threshold slides.
    assert len({len(band(hist, 0, t, 96)) for t in (1, 50, 128, 255)}) == 1
    return True


assert _agrees_with_scikit_image()
