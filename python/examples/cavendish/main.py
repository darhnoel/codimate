"""How do you weigh the Earth? The Cavendish experiment as one story.

1. The impossible question — Earth on a balance, which tips
2. Newton's clue — Earth pulls an apple; so do two balls on a table
3. The catch — that pull between lab balls is absurdly small
4. Turn the invisible into motion — Michell's torsion balance
5. From a tiny twist to the planet — and a zoom out to say how small it was

    .venv/bin/python python/examples/cavendish/main.py

The narration is Khmer captions on a plate, a bright mark stepping word by word,
at reading pace (`lines.py`, made by `segment_lines.py`). Drawn by hand rather
than with `codimate.science`: it came first, and is the long film the kit was
cut out of."""

import math
import random
from dataclasses import dataclass

import codimate as cm
from lines import SAY  # Khmer, word boundaries marked

cm.canvas(1280, 720)

EARTH_COLOR = "#58c4dd"
BRASS = "#c9a84c"
STRING_COLOR = "#93a0b2"
INK, DIM = "#e8eef7", "#93a0b2"
BLUE, ORANGE, GREEN, RED = "#58c4dd", "#ff9f43", "#7bd88f", "#ff6a5c"
STEEL, STEEL_EDGE = "#8fa3bf", "#c5d3e8"
APPLE, YELLOW = "#ff5d5d", "#ffd23f"

LAY_SCALE, LAY_EARTH, LAY_APPLE, LAY_PAIR = 1, 20, 550, 600
LAY_CITY, LAY_LAB, LAY_APP, LAY_S5 = 520, 790, 900, 1000
LAY_PLATE = 9000

ZC = (640, 320)  # the one point every zoom level nests around
TABLE_Y = 380


@dataclass(frozen=True)
class Look:
    """How something is drawn: colour, opacity and layer — and a type size, for
    what is made of text. Grouped because they always travel together."""

    color: str = "white"
    op: float = 1.0
    layer: int = 0
    size: float = 20.0


@dataclass(frozen=True)
class Arc:
    """A circle round `centre`, from `start` to `end` degrees (counter-clockwise
    positive)."""

    centre: tuple
    r: float
    start: float
    end: float


@dataclass(frozen=True)
class Pose:
    """Where the torsion balance is: its centre, the zoom `f` it is drawn at, the
    angle `phi` the rod has turned, how near the big balls are (`big_t`, 0 to 1)
    and how visible the force arrows are."""

    centre: tuple
    f: float
    phi: float = 0.0
    big_t: float = 1.0
    force_op: float = 0.0


# ---- captions, the way archimedes/main.py does them: a plate sized to the
# line, a bright mark stepping word by word, at most eight words on screen,
# and the scene lasting as long as the line takes to read.

ZWSP = "​"
CAPTION_BG = "#141a26"
READ, UNREAD = "#58C4DD", "#5d6a7e"
SAY_Y, SAY_SIZE, SAY_ROOM = 636.0, 34, 1150.0
PAGE = 8  # the most words shown at once
WORD, PER_LETTER, CLAUSE = 0.24, 0.035, 0.55  # reading pace, as archimedes
STOPS = "។៕៖,.;:?!"
_CLUSTER = __import__("re").compile(r"[ក-ឳ](?:្[ក-ឳ])*[ា-៓]*")


def chunks(line):
    """`line` as the pieces the mark steps over — Khmer words where the line
    has been segmented (ZWSP), spaced words otherwise — each flagged with
    whether a real space comes before it."""
    pieces = []
    for w, token in enumerate(line.split()):
        first = True
        for part in token.split(ZWSP):
            if part:
                pieces.append((part, (w > 0) and first))
                first = False
    return pieces


def clusters(line):
    """`line` split into orthographic clusters, so a word is timed by what a
    reader sees and not by how many code points hold its vowel signs."""
    marks = range(0x17B4, 0x17D4)
    line = line.replace(ZWSP, "")
    out, i = [], 0
    while i < len(line):
        piece, i = line[i], i + 1
        while i < len(line):
            here = ord(line[i])
            if here == 0x17D2 and i + 1 < len(line):
                piece, i = piece + line[i : i + 2], i + 2
            elif here in marks or here == 0x17DD or 0x0300 <= here <= 0x036F:
                piece, i = piece + line[i], i + 1
            else:
                break
        out.append(piece)
    return out


def pace(piece):
    """Seconds one word holds the mark: a moment, a little per letter, and a
    rest where a clause ends."""
    rest = CLAUSE if piece and piece[-1] in STOPS else 0.0
    return WORD + PER_LETTER * len(clusters(piece)) + rest


def fit(line, size, room=SAY_ROOM):
    while size > 13 and cm.measure(line, size=size)[0] > room:
        size -= 1
    return size


# ------------------------------------------------------------- small maths


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def cross(a, b):
    return (
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    )


def norm(v):
    m = math.sqrt(dot(v, v))
    return (v[0] / m, v[1] / m, v[2] / m)


def lerp(a, b, t):
    return a + (b - a) * t


def clamp(x, lo=0.0, hi=1.0):
    return max(lo, min(hi, x))


def smooth(t):
    return t * t * (3 - 2 * t)


def mix(c1, c2, t):
    a = [int(c1[i : i + 2], 16) for i in (1, 3, 5)]
    b = [int(c2[i : i + 2], 16) for i in (1, 3, 5)]
    return "#%02x%02x%02x" % tuple(int(lerp(x, y, clamp(t))) for x, y in zip(a, b))


# ----------------------------------------------------- the wireframe Earth

AZIMUTH, ELEVATION = 20.0, 12.0
DIST, FOCAL = 2600, 1700
CX, CY = 640, 300
SCREEN_PER_WORLD = FOCAL / DIST

