"""A Rubik's cube solving itself, and the same cube drawn as nine circles.

    python python/examples/rubiks_cube/main.py

A turn is a layer rotating ninety degrees. It is sampled into steps and handed
over one at a time — the way `dharma_wheel` turns and `pendulum` swings —
rather than asking the Engine to invent ninety degrees between two pictures.
So a sticker rotates instead of flying, and never shows a colour no cube has.

**A circle is a layer.** The drawing's nine circles are the cube's nine layers,
twelve places round each, every place on exactly two — the structure worked out
in `places.py` rather than assumed. So a quarter turn slides that circle's
twelve dots three places *along the circle*, and the nine the turn also carries
swing round the same centre. Nothing crosses the page.

The solve is the scramble backwards. No solver, and it cannot be wrong.
"""

import math

import codimate as cm

import cube
import geometry as geo
import graph
import places

cm.canvas(1280, 720)

INK = {"yellow": "#e7e31f", "magenta": "#d51fd1", "green": "#2ad52d",
       "blue": "#1818c6", "cyan": "#2bdad5", "red": "#d51f25"}
RING, SEAM, PAPER, CORE = "#6e747d", "#0b0b0b", "#e8eef7", "#121620"

# Flat colour makes a turning layer read as diamonds changing shape rather than
# a solid object turning: three faces of one colour are one colour, so there is
# nothing to say which way a sticker points. Shading by the sticker's own
# normal gives it back, and because the normal turns with the layer the shade
# slides through the turn instead of snapping at the end of it.
LIGHT = (0.29, 0.83, 0.50)

# Chosen so the opening frame lands as near the traced colours as a cube
# actually can: twenty-seven of the fifty-four places, where guessing gives
# nine. The drawing's own colouring is not a state any cube can be in.
SCRAMBLE = "U' L B2 L2 R D' U"
STEPS = 9          # samples per quarter turn

# A quarter turn takes STEPS * `default` seconds, and the dots ride it. They
# cover a long arc where the cube covers ten degrees, so the pace has to be set
# by the drawing rather than the cube: fast enough for the cube alone throws
# the dots across the page.
PACE = 0.065


@cm.trace()
def solve(state):
    """Open on a mixed cube and put it back, one sampled turn at a time."""
    cm.emit("settle")
    for move in cube.reverse(SCRAMBLE).split():
        # A prime move is one quarter turn the other way, not three this way.
        # The model only knows clockwise, so the state still takes three steps
        # — but the picture must not, or `U'` spins three quarters round while
        # a hand would have gone one quarter back.
        face, turns = move[0], {"": 1, "'": -1, "2": 2}[move[1:]]
        state["move"], state["face"], state["turns"] = move, face, turns
        steps = STEPS * abs(turns)
        for step in range(1, steps + 1):
            state["part"] = step / steps
            cm.emit("spin")
        for _ in range(turns % 4):
            # The turn also carries each sticker's corner list round, which
            # the view has to know or the reconciler spins every square in
            # place on the way to the next scene.
            carried = dict.fromkeys(range(54), 0)
            for facelet in geo.LAYER[face]:
                carried[state["where"][facelet]] = geo.SHIFT[face][facelet]
            state["order"] = [(k + carried[sticker]) % 4
                              for sticker, k in enumerate(state["order"])]
            state["where"] = cube.turn(state["where"], face)
        state["face"], state["part"], state["turns"] = None, 0.0, 0
        cm.emit("land")
    state["move"] = ""
    cm.emit("done")


def _along(here, there, middle, part, way=None):
    """One place to another, going round the circle rather than across it:
    the radius eases between, and `way` says which way the angle turns — +1,
    -1, or None to take the nearer way.

    The twelve on the circle must all be told, and all told the same. Left to
    pick for themselves, nine of them go one way and three go the other: the
    places are not evenly spaced, so for those three the three-place step
    spans more than half the circle and the nearer way is backwards. Twelve
    dots rotating and three coming to meet them is a swap, which is the thing
    the drawing exists not to do.

    The nine the turn also carries are not on the circle and have nowhere in
    particular to be, so they take the short way and swing about the centre.
    """
    cx, cy = middle
    ax, ay = graph.AT[here][0] - cx, graph.AT[here][1] - cy
    bx, by = graph.AT[there][0] - cx, graph.AT[there][1] - cy
    turn = math.atan2(by, bx) - math.atan2(ay, ax)
    swing = math.remainder(turn, math.tau) if way is None \
        else turn * way % math.tau * way
    angle = math.atan2(ay, ax) + swing * part
    reach = math.hypot(ax, ay) + (math.hypot(bx, by) - math.hypot(ax, ay)) * part
    return cx + reach * math.cos(angle), cy + reach * math.sin(angle)


def _shade(colour, normal):
    """`colour` as a face pointing `normal` would catch the light."""
    lit = max(0.0, sum(a * b for a, b in zip(normal, LIGHT)))
    part = 0.55 + 0.45 * lit
    return "#%02x%02x%02x" % tuple(
        min(255, round(int(colour[i:i + 2], 16) * part)) for i in (1, 3, 5))


