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

import sys

import codimate as cm

import vocabulary
import world as W

# `python main.py km` renders the Khmer one. The physics, the geometry and
# every layout decision are shared — only the vocabulary forks, so a fix to
# the picture cannot land in one language and not the other.
SAY, SCENES = vocabulary.pick(sys.argv[1] if len(sys.argv) > 1 else "en")

cm.canvas(1280, 720)

# ------------------------------------------------------------ the palette
INK, DIM, AIR = "#e8eef7", "#93a0b2", "#161c25"
WATER, GLASS = "#2f7fb8", "#58697e"
UP, DOWN, HAND = "#38d6e0", "#ff7a59", "#f2c14e"     # buoyancy, weight, push
PLATE = "#0b1018"
SKIN = {"water": "#4aa8dd", "ice": "#cfefff", "iron": "#96a2b0"}
RHO = {"water": W.RHO_WATER, "ice": W.RHO_ICE, "iron": W.RHO_IRON}
SAYS = {"water": SAY["water"], "ice": SAY["ice_word"],
        "iron": SAY["iron_word"]}

# ------------------------------------------------------------- the layout
OBJ_X = 0.5 * (W.TANK[0] + W.TANK[2])
NOTES_X = 1116.0                 # the column the tank never reaches into
TITLE_Y, SAY_Y = 26.0, 696.0
TITLE_SIZE, SAY_SIZE = 30, 21
ROOM = 1150.0                    # the widest any line of prose may be

# Both marks live outside the glass: the displaced volume down the left wall,
# how much of the object is under down the right. Nothing is drawn over the
# water, the glass or the object itself. They move with the glass, because the
# glass moves — the beaker grows into a basin when the ship is built.
MARK_GAP = 34.0

# Water is drawn *over* the objects, translucent, so anything below the
# surface is tinted without any clipping — the Engine has none. The hull's
# cavity is then drawn over the water for the same reason, in the frame's own
# own dark, so the inside of the ship reads as the inside of something
# rather than as a hole cut in the picture.
BODY_LAYER, WATER_LAYER, CAVITY_LAYER = 20, 40, 50
MARK_LAYER, TEXT_LAYER, LABEL_LAYER = 60, 80, 95


def _fits(line, size, room=ROOM):
    """The largest size at or below `size` that keeps `line` inside `room`.

    Khmer says the same thing in noticeably more glyphs, and `text` has no
    newlines to wrap at — nor, without a dictionary, anywhere safe to break a
    script that does not put spaces between its words. So the line is shrunk
    instead, measured with the real fonts rather than estimated.
    """
    while size > 13 and cm.measure(line, size=size)[0] > room:
        size -= 1
    return size


def _middle(state):
    return (OBJ_X, state["bottom"] - W.BOX_H / 2)


def create_water_tank(scene, level, mark):
    """Glass, water, and — when `mark` is given — how far the surface climbed.

    `mark` is the multiple of V that has been displaced, because "= V" was
    true only of the fully sunk water box: the ice displaces 0.917 V and the
    ship nearly eight.
    """
    left, top, right, bottom = W.TANK
    rest = W.REST_LEVEL
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
        set_bracket(scene, "rise", (W.TANK[0] - MARK_GAP, level, rest),
                    (UP, -1), (SAY["displaced"],
                               SAY["of_v"].format(mark=mark)))


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
    # Typeset, because these are symbols rather than words: the same F_B that
    # the law states at the end, and needing no translation in either film.
    # Always above the tip — below a downward arrow is where the caption
    # lives, and a long weight arrow put its label straight through it.
    scene.formula((name, "word"), words, size=24,
                  at=cm.at(x=at[0] + 78, y=tip[1] - 18)) \
         .fill(colour).on(layer=TEXT_LAYER)