az, el = math.radians(AZIMUTH), math.radians(ELEVATION)
CAM = (
    DIST * math.cos(el) * math.cos(az),
    DIST * math.cos(el) * math.sin(az),
    DIST * math.sin(el),
)
FORWARD = norm((-CAM[0], -CAM[1], -CAM[2]))
RIGHT = norm(cross(FORWARD, (0.0, 0.0, 1.0)))
UP = cross(RIGHT, FORWARD)


def project(p):
    rel = sub(p, CAM)
    cx, cy, cz = dot(rel, RIGHT), dot(rel, UP), dot(rel, FORWARD)
    s = FOCAL / cz
    return (CX + cx * s, CY - cy * s), cz


def _icosphere(subdiv):
    phi = (1 + 5**0.5) / 2
    verts = [
        norm(v)
        for v in [
            (-1, phi, 0),
            (1, phi, 0),
            (-1, -phi, 0),
            (1, -phi, 0),
            (0, -1, phi),
            (0, 1, phi),
            (0, -1, -phi),
            (0, 1, -phi),
            (phi, 0, -1),
            (phi, 0, 1),
            (-phi, 0, -1),
            (-phi, 0, 1),
        ]
    ]
    faces = [
        (0, 11, 5),
        (0, 5, 1),
        (0, 1, 7),
        (0, 7, 10),
        (0, 10, 11),
        (1, 5, 9),
        (5, 11, 4),
        (11, 10, 2),
        (10, 7, 6),
        (7, 1, 8),
        (3, 9, 4),
        (3, 4, 2),
        (3, 2, 6),
        (3, 6, 8),
        (3, 8, 9),
        (4, 9, 5),
        (2, 4, 11),
        (6, 2, 10),
        (8, 6, 7),
        (9, 8, 1),
    ]
    cache = {}

    def midpoint(i, j):
        key = (min(i, j), max(i, j))
        if key not in cache:
            vi, vj = verts[i], verts[j]
            verts.append(
                norm(((vi[0] + vj[0]) / 2, (vi[1] + vj[1]) / 2, (vi[2] + vj[2]) / 2))
            )
            cache[key] = len(verts) - 1
        return cache[key]

    for _ in range(subdiv):
        next_faces = []
        for a, b, c in faces:
            ab, bc, ca = midpoint(a, b), midpoint(b, c), midpoint(c, a)
            next_faces += [(a, ab, ca), (b, bc, ab), (c, ca, bc), (ab, bc, ca)]
        faces = next_faces
    return verts, faces


ICO_VERTS, ICO_FACES = _icosphere(2)
ICO_EDGES = sorted(
    {
        (min(i, j), max(i, j))
        for a, b, c in ICO_FACES
        for i, j in ((a, b), (b, c), (c, a))
    }
)


def rotate_z(v, degrees):
    a = math.radians(degrees)
    x, y, z = v
    return (x * math.cos(a) - y * math.sin(a), x * math.sin(a) + y * math.cos(a), z)


def draw_earth(scene, spin, center, r_screen, op):
    """Wireframe Earth at a screen position and size. Each edge is keyed by
    its own vertex pair, so it tweens as one line while the globe spins."""
    if op <= 0 or r_screen < 1:
        return
    r_world = r_screen / SCREEN_PER_WORLD
    spun = [rotate_z(v, spin) for v in ICO_VERTS]
    proj = [project((v[0] * r_world, v[1] * r_world, v[2] * r_world)) for v in spun]
    dx, dy = center[0] - CX, center[1] - CY
    segs = []
    for i, j in ICO_EDGES:
        (a, da), (b, db) = proj[i], proj[j]
        segs.append(
            ((da + db) / 2, (a[0] + dx, a[1] + dy), (b[0] + dx, b[1] + dy), (i, j))
        )
    segs.sort(key=lambda s: -s[0])
    zlo, zhi = segs[-1][0], segs[0][0]
    span = max(zhi - zlo, 1e-6)
    lw = clamp(r_screen / 70, 1.0, 1.8)
    for order, (depth, a, b, key) in enumerate(segs):
        o = (0.95 - 0.6 * (depth - zlo) / span) * op
        scene.line(("earth", *key), start=a, end=b, w=lw).fill(EARTH_COLOR).on(
            layer=LAY_EARTH + order, opacity=o
        )


# ------------------------------------------------------------ 1. the scale

PIVOT = (640, 265)
BEAM_HALF = 270
STRING_LEN = 170
PAN_W = 220
PAN_DEPTH = 22
POST_BOTTOM = 520
TIP_MAX = 11.0
HERO_CENTER, HERO_R = (640, 125), 95
REST_R = 70


def beam_ends(tip):
    th = math.radians(TIP_MAX * tip)
    return (
        (PIVOT[0] - BEAM_HALF * math.cos(th), PIVOT[1] + BEAM_HALF * math.sin(th)),
        (PIVOT[0] + BEAM_HALF * math.cos(th), PIVOT[1] - BEAM_HALF * math.sin(th)),
    )


def earth_pose(tip):
    left, _ = beam_ends(tip)
    rest = (left[0], left[1] + STRING_LEN - REST_R + 8)
    return (
        (
            HERO_CENTER[0] + (rest[0] - HERO_CENTER[0]) * tip,
            HERO_CENTER[1] + (rest[1] - HERO_CENTER[1]) * tip,
        ),
        HERO_R + (REST_R - HERO_R) * tip,
    )


