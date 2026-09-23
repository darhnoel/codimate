"""Archimedes' principle, from a box of water to a steel ship.

    python python/examples/archimedes/main.py

The whole film is one object. `("body", ...)` is a box of water, then ice, then
steel, then a ship's hull — never replaced, only changed — so the viewer's eye
carries from one scene to the next and the geometry cannot quietly differ
between them. That identity is Codimate's own: a name is what moves.

Nothing about the picture is hand-placed. `world.py` decides how deep things
float, how far the water rises, and how long every arrow is, from three real
densities; this file only says where to look and when.
"""

import codimate as cm

import world as W

cm.canvas(1280, 720)

# ------------------------------------------------------------ the palette
INK, DIM, FAINT = "#e8eef7", "#93a0b2", "#4a5666"
WATER, GLASS = "#2f7fb8", "#58697e"
UP, DOWN, HAND = "#38d6e0", "#ff7a59", "#f2c14e"     # buoyancy, weight, push
SKIN = {"water": "#4aa8dd", "ice": "#cfefff", "steel": "#96a2b0"}
SAYS = {"water": "WATER", "ice": "ICE", "steel": "STEEL"}

# ------------------------------------------------------------- the layout
OBJ_X = 0.5 * (W.TANK[0] + W.TANK[2])
NOTES_X = 1116.0                 # the column the tank never reaches into
TITLE_Y, SAY_Y = 44.0, 688.0

# Water is drawn *over* the objects, translucent, so anything below the
# surface is tinted without any clipping — the Engine has none.
BODY_LAYER, WATER_LAYER, MARK_LAYER, TEXT_LAYER = 20, 40, 60, 80


def _size(state):
    return ((W.HULL_W, W.HULL_H) if state["shape"] == "hull"
            else (W.BOX_W, W.BOX_H))


def _outline(state):
    """Where the object is, as `POINTS` points — a box or a hull, same count."""
    w, h = _size(state)
    middle = (OBJ_X, state["bottom"] - h / 2)
    return (W.hull_outline(middle) if state["shape"] == "hull"
            else W.box_outline(middle))


def create_water_tank(scene, level, marked):
    """Glass, water, and — when asked — how far the surface has climbed."""
    left, top, right, bottom = W.TANK
    for i, (a, b) in enumerate((((left, top), (left, bottom)),
                                ((left, bottom), (right, bottom)),
                                ((right, bottom), (right, top)))):
        scene.line(("glass", i), start=a, end=b, w=3.0).fill(GLASS) \
             .on(layer=MARK_LAYER)

    scene.rect("water", w=right - left, h=max(bottom - level, 0.0),
               at=(0.5 * (left + right), 0.5 * (level + bottom))) \
         .fill(WATER).on(layer=WATER_LAYER, opacity=0.46)
    scene.line("surface", start=(left, level), end=(right, level), w=2.4) \
         .fill("#7fd4f5").on(layer=WATER_LAYER + 1)

    if not marked:
        return

    # Where it started, as a dashed line. There are no dashes in the Engine,
    # so the dashes are short lines — a fixed number of them, so the rule
    # stretches rather than gaining and losing pieces.
    for i in range(16):
        a = left + (right - left) * i / 16
        scene.line(("was", i), start=(a + 6, W.REST_LEVEL),
                   end=(a + (right - left) / 16 - 6, W.REST_LEVEL), w=2.0) \
             .fill(DIM).on(layer=WATER_LAYER + 2, opacity=0.75)

    # And the climb itself, bracketed outside the glass where nothing else is.
    if W.REST_LEVEL - level > 2.0:
        x = left - 34
        scene.line("rise", start=(x, level), end=(x, W.REST_LEVEL), w=3.0) \
             .fill(UP).on(layer=MARK_LAYER + 2)
        for k, y in enumerate((level, W.REST_LEVEL)):
            scene.line(("rise", k), start=(x - 10, y), end=(x + 10, y), w=3.0) \
                 .fill(UP).on(layer=MARK_LAYER + 2)
        for k, words in enumerate(("water displaced", "= V")):
            scene.text(("rise", "word", k), words, size=18,
                       at=cm.at(x=x - 112, y=W.REST_LEVEL - 24 + 24 * k)) \
                 .fill(UP).on(layer=TEXT_LAYER)


