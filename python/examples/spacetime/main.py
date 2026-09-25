"""Space deformed by a mass moving through it.

    python python/examples/spacetime/main.py

A regular lattice, with every point moved to where it would have *fallen* from
rest after one fixed stretch of time. The mass goes round a small circle and
the camera creeps right to left, so the deformation is seen travelling and
turning at once.

Colour is the river model: how fast space falls inward, `v/c = sqrt(rs/r)`.
"""

import codimate as cm

import lattice

cm.canvas(1280, 720)

# Blue where the lattice is still nearly regular, green where it is strongly
# deformed — two colours saying one thing, rather than a rainbow that has to be
# decoded. Both stay off white so the mass is the brightest thing in frame.
RAMP = ("#4f83cf", "#3fa7bb", "#46d98a")
SUN, PAPER, BAND = "#ffffff", "#dfe7f2", "#05070c"

# Green marks the strongly distorted middle, not anything merely nearer than
# average. Too low and the whole cube drifts green, so "farther away is flat"
# stops being visible; too high and the green shrinks to nothing.
SHARP = 0.85

# A minute of film. The mass keeps its pace — about three seconds a circle —
# and simply goes round twenty times, rather than one slow lap stretched out.
TURNS, PER_TURN = 20, 48
STEPS = TURNS * PER_TURN


def _nodes():
    edge = lattice.REACH
    spread = [-edge + 2 * edge * k / (lattice.NODES - 1)
              for k in range(lattice.NODES)]
    return [(x, y, z) for x in spread for y in spread for z in spread]


def _edges():
    """Every edge between neighbouring nodes, sampled so it can bend."""
    edge = lattice.REACH
    spread = [-edge + 2 * edge * k / (lattice.NODES - 1)
              for k in range(lattice.NODES)]
    out = []
    for axis in range(3):
        for a in spread:
            for b in spread:
                for k in range(lattice.NODES - 1):
                    lo, hi = spread[k], spread[k + 1]
                    pts = []
                    for i in range(lattice.ALONG):
                        t = lo + (hi - lo) * i / (lattice.ALONG - 1)
                        pts.append((t, a, b) if axis == 0 else
                                   (a, t, b) if axis == 1 else (a, b, t))
                    out.append((("web", axis, a, b, k), pts))
    return out