def draw_scale(scene, reveal, tip):
    if reveal <= 0:
        return
    L = LAY_SCALE
    cx, py = PIVOT
    scene.polygon(
        ("scale", "post"),
        [(cx - 7, py), (cx + 7, py), (cx + 13, POST_BOTTOM), (cx - 13, POST_BOTTOM)],
    ).fill(BRASS).on(layer=L, opacity=reveal)
    scene.polygon(
        ("scale", "base"),
        [
            (cx - 120, POST_BOTTOM),
            (cx + 120, POST_BOTTOM),
            (cx + 95, POST_BOTTOM + 26),
            (cx - 95, POST_BOTTOM + 26),
        ],
    ).fill(BRASS).on(layer=L, opacity=reveal)
    left, right = beam_ends(tip)
    scene.line(("scale", "beam"), start=left, end=right, w=9).fill(BRASS).on(
        layer=L + 1, opacity=reveal
    )
    for name, end in (("capL", left), ("capR", right)):
        scene.circle(("scale", name), r=7, at=end).fill(BRASS).on(
            layer=L + 2, opacity=reveal
        )
    scene.circle(("scale", "pivot"), r=10, at=PIVOT).fill(
        BRASS, edge="#0b0f16", edge_w=2
    ).on(layer=L + 3, opacity=reveal)
    for side, end in (("L", left), ("R", right)):
        pan_y = end[1] + STRING_LEN
        for sgn, tag in ((-1, "a"), (1, "b")):
            scene.line(
                ("scale", f"string{side}{tag}"),
                start=end,
                end=(end[0] + sgn * PAN_W / 2, pan_y),
                w=2,
            ).fill(STRING_COLOR).on(layer=L + 1, opacity=reveal)
        dish = [
            (
                end[0] - (PAN_W / 2) * math.cos(math.pi * k / 14),
                pan_y + PAN_DEPTH * math.sin(math.pi * k / 14),
            )
            for k in range(15)
        ]
        scene.polygon(("scale", f"pan{side}"), dish).fill(
            BRASS, edge=BRASS, edge_w=2
        ).on(layer=L + 1, opacity=0.55 * reveal)


# --------------------------------------------- 2/3. apple, balls, the table


def draw_apple(scene, st):
    a = st["apple_op"]
    if a <= 0:
        return
    x, y = 640, st["apple_y"]
    L = LAY_APPLE
    scene.line(("apple", "stem"), start=(x, y - 15), end=(x + 3, y - 29), w=3).fill(
        "#8a5a2b"
    ).on(layer=L, opacity=a)
    scene.polygon(
        ("apple", "leaf"), [(x + 3, y - 25), (x + 20, y - 33), (x + 12, y - 19)]
    ).fill(GREEN).on(layer=L, opacity=a)
    scene.circle(("apple", "body"), r=17, at=(x, y)).fill(
        APPLE, edge="#0b0f16", edge_w=1.5
    ).on(layer=L + 1, opacity=a)
    if st["grav_arrow_op"] > 0:
        scene.arrow(
            ("apple", "pull"), start=(x, y + 24), end=(x, y + 86), w=5, head=15
        ).fill(BLUE).on(layer=L + 2, opacity=a * st["grav_arrow_op"])


def ball(scene, key, spot, look):
    (x, r), (color, op, layer) = spot, (look.color, look.op, look.layer)
    y = TABLE_Y - r
    scene.circle((*key, "body"), r=r, at=(x, y)).fill(
        color, edge=STEEL_EDGE, edge_w=2
    ).on(layer=layer, opacity=op)
    scene.circle((*key, "hi"), r=r * 0.28, at=(x - r * 0.32, y - r * 0.34)).fill(
        "#ffffff"
    ).on(layer=layer + 1, opacity=0.35 * op)


def draw_pair(scene, st):
    a = st["pair_op"]
    if a <= 0:
        return
    L = LAY_PAIR
    tb = a * st["table_op"]
    if tb > 0:
        scene.rect(("pair", "table"), w=1000, h=26, at=(640, TABLE_Y + 13)).fill(
            "#3a4a63"
        ).on(layer=L, opacity=tb)
        for sx in (-1, 1):
            scene.rect(
                ("pair", f"leg{sx}"), w=22, h=150, at=(640 + sx * 440, TABLE_Y + 101)
            ).fill("#2b3445").on(layer=L, opacity=tb)

    xA, rA, xB, rB = st["ballA_x"], st["ballA_r"], st["ballB_x"], st["ballB_r"]
    yA, yB = TABLE_Y - rA, TABLE_Y - rB
    colB = mix(STEEL, ORANGE, (70 - rB) / 55)
    if st["ballA_op"] > 0:
        ball(scene, ("pair", "A"), (xA, rA), Look(STEEL, a * st["ballA_op"], L + 2))
    if st["ballB_op"] > 0:
        ball(scene, ("pair", "B"), (xB, rB), Look(colB, a * st["ballB_op"], L + 2))

    ao, ln = a * st["arrow_op"], st["arrow_len"]
    if ao > 0 and ln >= 4:
        d = norm((xB - xA, yB - yA, 0.0))
        s1 = (xA + d[0] * (rA + 4), yA + d[1] * (rA + 4))
        s2 = (xB - d[0] * (rB + 4), yB - d[1] * (rB + 4))
        w, head = max(2.0, min(6.0, ln * 0.12)), min(15.0, ln * 0.6)
        scene.arrow(
            ("pair", "fA"),
            start=s1,
            end=(s1[0] + d[0] * ln, s1[1] + d[1] * ln),
            w=w,
            head=head,
        ).fill(BLUE).on(layer=L + 5, opacity=ao)
        scene.arrow(
            ("pair", "fB"),
            start=s2,
            end=(s2[0] - d[0] * ln, s2[1] - d[1] * ln),
            w=w,
            head=head,
        ).fill(BLUE).on(layer=L + 5, opacity=ao)

    lo = a * st["label_op"]
    if lo > 0:
        scene.formula(("pair", "m1"), "m_1", size=34, at=(xA, yA - rA - 34)).fill(
            INK
        ).on(layer=L + 6, opacity=lo)
        scene.formula(("pair", "m2"), "m_2", size=34, at=(xB, yB - rB - 34)).fill(
            INK
        ).on(layer=L + 6, opacity=lo)
        by = 150
        scene.line(("pair", "rline"), start=(xA, by), end=(xB, by), w=2).fill(DIM).on(
            layer=L + 6, opacity=lo
        )
        for name, x in (("rtA", xA), ("rtB", xB)):
            scene.line(("pair", name), start=(x, by - 9), end=(x, by + 9), w=2).fill(
                DIM
            ).on(layer=L + 6, opacity=lo)
        scene.formula(("pair", "rlab"), "r", size=32, at=((xA + xB) / 2, by + 26)).fill(
            INK
        ).on(layer=L + 6, opacity=lo)

    ko = a * st["kg_op"]
    if ko > 0:
        scene.formula(
            ("pair", "kgA"), r"158\ \text{kg}", size=34, at=(xA, yA - rA - 34)
        ).fill(INK).on(layer=L + 6, opacity=ko)
        scene.formula(
            ("pair", "kgB"), r"0.73\ \text{kg}", size=34, at=(xB + 60, yB - rB - 40)
        ).fill(ORANGE).on(layer=L + 6, opacity=ko)
    if st["F_op"] > 0:
        scene.formula(
            ("pair", "F"),
            r"F \approx 1.5\times10^{-7}\ \text{N}",
            size=58,
            at=(640, 485),
        ).fill(BLUE).on(layer=L + 7, opacity=a * st["F_op"])