def create_force_arrow(scene, name, at, length, look):
    """An arrow whose length *is* the force, and its label.

    `look` is `(colour, words, points_up, capped)` in one argument rather than
    four,
    because nothing in this library takes more than five things — the shapes
    grew handles for the same reason.

    One scale for the whole film (`world.arrow_length`), so the arrows can be
    compared across scenes rather than only within one.
    """
    colour, words, up, capped = look
    if length < 1.0:
        return
    tip = (at[0], at[1] - length if up else at[1] + length)
    scene.arrow(name, start=at, end=tip, w=7.0, head=19.0).fill(colour) \
         .on(layer=MARK_LAYER + 2)
    if capped:
        # The break an axis gets when it is cut short, rather than the words
        # "off scale" — which started on top of the arrow they described.
        mid = tip[1] + (34 if up else -34)
        for k in (0, 10):
            scene.line((name, "cut", k), start=(at[0] - 13, mid + k + 7),
                       end=(at[0] + 13, mid + k - 7), w=3.0) \
                 .fill("#0b0f16").on(layer=MARK_LAYER + 3)
    # Always above the tip. Below a downward arrow is where the caption
    # lives, and a long weight arrow put its label straight through it.
    scene.text((name, "word"), words, size=19,
               at=cm.at(x=at[0] + 78, y=tip[1] - 18)) \
         .fill(colour).on(layer=TEXT_LAYER)


def animate_force_balance(scene, state, level):
    """Up, down, and the hand — each drawn only when the scene wants it."""
    w, h = _size(state)
    _, sub = W.surface(state["bottom"], w, h)
    middle = (OBJ_X, state["bottom"] - h / 2)
    rho = {"water": W.RHO_WATER, "ice": W.RHO_ICE,
           "steel": W.RHO_STEEL}[state["material"]]

    if state["shape"] == "hull":
        # A hull carries the steel it was made from, spread over its outside:
        # what floats it is the average, not the metal.
        push = W.buoyancy(w * h * sub)
        pull = W.weight(W.RHO_STEEL, W.BOX_AREA)
    else:
        push = W.buoyancy(w * h * sub)
        pull = W.weight(rho, w * h)

    if "up" in state["arrows"]:
        length, capped = W.arrow_length(push)
        create_force_arrow(scene, "fb", middle, length,
                           (UP, "F_B", True, capped))
    if "down" in state["arrows"]:
        length, capped = W.arrow_length(pull)
        create_force_arrow(scene, "wt", middle, length,
                           (DOWN, "W", False, capped))
    if "hand" in state["arrows"]:
        top = (OBJ_X, middle[1] - h / 2 - 8)
        scene.arrow("hand", start=(top[0], top[1] - 96), end=top,
                    w=7.0, head=19.0).fill(HAND).on(layer=MARK_LAYER + 2)
        # Left of the shaft: the buoyancy label already has the right side.
        scene.text(("hand", "word"), "hand", size=19,
                   at=cm.at(x=top[0] - 78, y=top[1] - 96)).fill(HAND) \
             .on(layer=TEXT_LAYER)

    # An 8.3% difference is real and nearly invisible, so the gap between the
    # two arrow tips gets its own mark rather than the arrows being fudged.
    if state["net"]:
        up_px, _ = W.arrow_length(push)
        down_px, _ = W.arrow_length(pull)
        if abs(up_px - down_px) > 2.0:
            x = OBJ_X - 92
            a, b = middle[1] - up_px, middle[1] - down_px
            scene.line("net", start=(x, a), end=(x, b), w=3.0).fill(UP) \
                 .on(layer=MARK_LAYER + 2)
            scene.text(("net", "word"), "net push up", size=17,
                       at=cm.at(x=x - 66, y=0.5 * (a + b))).fill(UP) \
                 .on(layer=TEXT_LAYER)


