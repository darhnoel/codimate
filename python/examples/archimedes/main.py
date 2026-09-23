"""Archimedes' principle, from a box of water to a steel ship.

    python python/examples/archimedes/main.py

The whole film is one object. `"body"` is a box of water, then ice, then steel,
then a ship's hull — never replaced, only changed — so the viewer's eye carries
from one scene to the next and the geometry cannot quietly differ between them.
That identity is Codimate's own: a name is what moves.

Nothing about the picture is hand-placed. `world.py` decides how deep things
float, how far the water rises, and how long every arrow is, from three real
densities; this file only says where to look and when.
"""

import codimate as cm

import world as W

cm.canvas(1280, 720)

# ------------------------------------------------------------ the palette
INK, DIM, AIR = "#e8eef7", "#93a0b2", "#000000"
WATER, GLASS = "#2f7fb8", "#58697e"
UP, DOWN, HAND = "#38d6e0", "#ff7a59", "#f2c14e"     # buoyancy, weight, push
PLATE = "#0b1018"
SKIN = {"water": "#4aa8dd", "ice": "#cfefff", "steel": "#96a2b0"}
SAYS = {"water": "WATER", "ice": "ICE", "steel": "STEEL"}

# ------------------------------------------------------------- the layout
OBJ_X = 0.5 * (W.BEAKER[0] + W.BEAKER[2])       # both pools share this line
NOTES_X = 1116.0                 # the column the tank never reaches into
TITLE_Y, SAY_Y = 44.0, 694.0

# Both marks live outside the glass: the displaced volume down the left wall,
# how much of the object is under down the right. Nothing is drawn over the
# water, the glass or the object itself. They move with the glass, because the
# glass moves — the beaker grows into a basin when the ship is built.
MARK_GAP = 34.0

# Water is drawn *over* the objects, translucent, so anything below the
# surface is tinted without any clipping — the Engine has none. The hull's
# cavity is then drawn over the water for the same reason, in the frame's own
# black, so the inside of the ship reads as air rather than as flood.
BODY_LAYER, WATER_LAYER, CAVITY_LAYER = 20, 40, 50
MARK_LAYER, TEXT_LAYER, LABEL_LAYER = 60, 80, 95


def _toward(was, target, along):
    """One step of the way from `was` to `target`, numbers or pools alike.

    The pool is five numbers rather than one, and it has to move the same way
    everything else does — a step at a time — or the tank would jump from
    beaker to basin while the block slid.
    """
    if isinstance(target, tuple):
        return tuple(a + (b - a) * along for a, b in zip(was, target))
    return was + (target - was) * along


def _middle(state):
    return (OBJ_X, state["bottom"] - state["h"] / 2)


def create_water_tank(scene, pool, level, mark):
    """Glass, water, and — when `mark` is given — how far the surface climbed.

    `mark` is the multiple of V that has been displaced, because "= V" was
    true only of the fully sunk water box: the ice displaces 0.917 V and the
    ship nearly eight.
    """
    left, top, right, bottom, rest = pool
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

    if mark is None:
        return

    # Where it started, as a dashed line. There are no dashes in the Engine,
    # so the dashes are short lines — a fixed number of them, so the rule
    # stretches rather than gaining and losing pieces.
    for i in range(16):
        a = left + (right - left) * i / 16
        scene.line(("was", i), start=(a + 6, rest),
                   end=(a + (right - left) / 16 - 6, rest), w=2.0) \
             .fill(DIM).on(layer=WATER_LAYER + 2, opacity=0.75)

    if rest - level > 2.0:
        set_bracket(scene, "rise", (left - MARK_GAP, level, rest),
                    (UP, -1), ("water displaced", f"= {mark:.3g} V"))


def set_bracket(scene, name, span, look, words):
    """A measured span outside the tank, with its label on a dark plate.

    `look` is `(colour, side)` and `span` is `(x, top, bottom)`, packed so no
    call takes more than five things — the rule the whole library runs on.
    """
    x, top, bottom = span
    colour, side = look
    scene.line(name, start=(x, top), end=(x, bottom), w=3.0).fill(colour) \
         .on(layer=MARK_LAYER + 2)
    for k, y in enumerate((top, bottom)):
        scene.line((name, k), start=(x - 10, y), end=(x + 10, y), w=3.0) \
             .fill(colour).on(layer=MARK_LAYER + 2)

    middle = 0.5 * (top + bottom)
    for k, line in enumerate(words):
        wide, high = cm.measure(line, size=18)
        spot = (x + side * (26 + wide / 2), middle - 13 + 26 * k)
        scene.rect((name, "plate", k), w=wide + 16, h=high + 8, at=spot) \
             .fill(PLATE).round(4).on(layer=LABEL_LAYER - 1, opacity=0.92)
        scene.text((name, "word", k), line, size=18, at=spot).fill(colour) \
             .on(layer=LABEL_LAYER)