# ----------------------------------------------------- 4. the torsion balance

APP_C = (640, 320)
APP_L, APP_D, APP_DFAR = 190, 140, 185
R_SMALL, R_BIG = 16, 46
PHI_EXAG = 0.20  # radians — taught large, so it can be seen at all
PHI_REAL = 0.02


def curved_arrow(scene, key, arc, look, w):
    """An arc from math-angle th0 to th1 (degrees, counter-clockwise
    positive) with an arrowhead where it ends."""
    C, r, th0, th1 = arc.centre, arc.r, arc.start, arc.end
    color, a, layer = look.color, look.op, look.layer
    if a <= 0 or abs(th1 - th0) < 2:
        return
    n = max(3, int(abs(th1 - th0) / 4))
    pts = [
        (
            C[0] + r * math.cos(math.radians(th0 + (th1 - th0) * i / n)),
            C[1] - r * math.sin(math.radians(th0 + (th1 - th0) * i / n)),
        )
        for i in range(n + 1)
    ]
    scene.curve((*key, "arc"), pts, w=w).fill(color).on(layer=layer, opacity=a)
    tip, prev = pts[-1], pts[-3]
    ux, uy = tip[0] - prev[0], tip[1] - prev[1]
    d = math.hypot(ux, uy) or 1.0
    ux, uy = ux / d, uy / d
    hl, hw = 16, 9
    scene.polygon(
        (*key, "head"),
        [
            (tip[0] + ux * 6, tip[1] + uy * 6),
            (tip[0] - ux * hl - uy * hw, tip[1] - uy * hl + ux * hw),
            (tip[0] - ux * hl + uy * hw, tip[1] - uy * hl - ux * hw),
        ],
    ).fill(color).on(layer=layer, opacity=a)


def draw_apparatus(scene, tag, pose, a, layer):
    """The torsion balance seen from above: a rod hung from a wire at its
    middle, a small ball on each end, a big ball waiting near each one."""
    C, f, phi = pose.centre, pose.f, pose.phi
    big_t, force_op = pose.big_t, pose.force_op
    if a <= 0:
        return
    L = APP_L * f
    cs, sn = math.cos(phi), math.sin(phi)
    right, left = (C[0] + L * cs, C[1] - L * sn), (C[0] - L * cs, C[1] + L * sn)
    rs, rb = max(2.0, R_SMALL * f), max(3.0, R_BIG * f)
    d = lerp(APP_DFAR, APP_D, big_t) * f
    bigR, bigL = (C[0] + L, C[1] - d), (C[0] - L, C[1] + d)

    scene.line((tag, "rod"), start=left, end=right, w=max(1.5, 6 * f)).fill(BRASS).on(
        layer=layer + 1, opacity=a
    )
    scene.circle((tag, "ring"), r=max(4.0, 16 * f), at=C).fill(
        "none", edge=DIM, edge_w=max(1.0, 1.5 * f)
    ).on(layer=layer, opacity=0.7 * a)
    scene.circle((tag, "wire"), r=max(2.0, 5 * f), at=C).fill(INK).on(
        layer=layer + 3, opacity=a
    )
    for name, p in (("sR", right), ("sL", left)):
        scene.circle((tag, name), r=rs, at=p).fill(
            ORANGE, edge="#0b0f16", edge_w=max(0.8, 1.5 * f)
        ).on(layer=layer + 2, opacity=a)
    for name, p in (("bR", bigR), ("bL", bigL)):
        scene.circle((tag, name), r=rb, at=p).fill(
            STEEL, edge=STEEL_EDGE, edge_w=max(1.0, 2 * f)
        ).on(layer=layer + 2, opacity=a)

    fo = a * force_op
    if fo > 0:
        for name, s, b in (("fR", right, bigR), ("fL", left, bigL)):
            dx, dy = b[0] - s[0], b[1] - s[1]
            m = math.hypot(dx, dy)
            ux, uy = dx / m, dy / m
            st0 = (s[0] + ux * (rs + 3), s[1] + uy * (rs + 3))
            scene.arrow(
                (tag, name),
                start=st0,
                end=(st0[0] + ux * 48 * f, st0[1] + uy * 48 * f),
                w=max(1.5, 5 * f),
                head=max(5.0, 14 * f),
            ).fill(BLUE).on(layer=layer + 4, opacity=fo)


# ------------------------------------------------------------- zoom levels


def Tz(p, f):
    return (ZC[0] + (p[0] - ZC[0]) * f, ZC[1] + (p[1] - ZC[1]) * f)


def lvl_op(f):
    """How visible a nested level is at scale f: full while it is the one
    in focus (or the tiny thing nested inside the next), gone once it is
    far bigger than the screen or far too small to read."""
    u = math.log(max(f, 1e-9)) / math.log(8)
    return clamp((1.3 - u) / 1.1) * clamp((u + 1.8) / 0.6)