NODES, EDGES = _nodes(), _edges()
EDGE_MIDDLE = [pts[len(pts) // 2] for _, pts in EDGES]

# One brightness scale for the whole film, taken over every camera angle it
# passes through. Using each frame's own range instead makes the ruler move:
# everything is renormalised every frame and the lattice pulses where nothing
# happened.
_SEEN = [lattice.look(p, lattice.SWEEP * k / 8)[1]
         for p in NODES for k in range(9)]
NEAR, FAR = min(_SEEN), max(_SEEN)

# Draw order is fixed for the whole film, taken once at mid-sweep.
#
# It has to be. The Engine resolves order per *segment*, not per frame — it
# sorts by layer when it builds a segment and keeps that order through it — so
# any layer that changes is a hard cut at a scene boundary. At sixteen scenes
# a second that is the flicker, and it gets worse the more honest the layers
# are: ranking every shape by depth each frame reorders nearly all of them at
# every boundary.
#
# Nothing is lost by freezing it. These lines are translucent and unfilled, so
# which of two crossing lines is drawn first cannot be seen. The one shape
# that does need its place is the mass, which is opaque — and it is one shape,
# so it moves through the order alone instead of dragging a thousand with it.
_MID = lattice.SWEEP / 2


def _rank(away):
    """A fixed place in the drawing order: further off draws first."""
    return round(4000 * (FAR - away) / (FAR - NEAR))


EDGE_LAYER = [_rank(lattice.look(p, _MID)[1]) for p in EDGE_MIDDLE]
NODE_LAYER = [_rank(lattice.look(p, _MID)[1]) for p in NODES]


def orbit(state, emit):
    for step in range(1, STEPS + 1):
        state["part"] = TURNS * step / STEPS
        emit("tick")


def _ramp(t):
    """A colour off the ramp. One lerp between two colours cannot do this —
    it saturates early and the middle of the range goes muddy."""
    t = max(0.0, min(1.0, t)) * (len(RAMP) - 1)
    lo = min(int(t), len(RAMP) - 2)
    cold, hot, part = RAMP[lo], RAMP[lo + 1], t - lo
    return "#%02x%02x%02x" % tuple(
        round(int(cold[i:i + 2], 16) * (1 - part)
              + int(hot[i:i + 2], 16) * part)
        for i in (1, 3, 5))


def _caption(scene, name, words, size, place):
    """Words on a band of their own.

    Layer alone does not keep a caption legible. It does put the text on top —
    shapes sort by layer, and nothing caps the value — but pale letters over a
    bright lattice still read as crossed out. The band is what makes "never
    hidden" true rather than hoped for.
    """
    wide, high = cm.measure(words, size=size)
    # `measure` reports the width too, and ignoring it is how a caption ends
    # up running off both edges of the frame. Shrink to fit rather than trust.
    room = cm.width() - 96
    if wide > room:
        size *= room / wide
        wide, high = cm.measure(words, size=size)
    scene.rect((name, "band"), w=cm.width(), h=high + 30, at=place) \
         .fill(BAND).on(layer=9000, opacity=0.88)
    scene.text(name, words, size=size, at=place).fill(PAPER).on(layer=9001)


def view(frame):
    scene = cm.Scene()
    part = frame.state["part"]
    sun = lattice.sun_at(part)
    drift = lattice.SWEEP * part / TURNS

    for (name, pts), middle, layer in zip(EDGES, EDGE_MIDDLE, EDGE_LAYER):
        front = 1.0 - (lattice.look(middle, drift)[1] - NEAR) / (FAR - NEAR)
        # The edge's own middle, not the hottest of its samples. `max` picks a
        # different sample from one frame to the next as the mass slides, so
        # the colour steps instead of sliding.
        hot = lattice.heat(middle, sun) ** SHARP
        flat = [lattice.look(lattice.warp(p, sun), drift)[0] for p in pts]
        scene.curve(name, flat, w=1.1 + 1.3 * front + 1.2 * hot) \
             .fill(_ramp(hot)) \
             .on(layer=layer, opacity=0.22 + 0.34 * front + 0.34 * hot)

    for p, layer in zip(NODES, NODE_LAYER):
        at, away = lattice.look(lattice.warp(p, sun), drift)
        front = 1.0 - (away - NEAR) / (FAR - NEAR)
        hot = lattice.heat(p, sun) ** SHARP
        scene.circle(("node", p), r=1.4 + 2.2 * front + 1.2 * hot, at=at) \
             .fill(_ramp(hot)).on(layer=layer, opacity=0.26 + 0.46 * front)

    # By its NEAR surface, not its centre. The body is solid, so it hides the
    # front half of the collapsed shell that lands on it — by centre-depth
    # that half sorts in front and draws as scratches across the disc.
    #
    # And at the size it actually is: a disc smaller than the sphere
    # everything collapses onto leaves that shell outside itself, a cage of
    # chords wrapping a ball half its size.
    star, star_away = lattice.look(sun, drift)
    scene.circle("star", r=lattice.LENS * lattice.SUN_R / star_away * 1.04,
                 at=star).fill(SUN) \
         .on(layer=_rank(star_away - lattice.SUN_R))

    _caption(scene, "what", "Curved Space", 34, cm.at(x=640, top=36))
    return scene


cm.explain(
    trace=cm.trace(orbit, {"part": 0.0}),
    view=view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=3.0 / PER_TURN, opening=0.3, final_hold=0.3),
).render("results/spacetime.mp4", fps=60, scale=1.5)

print("wrote results/spacetime.mp4")