def create_force_arrow(scene, name, at, length, look):
    """An arrow whose length *is* the force, and its label.

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


def animate_force_balance(scene, state):
    """Up, down, and the hand — each drawn only when the scene wants it.

    The steel never changes amount, so the weight is always `rho x V` of the
    original block: that is the whole argument of the last section, and it
    would be lost if the hull's outer area were weighed instead.
    """
    w, h = state["w"], state["h"]
    _, sub = W.surface(state["bottom"], w, h, state["pool"])
    middle = _middle(state)
    rho = {"water": W.RHO_WATER, "ice": W.RHO_ICE,
           "steel": W.RHO_STEEL}[state["material"]]
    push = W.buoyancy(w * h * sub)
    pull = W.weight(rho, W.BOX_AREA)

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
    """How much of the object is under the water — outside the right wall.

    Beside the object it sat on the surface line, on the glass, and for the
    hull almost in the notes column. Out here it mirrors the displaced-volume
    bracket on the left and can never cover anything.
    """
    _, sub = W.surface(state["bottom"], state["w"], state["h"], state["pool"])
    x = state["pool"][2] + MARK_GAP
    for i, (a, b, colour, words) in enumerate((
            (level, state["bottom"], UP, f"{sub:.1%} under"),
            (state["bottom"] - state["h"], level, DIM, f"{1 - sub:.1%} above"))):
        if b - a >= 6:
            set_bracket(scene, ("brace", i), (x, a, b), (colour, 1), (words,))


def create_density_comparison(scene, state):
    """Equal volumes, unequal mass — two swatches and their densities."""
    for i, (words, rho, colour) in enumerate(state["compare"]):
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


def create_body(scene, state):
    """The one object, wherever it is in its life.

    Rounded at every corner, including the hull's, because the rounding lives
    in the outline itself — a rounded rect could not morph.
    """
    w, h, notch = state["w"], state["h"], state["notch"]
    middle = _middle(state)
    scene.polygon("body", W.outline(middle, w, h, notch)) \
         .fill(SKIN[state["material"]], edge=INK, edge_w=2.0) \
         .on(layer=BODY_LAYER)

    if notch > 2.0:
        left, top, right, bottom = W.cavity(middle, w, h, notch)
        scene.rect("hold", w=right - left, h=bottom - top,
                   at=(0.5 * (left + right), 0.5 * (top + bottom))) \
             .fill(AIR).on(layer=CAVITY_LAYER)

    if not state["label"]:
        return
    # Above everything, on a plate. The force arrows run up and down this same
    # centre line, so without both the label is drawn straight through.
    words = SAYS[state["material"]]
    wide, high = cm.measure(words, size=20)
    scene.rect("body_plate", w=wide + 22, h=high + 10, at=middle) \
         .fill(PLATE).round(5).on(layer=LABEL_LAYER - 1, opacity=0.92)
    scene.text("body_word", words, size=20, at=middle).fill(INK) \
         .on(layer=LABEL_LAYER)


CARD = "Archimedes' Principle"
CARD_SIZE = 58
CARD_Y = 330.0


def create_title_card(scene, state):
    """The name, one letter at a time, then a rule drawn under it.

    Each letter is its own shape with its own name, because that is the only
    way the Engine can be told that one of them arrived and the others did
    not. Laid out by measuring the prefix before it, so the spacing is the
    font's rather than a guess.
    """
    wide, _ = cm.measure(CARD, size=CARD_SIZE)
    left = 640.0 - wide / 2
    for i in range(min(state["letters"], len(CARD))):
        if CARD[i] == " ":
            continue
        before, _ = cm.measure(CARD[:i], size=CARD_SIZE)
        step, _ = cm.measure(CARD[i], size=CARD_SIZE)
        scene.text(("card", i), CARD[i], size=CARD_SIZE,
                   at=cm.at(x=left + before + step / 2, y=CARD_Y)) \
             .fill(INK).on(layer=TEXT_LAYER + 5)

    # Out from the middle, so it reads as being drawn rather than appearing.
    half = 0.5 * wide * state["rule"]
    if half > 1.0:
        scene.line("card_rule", start=(640 - half, CARD_Y + 48),
                   end=(640 + half, CARD_Y + 48), w=2.5) \
             .fill(UP).on(layer=TEXT_LAYER + 5, opacity=0.85)


# The four cases, on one scale. Steel is eight times off the end of it, which
# is the point: it is the only row that needs the axis broken.
VERDICTS = (("ice", W.RHO_ICE, SKIN["ice"], "floats"),
            ("water", W.RHO_WATER, SKIN["water"], "neutral"),
            ("steel ship", W.SHIP_RHO, SKIN["steel"], "floats"),
            ("steel block", W.RHO_STEEL, SKIN["steel"], "sinks"))
# Placed so the whole block — names, bars, values, verdicts — is centred on
# the frame rather than the bars alone being centred.
BAR_X, BAR_PER, BAR_MAX = 396.0, 0.30, 420.0     # px per kg/m3, and the cut
VERDICT_INK = {"floats": UP, "neutral": DIM, "sinks": DOWN}


def create_verdict_chart(scene, state):
    """Four densities against the one that decides: water's.

    A bar chart rather than four lines of text, because "floats" and "sinks"
    are not four separate facts — they are which side of 1,000 each bar ends
    on, and a bar says that without being read.
    """
    top, step = 276.0, 84.0
    line = BAR_X + W.RHO_WATER * BAR_PER
    deep = top + step * (len(VERDICTS) - 1) + 40
    scene.line("mark", start=(line, top - 46), end=(line, deep), w=2.0) \
         .fill(WATER).on(layer=MARK_LAYER, opacity=0.9)
    scene.text("mark_word", "water  1,000 kg/m3", size=18,
               at=cm.at(x=line + 104, y=top - 62)).fill(WATER) \
         .on(layer=TEXT_LAYER)

    for i, (words, rho, colour, verdict) in enumerate(VERDICTS[:state["rows"]]):
        y = top + step * i
        # Only the newest bar grows; the ones above it are already there.
        share = state["grow"] if i == state["rows"] - 1 else 1.0
        full = min(rho * BAR_PER, BAR_MAX)
        create_verdict_row(scene, i, (y, full * share, colour),
                           (words, f"{rho:,.0f}", verdict))
        if full >= BAR_MAX and share > 0.98:
            for k in (0, 11):
                scene.line(("cut", i, k),
                           start=(BAR_X + BAR_MAX - 20 + k, y - 20),
                           end=(BAR_X + BAR_MAX - 6 + k, y + 20), w=3.0) \
                     .fill(AIR).on(layer=MARK_LAYER + 2)


def create_verdict_row(scene, i, bar, words):
    """One labelled bar: what it is, how dense, and what it does."""
    y, length, colour = bar
    name, rho, verdict = words
    scene.rect(("bar", i), w=max(length, 1.0), h=34,
               at=(BAR_X + length / 2, y)).fill(colour).round(4) \
         .on(layer=MARK_LAYER - 1, opacity=0.9)
    # `cm.at` centres, and these want their right edges lined up against the
    # bars — so the width is measured and half of it taken off.
    wide, _ = cm.measure(name, size=21)
    scene.text(("bar", i, "name"), name, size=21,
               at=cm.at(x=BAR_X - 26 - wide / 2, y=y)).fill(INK) \
         .on(layer=TEXT_LAYER)
    # Clear of the water line, whatever the bar does: three of the four end
    # within a few pixels of it, and a value sitting on the line is unreadable.
    line = BAR_X + W.RHO_WATER * BAR_PER
    scene.text(("bar", i, "rho"), rho, size=20,
               at=cm.at(x=max(BAR_X + min(length, BAR_MAX) + 54, line + 58),
                        y=y)).fill(colour).on(layer=TEXT_LAYER)
    scene.text(("bar", i, "says"), verdict, size=21,
               at=cm.at(x=996, y=y)).fill(VERDICT_INK[verdict]) \
         .on(layer=TEXT_LAYER)


def view(frame):
    scene = cm.Scene()
    state = frame.state
    pool = state["pool"]
    level, _ = W.surface(state["bottom"], state["w"], state["h"], pool)

    scene.text("title", state["title"], size=32, at=cm.at(x=640, top=TITLE_Y)) \
         .fill(INK).on(layer=TEXT_LAYER)
    if state["say"]:
        scene.text("say", state["say"], size=21,
                   at=cm.at(x=640, bottom=SAY_Y)).fill(DIM).on(layer=TEXT_LAYER)

    if state["stage"]:
        mark = None
        if state["rise"]:
            mark = W.displaced(state["bottom"], state["w"], state["h"],
                               pool) / W.BOX_AREA
        create_water_tank(scene, pool, level, mark)
        create_body(scene, state)
        animate_force_balance(scene, state)
        if state["brace"]:
            set_submerged_bracket(scene, state, level)

    if state["compare"]:
        create_density_comparison(scene, state)
    if state["steps"]:
        create_equation_step(scene, state)

    if state["letters"]:
        create_title_card(scene, state)

    if state["big"]:
        scene.text("big", state["big"], size=46, at=cm.at(x=640, y=686)) \
             .fill(UP).on(layer=TEXT_LAYER + 5)
    if state["principle"]:
        scene.formula("law", r"F_B \;=\; \rho_{fluid}\, V_{displaced}\, g",
                      size=52, at=cm.at(x=640, y=300)).fill(INK) \
             .on(layer=TEXT_LAYER + 5)

    if state["rows"]:
        create_verdict_chart(scene, state)
    return scene


OPENING = {
    "title": "", "letters": 0, "rule": 0.0,
    "say": "", "stage": False, "material": "steel",
    "w": W.BOX_W, "h": W.BOX_H, "notch": 0.0,
    "bottom": 250.0, "pool": W.BEAKER, "label": False, "arrows": (),
    "net": False, "brace": False, "rise": False, "compare": (),
    "steps": 0, "big": "", "principle": False, "rows": 0, "grow": 0.0,
}

# Clear of the water, where the ship is built, and where it ends up.
HELD_UP = W.BASIN[4] - 8.0
AFLOAT = W.floating_bottom(W.SHIP_SUBMERGED, W.HULL_W, W.HULL_H, W.BASIN)


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

    def walk(name, steps=26, **targets):
        """Move every named value to its target, sampled.

        Continuous change has to be handed over a step at a time. Set it in
        one beat instead and it tweens in a straight line — the object would
        slide to its new depth while the water it displaces jumped there, and
        the box would cross to the hull through shapes that are neither.
        """
        was = {k: state[k] for k in targets}
        for i in range(1, steps + 1):
            for k, target in targets.items():
                state[k] = _toward(was[k], target, i / steps)
            cm.emit(name)

    # 0. The name first, typed on, then underlined. Nothing else is on
    #    screen, so the film opens on it rather than fading up to it.
    cm.emit("card")
    for i in range(1, len(CARD) + 1):
        state["letters"] = i
        cm.emit("type")
    walk("underline", steps=16, rule=1.0)
    cm.emit("held")

    # 1. The hook. A steel block falls in and keeps going.
    beat("hook", letters=0, rule=0.0, stage=True,
         title="A steel bolt sinks. A steel ship floats. Why?")
    walk("sink", steps=32, bottom=W.floor(W.BEAKER))
    beat("hook2", say="Same metal. Opposite answers.")

    # 2. Simplify: one box, volume V, held above the water.
    beat("simpler", title="Start simpler", say="One box. Volume V.",
         material="water", label=True, bottom=250.0)

    # 3. Make it a box *of water*, and lower it in.
    beat("waterbox", title="A box of water, in water", say="")
    walk("dip", steps=32,
         bottom=W.water_level(W.BOX_AREA, W.BEAKER) + W.BOX_H)
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
    walk("rise", steps=34,
         bottom=W.floating_bottom(W.ICE_SUBMERGED, W.BOX_W, W.BOX_H,
                                  W.BEAKER))
    beat("shrink", say="As it leaves the water, it displaces less — "
                       "so the push up falls.")

    # 7. The payoff, and only then the algebra.
    beat("stop", title="It stops here", say="", net=False, brace=True)
    beat("why", say="Not a random height: the depth where it displaces "
                    "its own weight.")
    # The brace comes off while the algebra is on screen: both want the right
    # hand side of the frame, and neither needs the other to be readable.
    for n in (1, 2, 3, 4):
        beat("derive", steps=n, say="", brace=False)
    beat("percent", big=f"{W.ICE_SUBMERGED:.1%} SUBMERGED", steps=0,
         say="", brace=True)

    # 8. Steel, in exactly the same box.
    beat("steel", title="Now make it steel", say="Same box. Same volume V.",
         material="steel", big="", brace=False, arrows=(),
         compare=(("water", W.RHO_WATER, SKIN["water"]),
                  ("steel", W.RHO_STEEL, SKIN["steel"])))
    walk("dip", steps=16, bottom=W.BEAKER[4] + W.BOX_H + 50.0)
    beat("steelforce", say="Even fully under, it cannot displace enough.",
         compare=(), arrows=("up", "down"))
    # The arrows go *before* it drops. Left on, a weight arrow this long
    # reaches out of the frame once the block is resting on the floor.
    beat("sinks", say="So down it goes.", arrows=())
    walk("sink", steps=24, bottom=W.floor(W.BEAKER))

    # 9. The reveal: the same steel, spread out. Four sampled stages, and all
    #    of the reshaping happens in the air — a solid slab that wide would
    #    sink, so doing it in the water would be showing something false.
    beat("question", title="Then how can a steel ship float?", say="",
         arrows=(), label=False)
    # A bigger tank first. Nothing in Archimedes cares what the water is held
    # in, and a beaker the box fills cannot also hold a hull nine times its
    # area — so the glass grows, sampled, and the block rides the floor down.
    beat("bigger", say="A bigger tank — the same water.")
    walk("grow", steps=26, pool=W.BASIN, bottom=W.floor(W.BASIN))
    walk("lift", steps=26, bottom=HELD_UP)
    beat("spreadsay", title="", say="The same steel, spread out...")
    walk("spread", steps=26, w=W.HULL_W, h=W.HULL_H)
    beat("hollowsay", say="...and hollowed out. Not one gram more or less.")
    walk("hollow", steps=22, notch=W.HULL_H - W.HULL_T)
    beat("lowersay", title="Same steel, spread out", say="")
    walk("settle", steps=30, bottom=AFLOAT)
    beat("ships", say="The steel never changed. The outside got "
                      f"{W.SPREAD:.0f} times bigger.",
         arrows=("up", "down"), brace=True, rise=True)
    beat("average", title="Steel and air together",
         say=f"Average density {W.SHIP_RHO:,.0f} kg/m3 — "
             f"lighter than water, so it floats.")

    # 10. Four cases, on one scale, a bar at a time. Each row grows from
    #     nothing so the eye follows it to whichever side of 1,000 it lands
    #     on — which is the whole verdict.
    beat("chart", title="Denser than water, or not", say="",
         stage=False, brace=False, arrows=(), rise=False)
    for i in range(len(VERDICTS)):
        state.update(rows=i + 1, grow=0.0)
        walk("draw", steps=14, grow=1.0)
        cm.emit("read")
    beat("law", title="", rows=0, principle=True)
    beat("said", say="A thing floats when it can displace its own weight "
                     "before it is all the way under.")


cm.explain(
    trace=story(dict(OPENING)),
    view=view,
    # Everything that moves was already sampled by the trace, so it must not be
    # eased a second time at every step.
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(
        default=0.1,
        events={"swap": 0.26,
                "card": 0.7, "type": 0.055, "underline": 0.03, "held": 1.6,
                "hook": 2.0, "sink": 0.05, "hook2": 2.0,
                "simpler": 2.4, "waterbox": 0.8, "dip": 0.05,
                "displace": 2.4, "neutral": 3.0,
                "ice": 2.8, "lighter": 2.4,
                "push": 3.0, "more": 2.8,
                "release": 0.8, "rise": 0.045, "shrink": 2.8,
                "stop": 2.2, "why": 2.6, "derive": 1.7, "percent": 3.4,
                "steel": 2.6, "steelforce": 3.0, "sinks": 1.8,
                "question": 2.4, "bigger": 1.6, "grow": 0.05,
                "lift": 0.045,
                "spreadsay": 0.9, "spread": 0.055,
                "hollowsay": 1.0, "hollow": 0.055,
                "lowersay": 1.2, "settle": 0.05,
                "ships": 3.0, "average": 3.2,
                "chart": 1.4, "draw": 0.04, "read": 1.0,
                "law": 3.0, "said": 4.0},
        opening=0.9, final_hold=1.8),
).render("results/archimedes.mp4", fps=60, scale=1.5)

print("wrote results/archimedes.mp4")