LAB = [
    (640, 320, 1000, 560, None),
    (640, 320, 90, 70, "#2b3445"),
    (350, 115, 260, 60, "#1d2635"),
    (930, 115, 300, 60, "#1d2635"),
    (260, 520, 160, 100, "#1d2635"),
    (1010, 500, 200, 120, "#1d2635"),
]

_rnd = random.Random(7)
CITY = []
for gx in range(-4, 5):
    for gy in range(-3, 4):
        if gx == 0 and gy == 0:
            continue
        cx_, cy_ = 640 + gx * 320, 320 + gy * 250
        park = (gx, gy) in {(2, -1), (-3, 1)}
        CITY.append((cx_, cy_, 260, 190, "park" if park else "block"))
        if not park:
            for _ in range(3):
                CITY.append(
                    (
                        cx_ + _rnd.uniform(-70, 70),
                        cy_ + _rnd.uniform(-45, 45),
                        _rnd.uniform(60, 110),
                        _rnd.uniform(40, 80),
                        "bld",
                    )
                )


def draw_lab(scene, f, a):
    if a <= 0:
        return
    for i, (cx_, cy_, w, h, fill) in enumerate(LAB):
        scene.rect(("lab", i), w=w * f, h=h * f, at=Tz((cx_, cy_), f)).fill(
            fill or "none",
            edge=BRASS if i == 0 else DIM,
            edge_w=max(1.2, (6 if i == 0 else 3) * f),
        ).on(layer=LAY_LAB + i, opacity=a)


def draw_city(scene, f, a):
    if a <= 0:
        return
    colors = {"block": "#16202e", "park": "#1b3326", "bld": "#26354b"}
    for i, (cx_, cy_, w, h, kind) in enumerate(CITY):
        scene.rect(("city", i), w=w * f, h=h * f, at=Tz((cx_, cy_), f)).fill(
            colors[kind]
        ).on(layer=LAY_CITY + i, opacity=a)
    scene.rect(("city", "plot"), w=260 * f, h=190 * f, at=Tz((640, 320), f)).fill(
        "none", edge="#4a5d7c", edge_w=max(1.0, 2 * f)
    ).on(layer=LAY_CITY + 900, opacity=a)


# --------------------------------------------------------------- 5. the chain

CHAIN_Y = 85
CHAIN = [
    (215, r"\theta", "មុំរមួល"),
    (490, "F", "កម្លាំងទាញ"),
    (790, "G", "ថេរទំនាញសកល"),
    (1065, "M_{⊕}", "ម៉ាស់ផែនដី"),
]
CHAIN_ARROWS = ["ជញ្ជីងរមួល", "ច្បាប់ទំនាញញូតុន", "g និងកាំផែនដី R"]


def plate(scene, key, latex, at, look):
    (x, y), (color, size, a, layer) = at, (look.color, look.size, look.op, look.layer)
    w, h = cm.measure_math(latex, size)
    bw, bh = w + 40, h + 22
    scene.rect((*key, "plate"), w=bw, h=bh, at=(x, y)).fill(
        "#0f1520", edge=color, edge_w=2
    ).round(10).on(layer=layer, opacity=a)
    scene.formula((*key, "text"), latex, size=size, at=(x, y)).fill(color).on(
        layer=layer + 1, opacity=a
    )
    return bw, bh


def draw_chain(scene, st):
    L = LAY_S5
    widths = []
    for i, (x, latex, label) in enumerate(CHAIN):
        a = st[f"ch{i + 1}"]
        if a <= 0:
            widths.append(0)
            continue
        color = YELLOW if i == 3 else BLUE
        bw, bh = plate(scene, ("ch", i), latex, (x, CHAIN_Y), Look(color, a, L, 54))
        widths.append(bw)
        scene.text(
            ("ch", i, "lab"), label, size=24, at=cm.at(x=x, top=CHAIN_Y + bh / 2 + 8)
        ).fill(DIM).on(layer=L + 2, opacity=a)
    for i in range(3):
        a = st[f"ca{i + 1}"]
        if a <= 0 or not widths[i] or not widths[i + 1]:
            continue
        x0 = CHAIN[i][0] + widths[i] / 2 + 10
        x1 = CHAIN[i + 1][0] - widths[i + 1] / 2 - 10
        scene.arrow(
            ("ca", i), start=(x0, CHAIN_Y), end=(x1, CHAIN_Y), w=4, head=14
        ).fill(GREEN).on(layer=L + 2, opacity=a)
        scene.text(
            ("ca", i, "lab"),
            CHAIN_ARROWS[i],
            size=22,
            at=cm.at(x=(x0 + x1) / 2, top=CHAIN_Y - 54),
        ).fill(GREEN).on(layer=L + 2, opacity=a)


# -------------------------------------------------------------------- view

CAP_SLOT_Y = (590, 634, 678)


