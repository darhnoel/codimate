"""Otsu's method — choosing a threshold by looking at the histogram.

    python python/examples/otsu/main.py

Four panels, revealed one at a time: the picture, the mask it makes, the
histogram the threshold cuts, and the score that cutting earns. The threshold
is one number in the trace, so every panel is a pure function of it and they
cannot drift out of step.

`otsu.py` holds the arithmetic and imports no Codimate; it checks itself
against scikit-image's own `threshold_otsu`.
"""

import codimate as cm

import otsu

cm.canvas(1280, 720)

INK, DIM = "#e8eef7", "#8b96a8"
BELOW, ABOVE = "#4f83cf", "#f2a33c"      # the two classes, everywhere
BEST, LINE = "#46d98a", "#ff6b8a"
PANEL = "#141922"

# Where the four panels live. Two pictures down the left, two charts down the
# right, and nothing else on screen at once.
SHOT = (262.0, 226.0)
MASK = (262.0, 496.0)
SHOT_SIZE = (306.0, 242.0)
HIST_AT, HIST_SIZE = (872.0, 214.0), (500.0, 150.0)
SCORE_AT, SCORE_SIZE = (872.0, 492.0), (500.0, 132.0)

IMAGE = otsu.load_image()
HIST = otsu.compute_histogram(IMAGE)
CURVE = otsu.compute_between_class_variance_curve(HIST)
BEST_T = otsu.compute_otsu_threshold(HIST)

HIST_TOP = float(HIST.max()) * 1.08
SCORE_TOP = float(CURVE.max()) * 1.15
BAND_POINTS = 96                     # per class, fixed, so the bands tween

# Every threshold the film shows. Written as PNGs up front, because an image
# is pixels: the Engine places and fades one but cannot compute it, so a
# sliding threshold is a sequence of files.
SWEEP = list(range(30, 223, 2))
SHOWN = sorted({*SWEEP, 60, 100, 160, BEST_T})
PICTURES = otsu.write_pictures(IMAGE, SHOWN)
FOREGROUND, BACKGROUND = otsu.write_foreground(IMAGE, BEST_T)


# The film opens already showing this, so nothing fades up from an empty
# screen. The trace's first `emit` is a hold, not a change.
OPENING = {"title": "How can we separate objects from the background?",
           "say": "A grey picture: every pixel is a number from 0 to 255.",
           "show": ("shot",), "t": 100, "split": False, "found": False}


def _nearest(t):
    """The mask actually written for a threshold near `t`."""
    return min(SHOWN, key=lambda k: abs(k - t))