def set_submerged_bracket(scene, state, level):
    """The bracket that says how much of the ice is under the water."""
    w, h = _size(state)
    _, sub = W.surface(state["bottom"], w, h)
    x = OBJ_X + w / 2 + 26
    for i, (a, b, colour, words) in enumerate((
            (level, state["bottom"], UP, f"{sub:.1%} under"),
            (state["bottom"] - h, level, DIM, f"{1 - sub:.1%} above"))):
        if b - a < 6:
            continue
        scene.line(("brace", i), start=(x, a), end=(x, b), w=3.0) \
             .fill(colour).on(layer=MARK_LAYER + 2)
        for k, y in enumerate((a, b)):
            scene.line(("brace", i, k), start=(x - 9, y), end=(x + 9, y),
                       w=3.0).fill(colour).on(layer=MARK_LAYER + 2)
        # Outside the bracket normally; inside it for the ship, whose right
        # edge is so near the glass that the label would land on the beaker.
        scene.text(("brace", i, "word"), words, size=19,
                   at=cm.at(x=x + 82, y=0.5 * (a + b))).fill(colour) \
             .on(layer=TEXT_LAYER)


def create_density_comparison(scene, state):
    """Equal volumes, unequal mass — two swatches and their densities."""
    pairs = state["compare"]
    for i, (words, rho, colour) in enumerate(pairs):
        y = 236.0 + 128.0 * i
        scene.rect(("cmp", i), w=92, h=72, at=(NOTES_X - 74, y)) \
             .fill(colour).round(6).on(layer=MARK_LAYER)
        scene.text(("cmp", i, "name"), words, size=19,
                   at=cm.at(x=NOTES_X + 66, y=y - 16)).fill(INK) \
             .on(layer=TEXT_LAYER)
        scene.text(("cmp", i, "rho"), f"{rho:,.0f} kg/m3", size=19,
                   at=cm.at(x=NOTES_X + 66, y=y + 14)).fill(colour) \
             .on(layer=TEXT_LAYER)


def create_equation_step(scene, state):
    """The derivation, one line at a time. Never all at once."""
    lines = (r"\rho_w V_{sub}\, g \;=\; \rho_i V_{ice}\, g",
             r"\rho_w V_{sub} \;=\; \rho_i V_{ice}",
             r"\frac{V_{sub}}{V_{ice}} \;=\; \frac{\rho_i}{\rho_w}",
             r"=\; \frac{917}{1000} \;=\; 0.917")
    for i in range(state["steps"]):
        scene.formula(("step", i), lines[i], size=24,
                      at=cm.at(x=NOTES_X, y=232.0 + 74.0 * i)).fill(INK) \
             .on(layer=TEXT_LAYER)


def view(frame):
    scene = cm.Scene()
    state = frame.state
    w, h = _size(state)
    level, sub = W.surface(state["bottom"], w, h)

    scene.text("title", state["title"], size=32, at=cm.at(x=640, top=TITLE_Y)) \
         .fill(INK).on(layer=TEXT_LAYER)
    if state["say"]:
        scene.text("say", state["say"], size=21,
                   at=cm.at(x=640, bottom=SAY_Y)).fill(DIM).on(layer=TEXT_LAYER)

    if state["stage"]:
        create_water_tank(scene, level, state["rise"])

        # One name for the whole film. Water becomes ice becomes steel becomes
        # a ship, and because the name never changes the Engine tweens between
        # them instead of cutting.
        skin = SKIN[state["material"]]
        scene.polygon("body", _outline(state)).fill(skin, edge=INK, edge_w=2.0) \
             .on(layer=BODY_LAYER)
        if state["label"]:
            # Centred, with a plate behind it. The force arrows share this
            # centre line, so without one the label is drawn straight through.
            words = SAYS[state["material"]]
            wide, high = cm.measure(words, size=20)
            spot = (OBJ_X, state["bottom"] - h / 2)
            # Above the arrows, not level with them. On the same layer the
            # plate was drawn first and the arrow straight over the top of it,
            # which is the whole thing the plate exists to prevent.
            scene.rect("body_plate", w=wide + 22, h=high + 10, at=spot) \
                 .fill("#0b1018").round(5).on(layer=TEXT_LAYER - 2, opacity=0.92)
            scene.text("body_word", words, size=20, at=spot) \
                 .fill(INK).on(layer=TEXT_LAYER - 1)

        animate_force_balance(scene, state, level)
        if state["brace"]:
            set_submerged_bracket(scene, state, level)

    if state["compare"]:
        create_density_comparison(scene, state)
    if state["steps"]:
        create_equation_step(scene, state)

    if state["big"]:
        # Below the glass. At mid-frame it lands straight across the ice it is
        # describing, which is the one thing it must not cover.
        scene.text("big", state["big"], size=46, at=cm.at(x=640, y=672)) \
             .fill(UP).on(layer=TEXT_LAYER + 5)
    if state["principle"]:
        scene.formula("law", r"F_B \;=\; \rho_{fluid}\, V_{displaced}\, g",
                      size=52, at=cm.at(x=640, y=300)).fill(INK) \
             .on(layer=TEXT_LAYER + 5)

    for i, words in enumerate(state["summary"]):
        scene.text(("sum", i), words, size=25,
                   at=cm.at(x=640, top=228.0 + 62.0 * i)).fill(INK) \
             .on(layer=TEXT_LAYER)
    return scene