def view(frame):
    st = frame.state
    scene = cm.Scene()

    draw_scale(scene, st["scale_reveal"], st["tip"])
    draw_earth(
        scene, st["spin"], (st["earth_x"], st["earth_y"]), st["earth_r"], st["earth_op"]
    )
    draw_apple(scene, st)
    if st["formula_op"] > 0:
        scene.formula(
            ("newton",), r"F = G\frac{m_1 m_2}{r^2}", size=52, at=(640, 72)
        ).fill(INK).on(layer=LAY_PAIR + 50, opacity=st["formula_op"])
    draw_pair(scene, st)

    if st["app_op"] > 0:
        a = st["app_op"]
        pose = Pose(APP_C, 1.0, st["rod_phi"], st["big_t"], st["force_op"])
        draw_apparatus(scene, "app", pose, a, LAY_APP)
        lo = a * st["applabel_op"]
        if lo > 0:
            # Each part is named beside itself with a line to it, and the
            # target follows the rod as it turns — stacked together with no
            # line, "rod" and "wire" read as one broken two-line label.
            cs, sn = math.cos(st["rod_phi"]), math.sin(st["rod_phi"])
            d_big = lerp(APP_DFAR, APP_D, st["big_t"])
            small = (APP_C[0] + APP_L * cs, APP_C[1] - APP_L * sn)
            big = (APP_C[0] + APP_L, APP_C[1] - d_big)
            rod_pt = (APP_C[0] - 105 * cs, APP_C[1] + 105 * sn)

            def edge(centre, r, toward):
                dx, dy = toward[0] - centre[0], toward[1] - centre[1]
                m = math.hypot(dx, dy) or 1.0
                return (centre[0] + dx / m * (r + 3), centre[1] + dy / m * (r + 3))

            def name(key, text, pos, target, off):
                scene.text(
                    ("applabel", key), text, size=24, at=cm.at(x=pos[0], y=pos[1])
                ).fill(DIM).on(layer=LAY_APP + 20, opacity=lo)
                dx, dy = target[0] - pos[0], target[1] - pos[1]
                m = math.hypot(dx, dy) or 1.0
                scene.line(
                    ("applabel", key, "lead"),
                    start=(pos[0] + dx / m * off, pos[1] + dy / m * off),
                    end=target,
                    w=1.5,
                ).fill(DIM).on(layer=LAY_APP + 19, opacity=0.7 * lo)

            lab_small = (small[0] + 95, small[1] + 34)
            lab_big = (big[0] + 105, big[1])
            name("lsmall", "គ្រាប់សំណតូច", lab_small, edge(small, R_SMALL, lab_small), 46)
            name("lbig", "គ្រាប់សំណធំ", lab_big, edge(big, R_BIG, lab_big), 36)
            name("lrod", "ដងព្យួរ", (rod_pt[0] - 25, rod_pt[1] + 62), rod_pt, 20)
            name(
                "lwire",
                "ខ្សែព្យួរ",
                (APP_C[0] + 48, APP_C[1] + 70),
                (APP_C[0] + 3, APP_C[1] + 8),
                20,
            )
        to = a * st["torque_op"]
        if to > 0:
            curved_arrow(
                scene, ("tg",), Arc(APP_C, 66, 40, 140), Look(BLUE, to, LAY_APP + 10), 5
            )
            frac = clamp(st["rod_phi"] / st["phi_eq"], 0.0, 1.6)
            curved_arrow(
                scene,
                ("tw",),
                Arc(APP_C, 48, 140, 140 - 100 * frac),
                Look(ORANGE, to, LAY_APP + 10),
                5,
            )
            # Beside the arcs, not out by the big balls — out at ±150 the wire
            # label sat behind the right-hand big ball.
            scene.formula(
                ("tg", "lab"),
                r"\tau_{\text{gravity}}",
                size=30,
                at=(APP_C[0] - 95, APP_C[1] - 105),
            ).fill(BLUE).on(layer=LAY_APP + 12, opacity=to)
            scene.formula(
                ("tw", "lab"),
                r"\tau_{\text{wire}}",
                size=30,
                at=(APP_C[0] + 95, APP_C[1] - 105),
            ).fill(ORANGE).on(layer=LAY_APP + 12, opacity=to)
        if st["eq_op"] > 0:
            scene.formula(
                ("eq",),
                r"\tau_{\text{gravity}}=\tau_{\text{wire}}",
                size=44,
                at=(640, 52),
            ).fill(INK).on(layer=LAY_APP + 12, opacity=a * st["eq_op"])
        if st["theta_op"] > 0:
            scene.formula(
                ("theta",), r"\theta \approx 0.001\ \text{rad}", size=44, at=(640, 52)
            ).fill(YELLOW).on(layer=LAY_APP + 12, opacity=a * st["theta_op"])

    draw_chain(scene, st)

    L5 = LAY_S5 + 20
    if st["eqG_op"] > 0:
        scene.formula(
            ("eqG",), r"G=\frac{F\,r^{2}}{m_1 m_2}", size=52, at=(640, 300)
        ).fill(INK).on(layer=L5, opacity=st["eqG_op"])
    if st["eqg_op"] > 0:
        scene.formula(
            ("eqg",), r"g=\frac{G\,M_{⊕}}{R_{⊕}^{2}}", size=40, at=(640, 230)
        ).fill(INK).on(layer=L5, opacity=st["eqg_op"])
    if st["eqM_op"] > 0:
        scene.formula(
            ("eqM",), r"M_{⊕}=\frac{g\,R_{⊕}^{2}}{G}", size=40, at=(640, 345)
        ).fill(INK).on(layer=L5, opacity=st["eqM_op"])
    if st["eqNum_op"] > 0:
        scene.formula(
            ("eqNum",),
            r"M_{⊕}=\frac{9.8\times\left(6.37\times10^{6}\right)^{2}}{6.67\times10^{-11}}",
            size=40,
            at=(640, 345),
        ).fill(INK).on(layer=L5, opacity=st["eqNum_op"])
    chips = [
        (r"g = 9.8\ \text{m/s}^2", 260),
        (r"R_{⊕} = 6.37\times10^{6}\ \text{m}", 640),
        (r"G = 6.67\times10^{-11}\ \text{N}\,\text{m}^2/\text{kg}^2", 1010),
    ]
    for i, (latex, x) in enumerate(chips):
        a = st[f"chip{i + 1}"] * (1 - st["chip_t"])
        if a > 0:
            # They rise toward the equation and fade; they never gather on
            # one point, where three plates sat on each other.
            plate(
                scene,
                ("chip", i),
                latex,
                (x, lerp(470, 430, st["chip_t"])),
                Look(GREEN, a, L5 + 2, 28),
            )
    if st["result_op"] > 0:
        scene.formula(
            ("result",),
            r"M_{⊕}\approx 5.97\times10^{24}\ \text{kg}",
            size=62,
            at=(640, 460),
        ).fill(YELLOW).on(layer=L5 + 4, opacity=st["result_op"])
    if st["note_op"] > 0:
        scene.text(
            ("note",),
            "(កាវេនឌីសវាស់ដង់ស៊ីតេមធ្យមរបស់ផែនដី។ គេគណនា G ពីលទ្ធផលនេះនៅពេលក្រោយ។)",
            size=22,
            at=cm.at(x=640, top=535),
        ).fill(DIM).on(layer=L5 + 4, opacity=st["note_op"])

    if st["zoom_op"] > 0:
        z = st["zoom"]
        zo = st["zoom_op"]
        draw_city(scene, 64 * z, lvl_op(64 * z) * zo)
        draw_lab(scene, 8 * z, lvl_op(8 * z) * zo)
        draw_apparatus(scene, "zapp", Pose(ZC, z, PHI_REAL), lvl_op(z) * zo, LAY_APP)
    if st["final_op"] > 0:
        scene.formula(
            ("final",),
            r"M_{⊕}\approx 5.97\times10^{24}\ \text{kg}",
            size=70,
            at=(640, 470),
        ).fill(YELLOW).on(layer=L5 + 5, opacity=st["final_op"])

    scene.rect(("mask",), w=1280, h=150, at=(640, 645)).fill("#000000").on(
        layer=LAY_PLATE, opacity=0.94
    )
    draw_caption(scene, st["cap_line"], st["cap_said"])
    return scene