def story(state, emit):
    """The whole lesson, as a walk through one number and what is on screen.

    Every section is two beats: a short one where the panels change, and a long
    one where nothing changes at all.

    That split is not tidiness. A shape entering or leaving a Scene fades, and
    the fade takes the *whole beat* — so a panel appearing at the top of a
    five-second section spends five seconds arriving. Making the change its own
    quarter-second beat puts the fade where it belongs and lets the section
    simply hold.
    """

    def beat(name, **change):
        state.update(change)
        emit("swap")         # a quarter second: the panels change here
        emit(name)           # and this holds, with nothing changing at all

    def settle(target, steps=18):
        """Walk the threshold to `target` instead of jumping to it.

        A sweep that ends at 222 and then cuts to T* in one beat rewinds: the
        line races back across the histogram, the bands re-split at speed, and
        the marker on the score curve travels in a straight line *through* the
        chart rather than along the curve, because a position tweens
        straight. Sampling the way back keeps the marker on the curve.
        """
        was = state["t"]
        for k in range(1, steps + 1):
            state.update(t=round(was + (target - was) * k / steps))
            emit("settle")

    # 1. The problem. Only the photograph.
    #
    # No `beat`: the film already opens on this, because the trace starts from
    # it. Emitting it as a change would make the opening a fade up from black.
    emit("intro")

    # 2. One threshold, and what it does.
    beat("cut", title="Pick a threshold T",
         say="Below T becomes black, at or above T becomes white.",
         show=("shot", "mask"), t=100)

    # 3. Different thresholds, different answers. No `swap` needed — only the
    # mask's file changes, and a picture whose path changes does not
    # cross-fade, it cuts (ADR 0013).
    for t in (60, 100, 160):
        state.update(t=t, say=f"T = {t}")
        emit("try")
    beat("which", title="But which T?",
         say="Every threshold gives a different answer. Which one is right?")

    # 4. The histogram.
    beat("hist", title="Look at the histogram",
         say="How many pixels sit at each grey level.",
         show=("shot", "mask", "hist"), t=100)

    # 5. A threshold cuts it in two.
    beat("split", title="A threshold splits it in two",
         say="Everything below T is one class; everything above is the other.",
         split=True)

    # 6. The search. The main moment: mask and split move together, and this
    # one really is continuous — one emit per threshold, no holds.
    beat("search", title="Try every threshold", say="")
    for t in SWEEP:
        state.update(t=t)
        emit("slide")

    # 7. The idea, before any equation.
    settle(BEST_T)
    beat("idea", title="Otsu's idea",
         say="Choose the T that makes the two classes as unlike as possible.")

    # 8. The score, and what its parts mean.
    beat("score", title="Between-class variance", show=("shot", "hist", "eq"),
         say="w are the class sizes, mu their mean intensities.")
    state.update(say="Far apart and evenly sized scores high.")
    emit("score2")

    # 9. The curve, and its peak.
    #
    # Walk the threshold back to the start *before* the chart arrives. Setting
    # it in the same beat that brings the chart in makes the histogram line
    # race leftward across a quarter second while everything else is fading
    # up — the same jump as at the end of a sweep, in a third place.
    settle(30)
    beat("curve", title="Score at every threshold", say="",
         show=("shot", "mask", "hist", "curve"))
    for t in SWEEP:
        state.update(t=t)
        emit("scan")
    settle(BEST_T)
    state.update(title=f"Otsu threshold  T* = {BEST_T}", found=True,
                 say="The peak of the curve, found in one pass over 256 numbers.")
    emit("swap")
    emit("found")

    # 10. What it gives you.
    beat("result", title="Foreground extracted", say="",
         show=("shot", "fore", "back"))

    # 11. Where it struggles.
    beat("limit", title="One threshold for the whole picture",
         say="The coins are lit unevenly, so the dim ones at the top are lost.",
         show=("mask_big",))

    # 12. What to remember.
    beat("takeaway", title="Otsu's method", say="", show=("points",))


def _panel(scene, name, middle, size, label):
    """A dark card with a label, so a picture has somewhere to sit."""
    scene.rect((name, "card"), w=size[0] + 26, h=size[1] + 26,
               at=middle).fill(PANEL).round(10).on(layer=1)
    scene.text((name, "label"), label, size=17,
               at=cm.at(x=middle[0], bottom=middle[1] - size[1] / 2 - 18)) \
         .fill(DIM).on(layer=4)


def _histogram(scene, state):
    """The histogram, and — once T matters — the two classes it is cut into."""
    plot = cm.axes(x=(0, 255), y=(0, HIST_TOP), at=cm.at(*HIST_AT),
                   size=HIST_SIZE).looks(ink=DIM, label=15)
    plot.draw(scene, "hist", about=5)

    t = state["t"]
    if state["split"]:
        # Two bands, each resampled to a fixed number of points, so they
        # stretch and shrink as T slides instead of snapping (ADR 0010).
        for name, colour, lo, hi in (("low", BELOW, 0, t), ("high", ABOVE, t, 255)):
            pts = [plot.at(x, y) for x, y in
                   otsu.band(HIST, lo, hi, BAND_POINTS)]
            scene.polygon(("hist", name), pts).fill(colour) \
                 .on(layer=3, opacity=0.62)
    else:
        pts = [plot.at(x, y) for x, y in otsu.band(HIST, 0, 255, BAND_POINTS)]
        scene.polygon(("hist", "all"), pts).fill(DIM).on(layer=3, opacity=0.5)

    scene.line("hist_t", start=plot.at(t, 0), end=plot.at(t, HIST_TOP), w=2.4) \
         .fill(LINE).on(layer=6)
    scene.text("hist_v", f"T = {t}", size=18,
               at=cm.at(x=HIST_AT[0] + HIST_SIZE[0] / 2 - 46,
                        y=HIST_AT[1] - HIST_SIZE[1] / 2 + 6)).fill(LINE) \
         .on(layer=7)
    return plot