OPENING = {
    "title": "A steel bolt sinks. A steel ship floats. Why?",
    "say": "", "stage": True, "shape": "box", "material": "steel",
    "bottom": 250.0, "label": False, "arrows": (), "hand": False,
    "net": False, "brace": False, "rise": False, "compare": (),
    "steps": 0, "big": "", "principle": False, "summary": (),
}


@cm.trace()
def story(state):
    """Every section is a short beat that changes the picture, then a long one
    that holds it.

    A shape entering or leaving a Scene fades, and the fade takes the *whole*
    beat — so a panel appearing at the top of a six-second section would spend
    six seconds arriving. Splitting change from hold puts the fade in a quarter
    second and lets the section simply sit there.
    """

    def beat(name, **change):
        state.update(change)
        cm.emit("swap")
        cm.emit(name)

    def glide(target, steps=26, name="move"):
        """Move the object to `target`, sampled.

        Continuous motion has to be handed over a step at a time. Set the
        position in one beat instead and it tweens in a straight line — the
        object would slide to its new depth while the water it displaces
        jumped there, and the two would visibly disagree.
        """
        was = state["bottom"]
        for k in range(1, steps + 1):
            state["bottom"] = was + (target - was) * k / steps
            cm.emit(name)

    # 1. The hook. A steel block falls in and keeps going.
    cm.emit("hook")
    glide(W.TANK[3] - W.BOX_H / 2 + 40, steps=30, name="sink")
    beat("hook2", say="Same metal. Opposite answers.")

    # 2. Simplify: one box, volume V, held above the water.
    beat("simpler", title="Start simpler", say="One box. Volume V.",
         material="water", label=True, bottom=300.0)

    # 3. Make it a box *of water*, and lower it in.
    beat("waterbox", title="A box of water, in water", say="")
    glide(W.water_level(W.BOX_AREA) + W.BOX_H, steps=30, name="dip")
    beat("displace", say="The water climbs by exactly the volume that went in.",
         rise=True)
    beat("neutral", title="It neither rises nor sinks",
         say="The water it displaces weighs what the box weighs.",
         arrows=("up", "down"))

    # 4. Same box, different stuff.
    beat("ice", title="Now make it ice", say="Same box. Same volume V.",
         material="ice", arrows=(),
         compare=(("water", W.RHO_WATER, SKIN["water"]),
                  ("ice", W.RHO_ICE, SKIN["ice"])))
    beat("lighter", say="Same volume, less mass.")

    # 5. Hold it under, and look at the two forces.
    beat("push", title="Hold it under", say="", compare=(),
         arrows=("up", "down", "hand"), net=True)
    beat("more", say="Full volume displaced, but less weight to carry.")

    # 6. Let go. It rises; the push stays put until it breaks the surface.
    beat("release", title="Let go", say="", arrows=("up", "down"))
    glide(W.floating_bottom(W.ICE_SUBMERGED, W.BOX_W, W.BOX_H), steps=34,
          name="rise")
    beat("shrink", say="As it leaves the water, it displaces less — "
                       "so the push up falls.")

    # 7. The payoff, and only then the algebra.
    beat("stop", title="It stops here", say="", net=False, brace=True)
    beat("why", say="Not a random height: the depth where it displaces "
                    "its own weight.")
    for n in (1, 2, 3, 4):
        beat("derive", steps=n, say="")
    beat("percent", big=f"{W.ICE_SUBMERGED:.1%} SUBMERGED", steps=0,
         say="", brace=True)

    # 8. Steel, in exactly the same box.
    beat("steel", title="Now make it steel", say="Same box. Same volume V.",
         material="steel", big="", brace=False, arrows=(),
         compare=(("water", W.RHO_WATER, SKIN["water"]),
                  ("steel", W.RHO_STEEL, SKIN["steel"])))
    # Under the water first, then the arrows, then the arrows *off* before it
    # drops. Left on, a weight arrow this long reaches out of the frame and
    # through the caption once the block is resting on the floor.
    glide(W.REST_LEVEL + W.BOX_H + 50.0, steps=14, name="dip")
    beat("steelforce", say="Even fully under, it cannot displace enough.",
         compare=(), arrows=("up", "down"))
    beat("sinks", say="So down it goes.", arrows=())
    glide(W.TANK[3] - W.BOX_H / 2 - 2.0, steps=22, name="sink")

    # 9. The reveal: the same steel, spread out.
    #
    # The reshape gets a beat of its own, and a long one. Changed inside a
    # quarter-second swap with everything else, the box tweens into a hull too
    # fast to read — it reads as a glitch rather than as the answer.
    beat("question", title="Then how can a steel ship float?", say="",
         arrows=())
    state.update(shape="hull", label=False, bottom=W.REST_LEVEL - 30.0)
    cm.emit("morph")
    beat("ship", title="Same steel, spread out", say="")
    glide(W.floating_bottom(W.SHIP_SUBMERGED, W.HULL_W, W.HULL_H), steps=28,
          name="settle")
    beat("ships", say="The steel never changed. The outside got "
                      f"{W.SPREAD:.0f} times bigger.",
         arrows=("up", "down"), brace=True)
    beat("average", title="Steel and air together",
         say=f"Average density {W.SHIP_RHO:,.0f} kg/m3 — "
             f"lighter than water, so it floats.")

    # 10. Four cases, the law, the name.
    beat("summary", title="", say="", stage=False, brace=False, arrows=(),
         summary=("ice          917  <  1000      floats",
                  "water box   1000  =  1000      neutral",
                  "steel block 7850  >  1000      sinks",
                  f"steel ship   {W.SHIP_RHO:,.0f}  <  1000      floats"))
    beat("law", title="", summary=(), principle=True)
    beat("said", say="A thing floats when it can displace its own weight "
                     "before it is all the way under.")
    beat("name", title="Archimedes' Principle", say="", principle=False,
         big="")


cm.explain(
    trace=story(dict(OPENING)),
    view=view,
    # Everything that moves was already sampled by the trace, so it must not be
    # eased a second time at every step.
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(
        default=0.1,
        events={"swap": 0.26,
                "hook": 2.0, "sink": 0.05, "hook2": 2.0,
                "simpler": 2.4, "waterbox": 0.8, "dip": 0.05,
                "displace": 2.4, "neutral": 3.0,
                "ice": 2.8, "lighter": 2.4,
                "push": 3.0, "more": 2.8,
                "release": 0.8, "rise": 0.045, "shrink": 2.8,
                "stop": 2.2, "why": 2.6, "derive": 1.7, "percent": 3.4,
                "steel": 2.6, "steelforce": 3.0, "sinks": 1.8,
                "question": 2.4, "morph": 2.0, "ship": 1.6,
                "settle": 0.05, "ships": 3.0, "average": 3.2,
                "summary": 4.2, "law": 3.0, "said": 3.0, "name": 2.4},
        opening=0.9, final_hold=1.8),
).render("results/archimedes.mp4", fps=60, scale=1.5)

print("wrote results/archimedes.mp4")