def draw_caption(scene, line, said):
    """The subtitle on its plate, with the mark on the word being said. A
    line over `PAGE` words turns a page rather than shrinking to fit."""
    pieces = chunks(line) if line else []
    if not pieces:
        return
    page = max(said - 1, 0) // PAGE
    shown = pieces[page * PAGE : (page + 1) * PAGE]
    here = (said - 1) - page * PAGE

    size = fit(" ".join(p for p, _ in shown), SAY_SIZE)
    space = cm.measure(" ", size=size)[0]
    widths = [cm.measure(p, size=size)[0] for p, _ in shown]
    # Khmer sets its words flush: only a space the author typed is a space.
    leads = [0.0] + [space if gap else 0.0 for _, gap in shown[1:]]
    centres, run = [], 0.0
    for lead, wide in zip(leads, widths):
        run += lead
        centres.append(run + wide / 2)
        run += wide
    left = 640.0 - run / 2

    scene.rect(
        ("say_plate", line, page), w=run + 56, h=size + 32, at=(640, SAY_Y)
    ).fill(CAPTION_BG).round(15).on(layer=LAY_PLATE + 1)
    for i, ((piece, _), centre) in enumerate(zip(shown, centres)):
        scene.text(
            ("say", page * PAGE + i, piece),
            piece,
            size=size,
            at=cm.at(x=left + centre, y=SAY_Y),
        ).fill(READ if i == here else UNREAD).on(layer=LAY_PLATE + 2)


# ------------------------------------------------------------------- story

TICK = 1.0 / 30

S5_KEYS = [
    "ch1",
    "ch2",
    "ch3",
    "ch4",
    "ca1",
    "ca2",
    "ca3",
    "eqG_op",
    "eqg_op",
    "eqM_op",
    "eqNum_op",
    "chip1",
    "chip2",
    "chip3",
    "result_op",
    "note_op",
]