def _score(scene, state):
    """The between-class variance at every threshold, with a marker on T."""
    plot = cm.axes(x=(0, 255), y=(0, SCORE_TOP), at=cm.at(*SCORE_AT),
                   size=SCORE_SIZE).looks(ink=DIM, label=15)
    plot.draw(scene, "score", about=5)

    # 256 points, fixed, so the curve is one shape that simply appears.
    scene.curve("score_line", [plot.at(i, CURVE[i]) for i in range(256)],
                w=2.6).fill(BEST).on(layer=3)

    t = state["t"]
    scene.circle("score_dot", r=7, at=plot.at(t, CURVE[t])).fill(LINE) \
         .on(layer=6)
    if state["found"]:
        scene.line("score_best", start=plot.at(BEST_T, 0),
                   end=plot.at(BEST_T, SCORE_TOP), w=2.0).fill(BEST) \
             .on(layer=4, opacity=0.8)
        scene.text("score_lab", f"T* = {BEST_T}", size=19,
                   at=cm.at(x=SCORE_AT[0] + 92,
                            y=SCORE_AT[1] - SCORE_SIZE[1] / 2 - 2)) \
             .fill(BEST).on(layer=7)


def view(frame):
    scene = cm.Scene()
    state = frame.state
    show, t = state["show"], state["t"]

    scene.text("title", state["title"], size=31, at=cm.at(x=640, top=30)) \
         .fill(INK).on(layer=20)
    if state["say"]:
        scene.text("say", state["say"], size=20, at=cm.at(x=640, bottom=688)) \
             .fill(DIM).on(layer=20)

    if "shot" in show:
        _panel(scene, "shot", SHOT, SHOT_SIZE, "the picture")
        scene.image("shot", PICTURES["grey"], size=SHOT_SIZE, at=SHOT) \
             .on(layer=2)

    if "mask" in show:
        _panel(scene, "mask", MASK, SHOT_SIZE, f"pixels above T = {t}")
        scene.image(("mask", _nearest(t)), PICTURES[("mask", _nearest(t))],
                    size=SHOT_SIZE, at=MASK).on(layer=2)

    if "fore" in show:
        _panel(scene, "fore", MASK, SHOT_SIZE, "foreground kept")
        scene.image("fore", FOREGROUND, size=SHOT_SIZE, at=MASK).on(layer=2)

    if "back" in show:
        middle = (872.0, 360.0)
        _panel(scene, "back", middle, (390.0, 308.0), "background only")
        scene.image("back", BACKGROUND, size=(390.0, 308.0), at=middle) \
             .on(layer=2)

    if "mask_big" in show:
        middle = (640.0, 372.0)
        _panel(scene, "big", middle, (430.0, 340.0), f"T* = {BEST_T} everywhere")
        scene.image(("mask", BEST_T), PICTURES[("mask", BEST_T)],
                    size=(430.0, 340.0), at=middle).on(layer=2)

    if "hist" in show:
        _histogram(scene, state)

    if "curve" in show:
        _score(scene, state)

    if "eq" in show:
        w0, w1, mu0, mu1 = otsu.class_split(HIST, t)
        # Bottom left, clear of where the score chart arrives. Put it there
        # and the equation and the chart cross-fade through one another.
        scene.formula("eq", r"\sigma_B^2 = w_0\,w_1\,(\mu_0-\mu_1)^2",
                      size=38, at=cm.at(x=396, y=474)).fill(INK).on(layer=6)
        for i, (words, colour) in enumerate((
                (f"w0 = {w0:.2f}   mu0 = {mu0:.0f}", BELOW),
                (f"w1 = {w1:.2f}   mu1 = {mu1:.0f}", ABOVE))):
            scene.text(("eqv", i), words, size=21,
                       at=cm.at(x=396, top=528 + 36 * i)).fill(colour) \
                 .on(layer=6)

    if "points" in show:
        for i, words in enumerate((
                "Otsu picks the threshold for you.",
                "It maximises the difference between the two classes.",
                "It works best when their brightnesses barely overlap.")):
            scene.text(("point", i), words, size=27,
                       at=cm.at(x=640, top=262 + 62 * i)).fill(INK) \
                 .on(layer=20)
    return scene


cm.explain(
    trace=cm.trace(story, dict(OPENING)),
    view=view,
    # The threshold slide is already continuous by the time it reaches the
    # Engine, so it must not be eased again at every sample.
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(
        default=0.13,
        events={"swap": 0.26, "settle": 0.05,
                "intro": 4.5, "cut": 5.2, "try": 2.0, "which": 3.4,
                "hist": 4.8, "split": 4.4, "search": 1.1, "slide": 0.090,
                "idea": 3.2, "score": 4.2, "score2": 3.0, "curve": 1.2,
                "scan": 0.065, "found": 4.6, "result": 5.0, "limit": 5.0,
                "takeaway": 5.8},
        opening=0.8, final_hold=1.6),
).render("results/otsu.mp4", fps=60, scale=1.5)

print("wrote results/otsu.mp4")