def _order(facelet, face, degrees):
    """Painter's order for one facelet, far first.

    Sorting by the middle of each sticker is not enough on its own: half way
    through a turn a sticker's middle can come forward while the sticker is
    still behind the face it overlaps, and one lands on top of the cube like a
    sequin. There are two exact splits to make first.

    Which side of the cube a sticker is on is its normal's business — the far
    corner of a face you are looking straight at is further off than the middle
    of the cube and perfectly visible. And the turning layer is cut from the
    two that stay by a plane, so whichever side of it the camera is on is
    wholly in front of the other. Sorting by middles is left to settle the
    stickers *within* a group, where they share a plane and cannot disagree.
    """
    band = 2 if geo.facing(facelet, face, degrees) else 0
    if face:
        towards = sum(a * b for a, b in zip(geo.NORMAL[face], geo.EYE)) > 0
        band += (facelet in geo.LAYER[face]) == towards
    return band * 200 + 100 + round(geo.depth(facelet, face, degrees) * 8)


def view(frame):
    scene = cm.Scene()
    s = frame.state
    where, face, part = s["where"], s["face"], s["part"]
    turns = s["turns"]
    degrees = part * 90.0 * turns
    turned = where
    for _ in range(turns % 4):
        turned = cube.turn(turned, face)

    # Where each sticker sits now, and where this turn is taking it.
    now, then = [0] * 54, [0] * 54
    for facelet, sticker in enumerate(where):
        now[sticker] = facelet
    for facelet, sticker in enumerate(turned):
        then[sticker] = facelet

    # --- the drawing: nine circles and the lines between the places --------
    for c, (cx, cy) in enumerate(graph.CENTRES):
        for i, r in enumerate(graph.RADII):
            lit = face is not None and places.CIRCLE[face] == (c, i)
            scene.circle(("ring", c, i), r=r, at=(cx, cy)) \
                 .fill("none", edge="#7dd3fc" if lit else RING,
                       edge_w=2.6 if lit else 1.1) \
                 .on(layer=1, opacity=0.95 if lit else 0.5)
    for a, b in graph.EDGES:
        scene.line(("wire", a, b), start=graph.AT[a], end=graph.AT[b], w=1.2) \
             .fill(RING).on(layer=2, opacity=0.45)

    # --- the dots: one per sticker, sitting where that sticker now is ------
    middle = places.centre(face) if face else None
    on_circle = set(places.RING[places.CIRCLE[face]]) if face else set()
    for sticker in range(54):
        here, there = places.PLACE[now[sticker]], places.PLACE[then[sticker]]
        # The circle's own twelve are what the move is about, and are all sent
        # the same way round it; the nine it also carries are passengers, take
        # the short way, and step back so they are not read as part of the ring.
        riding = here in on_circle
        at = graph.AT[here] if here == there else \
            _along(here, there, middle, part,
                   places.SLIDE[face] * (1 if turns > 0 else -1)
                   if riding else None)
        # All twenty-one of the turning layer come forward, the other
        # thirty-three step back — the circle's twelve and the nine on its
        # face are one move, and the drawing should say so.
        active = face is not None and now[sticker] in geo.LAYER[face]
        scene.circle(("dot", sticker), r=9.5, at=at) \
             .fill(INK[cube.colour_of(sticker)], edge=SEAM, edge_w=1.3) \
             .on(layer=5 if riding else 4 if active else 3,
                 opacity=1.0 if active or face is None else 0.3)

    # --- the cube: facelets rotate, carrying whatever sits on them ----------
    # Every facelet, every frame. Leaving the far side undrawn makes stickers
    # enter and leave as they cross the horizon, and the reconciler fades
    # anything that enters or leaves — a cube full of ghosts part way through a
    # turn. So the far side is drawn too, in the dark of the cube's inside,
    # which is what you see down the gap a turn opens. It also tiles the far
    # side exactly, where one shape cut to the cube's outline would hang past
    # the edge the moment a layer swung away.
    #
    # Named by sticker, not by the facelet it is sitting on. A name is what the
    # reconciler follows, and a facelet does not move: at the end of a turn the
    # sticker is somewhere new, so a shape called after the facelet has to come
    # all the way back — which it did, rotating ninety degrees out and ninety
    # degrees home again on every move. Called after the sticker, the turn
    # finishes exactly where the next scene starts and nothing travels twice.
    for sticker in range(54):
        facelet = now[sticker]
        aim = geo.aim(facelet, face, degrees)
        near = sum(a * b for a, b in zip(aim, geo.EYE)) > 0.1
        pts = geo.quad(facelet, face, degrees)
        k = s["order"][sticker]
        scene.polygon(("sticker", sticker), pts[k:] + pts[:k]) \
             .fill(_shade(INK[cube.colour_of(sticker)], aim) if near else CORE,
                   edge=SEAM, edge_w=2.0) \
             .on(layer=_order(facelet, face, degrees))

    said = "a mixed cube" if not s["move"] else (
        "solved" if frame.is_("done") else "putting it back")
    scene.text("title", said, size=30, at=cm.at(x=640, top=44)).fill(PAPER)
    scene.text("move", s["move"], size=50,
               at=cm.at(x=640, bottom=666)).fill("#4ade80")
    return scene


cm.explain(
    trace=solve({"where": cube.apply(cube.solved(), SCRAMBLE),
                 "order": [0] * 54, "turns": 0,
                 "face": None, "part": 0.0, "move": ""}),
    view=view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=PACE, events={"land": 0.34, "settle": 1.4,
                                            "done": 2.4},
                     opening=0.8, final_hold=2.0),
).render("results/rubiks_cube.mp4", fps=60, scale=1.5)

print("wrote results/rubiks_cube.mp4")