def story(state, emit):
    cap = {}  # the line being read: when it began, when each word falls
    ramps = []  # background ramps: [key, from, to, ticks, done]
    osc = {"on": False, "t0": 0}

    def tick(name="t"):
        state["tick"] += 1
        state["spin"] += 0.35
        for r in ramps:
            r[4] += 1
            state[r[0]] = lerp(r[1], r[2], smooth(min(1.0, r[4] / r[3])))
        ramps[:] = [r for r in ramps if r[4] < r[3]]
        if osc["on"]:
            t = (state["tick"] - osc["t0"]) / 30.0
            state["rod_phi"] = PHI_EXAG * (
                1 - math.exp(-t / 1.2) * math.cos(2 * math.pi * t / 2.0)
            )
        if cap:
            elapsed = (state["tick"] - cap["t0"]) * TICK
            state["cap_said"] = max(1, sum(1 for s in cap["starts"] if s <= elapsed))
        emit(name)

    def to(n, **targets):
        """Ramp these in the background; the story carries on meanwhile."""
        for key, v in targets.items():
            ramps[:] = [r for r in ramps if r[0] != key]
            ramps.append([key, state[key], v, n, 0])

    def go(n, **targets):
        to(n, **targets)
        for _ in range(n):
            tick("go")

    def hold(n):
        for _ in range(n):
            tick("hold")

    def say(key):
        """Start reading a line. What the scene does while it is read is
        the next few calls; `finish()` waits out whatever is left."""
        line = SAY[key]
        starts, run = [], 0.0
        for piece, _ in chunks(line):
            starts.append(run)
            run += pace(piece)
        cap.clear()
        cap.update(t0=state["tick"], starts=starts, total=run)
        state["cap_line"], state["cap_said"] = line, 1

    def finish(tail=0.45):
        """Wait until the line is read, a breath after, then take it down."""
        while (state["tick"] - cap["t0"]) * TICK < cap["total"] + tail:
            tick("read")
        cap.clear()
        state["cap_line"] = ""

    def reading():
        return bool(cap) and (state["tick"] - cap["t0"]) * TICK < cap["total"] + 0.45

    # ------------------------------------------------ 1. the impossible question
    hold(15)
    say("rock")
    finish()
    say("person")
    finish()
    say("but_earth")
    go(25, scale_reveal=1.0)
    finish()
    say("how")
    for k in range(1, 51):
        state["tip"] = smooth(k / 50)
        (state["earth_x"], state["earth_y"]), state["earth_r"] = earth_pose(
            state["tip"]
        )
        tick("drop")
    finish()
    hold(30)

    # --------------------------------------------------------- 2. Newton's clue
    go(45, scale_reveal=0.0, earth_x=640, earth_y=880, earth_r=400)
    say("apple")
    go(15, apple_op=1.0, formula_op=1.0)
    go(10, grav_arrow_op=1.0)
    for k in range(1, 41):
        state["apple_y"] = lerp(210, 463, (k / 40) ** 2)
        tick("fall")
    finish()
    say("newton")
    go(
        55,
        earth_x=450,
        earth_y=TABLE_Y - 70,
        earth_r=70,
        apple_op=0.0,
        grav_arrow_op=0.0,
        pair_op=1.0,
        ballB_op=1.0,
        arrow_op=1.0,
        label_op=1.0,
    )
    finish()
    say("so_balls")
    go(35, earth_op=0.0, ballA_op=1.0, table_op=1.0)
    finish()
    say("pull_too")
    finish()
    hold(60)  # the moment the whole story turns on — let it sit

    # ------------------------------------------------------------ 3. the catch
    say("nothing")
    go(15, label_op=0.0)
    go(50, ballA_r=90, ballB_r=15, ballA_x=572, ballB_x=707, arrow_len=7, kg_op=1.0)
    finish()
    say("tiny")
    go(20, F_op=1.0)
    finish()
    hold(40)

    # --------------------------------------------------- 4. the torsion balance
    go(30, pair_op=0.0, formula_op=0.0, F_op=0.0)
    go(25, app_op=1.0, applabel_op=1.0)
    say("michell")
    finish()
    say("torsion")
    finish()
    say("cavendish")
    finish()
    say("big_pulls")
    go(50, big_t=1.0)
    go(15, force_op=1.0)
    finish()

    # The rod swings in the background — turn, twist, settle — while the
    # four lines that describe it are read.
    state["phi_eq"] = PHI_EXAG
    osc["on"], osc["t0"] = True, state["tick"]
    say("rod_turns")
    finish()
    say("wire_twists")
    to(40, torque_op=1.0)
    finish()
    say("back")
    finish()
    say("equal")
    to(30, eq_op=1.0)
    finish()
    osc["on"] = False
    state["rod_phi"] = PHI_EXAG
    hold(45)

    go(35, torque_op=0.0, eq_op=0.0, big_t=0.0, rod_phi=0.0, force_op=0.0)
    say("real")
    go(45, big_t=1.0, force_op=1.0)
    go(45, rod_phi=PHI_REAL)
    to(15, theta_op=1.0)
    finish()
    say("measurable")
    finish()
    hold(20)

    # --------------------------------------- 5. from a tiny twist to the planet
    go(30, app_op=0.0, theta_op=0.0, force_op=0.0, applabel_op=0.0)
    say("angle_force")
    go(15, ch1=1.0)
    go(15, ca1=1.0, ch2=1.0)
    finish()
    say("force_G")
    go(15, ca2=1.0, ch3=1.0)
    go(15, eqG_op=1.0)
    finish()
    go(15, eqG_op=0.0)
    say("G_earth")
    go(15, ca3=1.0, ch4=1.0)
    go(15, eqg_op=1.0)
    finish()
    go(15, eqM_op=1.0)
    hold(30)
    go(12, chip1=1.0)
    hold(10)
    go(12, chip2=1.0)
    hold(10)
    go(12, chip3=1.0)
    hold(25)
    go(30, chip_t=1.0, eqM_op=0.0)
    say("result")
    go(20, eqNum_op=1.0)
    finish()
    state["earth_x"], state["earth_y"], state["earth_r"] = 1110, 330, 85
    say("result2")
    go(25, result_op=1.0, earth_op=1.0)
    finish()
    go(20, note_op=1.0)
    hold(110)

    # ------------------------------------------------------ the zoom-out ending
    # The chips are already gone (chip_t = 1). Zero them before chip_t goes
    # back to 0, or they pop back into view and fade out under the result.
    state["chip1"] = state["chip2"] = state["chip3"] = 0.0
    go(30, **{k: 0.0 for k in S5_KEYS}, earth_op=0.0, chip_t=0.0)
    state["earth_x"], state["earth_y"], state["earth_r"] = ZC[0], ZC[1], 300 * 512
    go(25, zoom_op=1.0)
    end_log = math.log(1 / 512)
    pending = ["end2", "end3"]
    say("end1")
    n = 400
    for k in range(1, n + 1):
        z = math.exp(end_log * smooth(k / n))
        state["zoom"] = z
        f3 = z * 512
        state["earth_r"] = 300 * f3
        state["earth_op"] = lvl_op(f3)
        if pending and not reading():
            say(pending.pop(0))
        tick("zoom")
    finish()
    go(50, zoom_op=0.0, earth_x=640, earth_y=235, earth_r=135)
    go(30, final_op=1.0)
    hold(150)


def initial():
    keys = [
        "earth_op",
        "scale_reveal",
        "tip",
        "apple_op",
        "formula_op",
        "grav_arrow_op",
        "pair_op",
        "table_op",
        "ballA_op",
        "ballB_op",
        "arrow_op",
        "label_op",
        "kg_op",
        "F_op",
        "app_op",
        "big_t",
        "rod_phi",
        "force_op",
        "torque_op",
        "eq_op",
        "theta_op",
        "applabel_op",
        "chip_t",
        "zoom_op",
        "final_op",
        "spin",
        "phi_eq",
    ] + S5_KEYS
    st = {k: 0.0 for k in keys}
    st.update(
        {
            "tick": 0,
            "earth_op": 1.0,
            "earth_x": HERO_CENTER[0],
            "earth_y": HERO_CENTER[1],
            "earth_r": HERO_R,
            "apple_y": 210.0,
            "arrow_len": 85.0,
            "zoom": 1.0,
            "ballA_x": 450.0,
            "ballA_r": 70.0,
            "ballB_x": 830.0,
            "ballB_r": 70.0,
            "phi_eq": PHI_EXAG,
            "cap_line": "",
            "cap_said": 1,
        }
    )
    return st


cm.explain(
    trace=cm.trace(story, initial()),
    view=view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=TICK, opening=0.4, final_hold=2.0),
).render("results/cavendish.mp4", fps=30, scale=1.5)

print("wrote results/cavendish.mp4")