def animate_force_balance(scene, state):
    """Up, down, and the hand — each drawn only when the scene wants it.

    The steel never changes amount, so the weight is always `rho x V` of the
    original block: that is the whole argument of the last section, and it
    would be lost if the hull's outer area were weighed instead.
    """
    w, h = W.BOX_W, W.BOX_H
    _, sub = W.surface(state["bottom"], w, h)
    middle = _middle(state)
    # The average over the outside, which for a solid box is just the stuff it
    # is made of and for the hollow one is a few hundred. One formula, so the
    # arrows cannot say one thing in the last section and another before it.
    rho = W.density(RHO[state["material"]], state["wall"])
    push = W.buoyancy(w * h * sub)
    pull = W.weight(rho, W.BOX_AREA)

    if "up" in state["arrows"]:
        length, capped = W.arrow_length(push)
        create_force_arrow(scene, "fb", middle, length,
                           (UP, r"F_B", True, capped))
    if "down" in state["arrows"]:
        length, capped = W.arrow_length(pull)
        create_force_arrow(scene, "wt", middle, length,
                           (DOWN, r"W", False, capped))
    if "hand" in state["arrows"]:
        top = (OBJ_X, middle[1] - h / 2 - 8)
        scene.arrow("hand", start=(top[0], top[1] - 96), end=top,
                    w=7.0, head=19.0).fill(HAND).on(layer=MARK_LAYER + 2)
        # Left of the shaft: the buoyancy label already has the right side.
        scene.text(("hand", "word"), SAY["hand"], size=19,
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
            scene.text(("net", "word"), SAY["net"], size=17,
                       at=cm.at(x=x - 66, y=0.5 * (a + b))).fill(UP) \
                 .on(layer=TEXT_LAYER)


def set_submerged_bracket(scene, state, level):
    """How much of the object is under the water — outside the right wall.

    Beside the object it sat on the surface line, on the glass, and for the
    hull almost in the notes column. Out here it mirrors the displaced-volume
    bracket on the left and can never cover anything.
    """
    _, sub = W.surface(state["bottom"], W.BOX_W, W.BOX_H)
    x = W.TANK[2] + MARK_GAP
    for i, (a, b, colour, words) in enumerate((
            (level, state["bottom"], UP, SAY["under"].format(share=sub)),
            (state["bottom"] - W.BOX_H, level, DIM,
             SAY["above"].format(share=1 - sub)))):
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
        scene.formula(("cmp", i, "rho"), vocabulary.unit(rho), size=20,
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
    middle = _middle(state)
    scene.polygon("body", W.outline(middle, state["wall"])) \
         .fill(SKIN[state["material"]], edge=INK, edge_w=2.0) \
         .on(layer=BODY_LAYER)

    left, top, right, bottom = W.cavity(middle, state["wall"])
    if right - left > 2.0:
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


CARD = SAY["card"]
CARD_SIZE = 58
CARD_Y = 330.0


def clusters(line):
    """`line` split where a reader would say one character begins.

    Not by code point. Khmer writes a syllable as a base consonant plus marks
    that hang off it — vowel signs, and COENG (U+17D2), which turns the letter
    *after* it into a subscript. Split those apart and the shaper is handed
    fragments that mean nothing: គោលការណ៍ types out as គ លេក រណអ៍.

    So a cluster is a base plus everything that belongs to it, and the title
    is revealed a cluster at a time in any language.
    """
    marks = range(0x17B4, 0x17D4)                  # Khmer vowels and signs
    out, i = [], 0
    while i < len(line):
        piece, i = line[i], i + 1
        while i < len(line):
            here = ord(line[i])
            if here == 0x17D2 and i + 1 < len(line):    # COENG + its consonant
                piece, i = piece + line[i:i + 2], i + 2
            elif here in marks or here == 0x17DD or 0x0300 <= here <= 0x036F:
                piece, i = piece + line[i], i + 1
            else:
                break
        out.append(piece)
    return out


CARD_PIECES = clusters(CARD)
CARD_W, CARD_H = cm.measure(CARD, size=CARD_SIZE)
CARD_LEFT = 640.0 - CARD_W / 2

# Where the reveal stops after each cluster. Measured once, from the *whole*
# title each time, so the mask edge always lands on a boundary the shaper
# agrees with rather than part way through a glyph.
CARD_STOPS = [0.0] + [cm.measure("".join(CARD_PIECES[:i + 1]), size=CARD_SIZE)[0]
                      for i in range(len(CARD_PIECES))]


def create_title_card(scene, state):
    """The name, uncovered a cluster at a time, then a rule drawn under it.

    The text is drawn whole and never moves. An earlier version grew the
    string itself, which meant moving the shape left every step to keep its
    left edge still — so the title crept sideways as it typed. Here a plate in
    the frame's own black covers what has not been reached yet and slides off
    to the right, which is the same reading and no motion in the words.

    It also means the reveal needs no per-character layout at all: the title is
    laid out once, by the engine, exactly as it will finally appear.

    The mask arrives one scene *before* the words. Both fade in when they
    enter, and a half-faded plate does not hide a half-faded title — so the
    whole name ghosted through on the opening frame. Letting the plate fade up
    alone against black costs nothing to look at and is opaque by the time
    there is anything behind it.
    """
    edge = CARD_LEFT + CARD_STOPS[min(state["letters"], len(CARD_PIECES))]
    if edge < CARD_LEFT + CARD_W:
        far = CARD_LEFT + CARD_W + 8
        scene.rect("card_mask", w=far - edge, h=CARD_H + 24,
                   at=(0.5 * (edge + far), CARD_Y)) \
             .fill("#000000").on(layer=TEXT_LAYER + 6)

    if state["letters"] < 1:
        return
    scene.text("card", CARD, size=CARD_SIZE, at=cm.at(x=640, y=CARD_Y)) \
         .fill(INK).on(layer=TEXT_LAYER + 5)

    # Left to right, under the words, the way the pen went.
    run = CARD_W * state["rule"]
    if run > 1.0:
        scene.line("card_rule", start=(CARD_LEFT, CARD_Y + 48),
                   end=(CARD_LEFT + run, CARD_Y + 48), w=2.5) \
             .fill(UP).on(layer=TEXT_LAYER + 5, opacity=0.85)


# The four cases, on one scale. Steel is eight times off the end of it, which
# is the point: it is the only row that needs the axis broken.
VERDICTS = ((SAY["ice_name"], W.RHO_ICE, SKIN["ice"], SAY["floats"]),
            (SAY["water_name"], W.RHO_WATER, SKIN["water"], SAY["even"]),
            (SAY["hollow_name"], W.HOLLOW_RHO, SKIN["iron"], SAY["floats"]),
            (SAY["block_name"], W.RHO_IRON, SKIN["iron"], SAY["sinks"]))
# Placed so the whole block — names, bars, values, verdicts — is centred on
# the frame rather than the bars alone being centred.
BAR_X, BAR_PER, BAR_MAX = 396.0, 0.30, 420.0     # px per kg/m3, and the cut
VERDICT_INK = {SAY["floats"]: UP, SAY["even"]: DIM, SAY["sinks"]: DOWN}


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
    # The word to the left of the line, the quantity to the right. Laying
    # them out end to end would need the formula's width, and a formula is
    # typeset by Typst at build time — there is nothing here to measure.
    scene.text("mark_word", SAY["water_name"], size=18,
               at=cm.at(x=line - 62, y=top - 62)).fill(WATER) \
         .on(layer=TEXT_LAYER)
    scene.formula("mark_rho", vocabulary.unit(W.RHO_WATER), size=19,
                  at=cm.at(x=line + 92, y=top - 62)).fill(WATER) \
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
                     .fill("#000000").on(layer=MARK_LAYER + 2)


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
    level, _ = W.surface(state["bottom"], W.BOX_W, W.BOX_H)

    scene.text("title", state["title"], size=_fits(state["title"], TITLE_SIZE),
               at=cm.at(x=640, top=TITLE_Y)).fill(INK).on(layer=TEXT_LAYER)
    if state["say"]:
        scene.text("say", state["say"], size=_fits(state["say"], SAY_SIZE),
                   at=cm.at(x=640, bottom=SAY_Y)).fill(DIM) \
             .on(layer=TEXT_LAYER)
    if state["note"]:
        scene.formula("note", state["note"], size=27,
                      at=cm.at(x=640, y=622)).fill(INK) \
             .on(layer=TEXT_LAYER)

    if state["stage"]:
        mark = None
        if state["rise"]:
            mark = W.displaced(state["bottom"], W.BOX_W,
                               W.BOX_H) / W.BOX_AREA
        create_water_tank(scene, level, mark)
        create_body(scene, state)
        animate_force_balance(scene, state)
        if state["brace"]:
            set_submerged_bracket(scene, state, level)

    if state["compare"]:
        create_density_comparison(scene, state)
    if state["steps"]:
        create_equation_step(scene, state)

    if state["carding"]:
        create_title_card(scene, state)

    if state["big"]:
        # In the band between the title and the tank's rim. Inside the tank
        # it lands on the floating box, and below the glass is the subtitle.
        scene.text("big", state["big"], size=42, at=cm.at(x=640, y=110)) \
             .fill(UP).on(layer=TEXT_LAYER + 5)
    if state["principle"]:
        scene.formula("law", r"F_B \;=\; \rho_{fluid}\, V_{displaced}\, g",
                      size=52, at=cm.at(x=640, y=300)).fill(INK) \
             .on(layer=TEXT_LAYER + 5)

    if state["rows"]:
        create_verdict_chart(scene, state)
    return scene


OPENING = {
    "title": "", "carding": True, "letters": 0, "rule": 0.0, "note": "",
    "say": "", "stage": False, "material": "iron",
    "wall": W.SOLID,
    "bottom": 250.0, "label": False, "arrows": (),
    "net": False, "brace": False, "rise": False, "compare": (),
    "steps": 0, "big": "", "principle": False, "rows": 0, "grow": 0.0,
}

AFLOAT = W.settles(W.HOLLOW_RHO)


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
        """One section: the picture changes, then it holds.

        The name is the scene's name, so the title and the subtitle come from
        the vocabulary rather than from the call. That is what keeps the two
        jobs apart — a beat cannot quietly acquire a sentence for a title, or
        a title with nothing said underneath it.
        """
        title, say = SCENES[change.pop("scene", name)]
        state.update(title=title.format(**change.pop("says", {})),
                     say=say.format(**change.pop("said", {})))
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
                state[k] = was[k] + (target - was[k]) * i / steps
            cm.emit(name)

    # 0. The name first, typed on, then underlined. Nothing else is on
    #    screen, so the film opens on it rather than fading up to it.
    cm.emit("card")
    for i in range(1, len(CARD_PIECES) + 1):
        state["letters"] = i
        cm.emit("type")
    walk("underline", steps=16, rule=1.0)
    cm.emit("held")

    # 1. The hook. A steel block falls in and keeps going.
    beat("hook", carding=False, letters=0, rule=0.0, stage=True)
    walk("sink", steps=32, bottom=W.FLOOR)
    beat("hook2")

    # 2. Simplify: one box, volume V, held above the water.
    beat("simpler", material="water", label=True, bottom=250.0)

    # 3. Make it a box *of water*, and lower it in.
    beat("waterbox")
    walk("dip", steps=32, bottom=W.water_level(W.BOX_AREA) + W.BOX_H)
    beat("displace", rise=True)
    beat("neutral", arrows=("up", "down"))
    beat("why_neutral", scene="neutral")

    # 4. Same box, different stuff.
    beat("ice", material="ice", arrows=(),
         compare=((SAY["water_name"], W.RHO_WATER, SKIN["water"]),
                  (SAY["ice_name"], W.RHO_ICE, SKIN["ice"])))
    beat("lighter")

    # 5. Hold it under, and look at the two forces.
    beat("push", compare=(), arrows=("up", "down", "hand"), net=True)
    beat("more")

    # 6. Let go. It rises; the push stays put until it breaks the surface.
    beat("release", arrows=("up", "down"))
    walk("rise", steps=34,
         bottom=W.settles(W.RHO_ICE))
    beat("shrink")

    # 7. The payoff, and only then the algebra.
    beat("stop", net=False, brace=True)
    beat("why")
    # The brace comes off while the algebra is on screen: both want the right
    # hand side of the frame, and neither needs the other to be readable.
    for n in (1, 2, 3, 4):
        beat("derive", steps=n, brace=False)
    beat("percent", steps=0, brace=True,
         big=SAY["submerged"].format(share=W.ICE_SUBMERGED))

    # 8. Steel, in exactly the same box.
    beat("steel", material="iron", big="", brace=False, arrows=(),
         compare=((SAY["water_name"], W.RHO_WATER, SKIN["water"]),
                  (SAY["iron_word"], W.RHO_IRON, SKIN["iron"])))
    walk("dip", steps=16, bottom=W.REST_LEVEL + W.BOX_H + 50.0)
    beat("steelforce", compare=(), arrows=("up", "down"))
    # The arrows go *before* it drops. Left on, a weight arrow this long
    # reaches out of the frame once the block is resting on the floor.
    beat("sinks", arrows=())
    walk("sink", steps=24, bottom=W.FLOOR)

    # 9. The answer. The box never changes size and never leaves the floor to
    #    be worked on, and it is never told to rise: `settles` is asked where
    #    a box of that average density belongs, and the answer changes once
    #    enough iron is gone.
    beat("question", arrows=(), label=False)

    # The walls thin inward. Nothing is added and nothing grows — iron is
    # taken away, and the average over the same outside falls with it.
    beat("hollowing")
    walk("thin", steps=34, wall=W.thinning(W.LIFTS_AT))

    # The last of the thinning and the lift-off are one move. Left as two, the
    # film shows a box that can float sitting on the bottom of the tank.
    beat("rising")
    walk("lifts", steps=30, wall=W.WALL, bottom=AFLOAT)
    beat("floats", said={"left": 100 * W.LEFT_OF_IT},
         arrows=("up", "down"), brace=True, rise=True)
    beat("average", note=vocabulary.unit(W.HOLLOW_RHO)
         + r"\;<\;" + vocabulary.unit(W.RHO_WATER))

    # 10. Four cases, on one scale, a bar at a time. Each row grows from
    #     nothing so the eye follows it to whichever side of 1,000 it lands
    #     on — which is the whole verdict.
    beat("chart", note="", stage=False, brace=False, arrows=(),
         rise=False)
    for i in range(len(VERDICTS)):
        state.update(rows=i + 1, grow=0.0)
        walk("draw", steps=14, grow=1.0)
        cm.emit("read")
    beat("law", rows=0, principle=True)
    beat("said")


cm.explain(
    trace=story(dict(OPENING)),
    view=view,
    # Everything that moves was already sampled by the trace, so it must not be
    # eased a second time at every step.
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(
        default=0.1,
        events={"swap": 0.26,
                "card": 0.25, "type": 0.06, "underline": 0.028, "held": 1.7,
                "hook": 2.0, "sink": 0.05, "hook2": 2.0,
                "simpler": 2.4, "waterbox": 0.8, "dip": 0.05,
                "displace": 2.4, "neutral": 2.6, "why_neutral": 3.0,
                "ice": 2.8, "lighter": 2.4,
                "push": 3.0, "more": 2.8,
                "release": 0.8, "rise": 0.045, "shrink": 2.8,
                "stop": 2.2, "why": 2.6, "derive": 1.7, "percent": 3.4,
                "steel": 2.6, "steelforce": 3.0, "sinks": 1.8,
                "question": 2.6,
                "hollowing": 1.6, "thin": 0.05,
                "rising": 1.4, "lifts": 0.055,
                "average": 3.2,
                "floats": 3.0, "chart": 1.4, "draw": 0.04, "read": 1.0,
                "law": 3.0, "said": 4.0},
        # The film opens on a masked title, which is a black frame — so the
        # opening hold is short. A second of it would read as a stall.
        opening=0.25, final_hold=1.8),
).render(SAY["out"], fps=60, scale=1.5)

print(f"wrote {SAY['out']}")
