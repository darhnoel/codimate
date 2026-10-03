"""How did Maxwell find his equations? The discovery as one story.

1. The question — four equations, and one term added later
2. Electricity moves a compass — Ørsted and Ampère, 1820
3. Magnetism makes electricity — Faraday, 1831
4. Faraday's picture — fields are real, and fill space
5. The hole — Ampère's law gives two answers in a capacitor
6. Maxwell's patch — a changing E acts like a current
7. The loop never ends — each makes the next
8. The speed — the number that matched light

    .venv/bin/python python/examples/maxwell/main.py

Every equation is LaTeX, drawn as glyphs by `scene.formula` and written on the
moment its discovery happens. Maxwell's own term is a separate formula so it can
arrive alone, in orange, in scene 6. Captions are Khmer on a plate, a bright mark
stepping word by word (`lines.py`, made by `segment_lines.py`). Each scene lasts as
long as its line takes to read."""

import math
import sys
from dataclasses import dataclass
from pathlib import Path

import codimate as cm
import lines  # Khmer, word boundaries marked
from codimate import science

# The caption lives in `year` for now (ADR 0019 leaves it out of the kit).
sys.path.append(str(Path(__file__).resolve().parent.parent / "year"))
import caption  # noqa: E402

cm.canvas(1280, 720)
W, H = cm.width(), cm.height()
FPS = 30  # one moment per frame, so nothing is interpolated between two
FADE, LEAD, TAIL = 0.5, 0.4, 1.4  # a scene arriving; the caption's start; a rest

INK, DIM, TAG = "#e8eef7", "#93a0b2", "#c5d0e0"
E_COL, B_COL = "#ff9f43", "#58c4dd"
COPPER, STEEL, PANEL = "#c9885a", "#8fa3bf", "#1b2438"
RED, YELLOW, NORTH, SOUTH = "#ff6a5c", "#ffd23f", "#d64a4a", "#3f6fd6"
LAY_CAPTION = 900


@dataclass(frozen=True)
class Look:
    """How something is drawn. Grouped because it always travels together:
    colour, opacity, stroke, a type size, an arrow's head, and — for a formula —
    how much of it is written on. `edge` draws a ring instead of a disc."""

    color: str = INK
    op: float = 1.0
    w: float = 3.0
    layer: int = None
    size: float = 24.0
    head: float = 18.0
    edge: str = ""
    reveal: float = 1.0


def clamp(x, lo=0.0, hi=1.0):
    return max(lo, min(hi, x))


def ramp(u, a, b):
    """0 until `a`, 1 from `b`, easing in between."""
    x = clamp((u - a) / (b - a))
    return x * x * (3 - 2 * x)


class Stage:
    """One scene's pen. Names carry the scene, so a shape that is in scene 3 is
    not mistaken for one in scene 4; opacity carries the fade in and out."""

    def __init__(self, scene, k, op):
        self.scene, self.k, self.op = scene, k, op

    def _key(self, name):
        return (f"s{self.k}", name)

    def _done(self, handle, look):
        return handle.on(layer=look.layer, opacity=clamp(look.op) * self.op)

    def _paint(self, handle, look):
        if look.edge:
            handle.fill("none", edge=look.edge, edge_w=look.w)
        else:
            handle.fill(look.color)
        return self._done(handle, look)

    def circle(self, name, at, r, look):
        return self._paint(self.scene.circle(self._key(name), r=r, at=at), look)

    def rect(self, name, at, size, look):
        w, h = size
        return self._paint(self.scene.rect(self._key(name), w=w, h=h, at=at), look)

    def ring(self, name, at, radii, look):
        """An ellipse outline: a loop of wire, a field line round a current."""
        handle = self.scene.arc(self._key(name), r=radii, sweep=(0, 360), at=at)
        return self._done(handle.fill("none", edge=look.color, edge_w=look.w), look)

    def disc(self, name, at, radii, look):
        """An ellipse filled in: a surface stretched across a loop."""
        handle = self.scene.arc(self._key(name), r=radii, sweep=(0, 360), at=at)
        return self._done(handle.fill(look.color).round(1), look)

    def line(self, name, a, b, look):
        return self._done(
            self.scene.line(self._key(name), start=a, end=b, w=look.w).fill(look.color),
            look,
        )

    def arrow(self, name, a, b, look):
        return self._done(
            self.scene.arrow(
                self._key(name), start=a, end=b, w=look.w, head=look.head
            ).fill(look.color),
            look,
        )

    def poly(self, name, points, look):
        return self._paint(self.scene.polygon(self._key(name), points), look)

    def curve(self, name, points, look):
        return self._done(
            self.scene.curve(self._key(name), points, w=look.w).fill(look.color), look
        )

    def text(self, name, content, at, look):
        return self._done(
            self.scene.text(self._key(name), content, size=look.size, at=at).fill(
                look.color
            ),
            look,
        )

    def formula(self, name, latex, at, look):
        return self._done(
            self.scene.formula(self._key(name), latex, size=look.size, at=at)
            .fill(look.color)
            .write(reveal=look.reveal),
            look,
        )


def side_by_side(latexes, size, centre):
    """x-centres that set several formulas end to end, centred on `centre`:
    how one equation is shown in two pieces, so the second can arrive alone."""
    widths = [cm.measure_math(latex, size)[0] for latex in latexes]
    x, centres = centre - sum(widths) / 2, []
    for wide in widths:
        centres.append(x + wide / 2)
        x += wide
    return centres


def bezier(p0, p1, p2, p3, count=40):
    pts = []
    for i in range(count + 1):
        t = i / count
        a, b, c, d = (1 - t) ** 3, 3 * (1 - t) ** 2 * t, 3 * (1 - t) * t**2, t**3
        pts.append(
            (
                a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0],
                a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1],
            )
        )
    return pts


def head_at(point, direction, size=11):
    """A small triangle on a line, pointing along `direction` (a unit vector)."""
    (x, y), (dx, dy) = point, direction
    nx, ny = -dy, dx
    return [
        (x + dx * size, y + dy * size),
        (x - dx * size * 0.6 + nx * size * 0.6, y - dy * size * 0.6 + ny * size * 0.6),
        (x - dx * size * 0.6 - nx * size * 0.6, y - dy * size * 0.6 - ny * size * 0.6),
    ]


# The four equations as the film writes them, in differential form.
GAUSS = r"\nabla\cdot\mathbf{E}=\frac{\rho}{\varepsilon_0}"
NO_MONOPOLE = r"\nabla\cdot\mathbf{B}=0"
FARADAY = r"\nabla\times\mathbf{E}=-\frac{∂\mathbf{B}}{∂ t}"
AMPERE = r"\nabla\times\mathbf{B}=\mu_0\mathbf{J}"
MAXWELL_TERM = r"+\mu_0\varepsilon_0\frac{∂\mathbf{E}}{∂ t}"
AMPERE_ONLY_E = r"\nabla\times\mathbf{B}=\mu_0\varepsilon_0\frac{∂\mathbf{E}}{∂ t}"


# ---------------------------------------------------------------- the scenes
# Each draws one moment: `u` is how far through the scene (0 to 1), `t` the
# seconds since it began and `D` its length.


def scene_question(s, u, t, D):
    s.text("who", "ម៉ាក់ស្វិល", (W / 2, 78), Look(size=54))
    s.text(
        "when",
        "James Clerk Maxwell · 1861 – 1865",
        (W / 2, 130),
        Look(color=DIM, size=26),
    )
    rows = [(GAUSS, 215), (NO_MONOPOLE, 300), (FARADAY, 385)]
    for i, (latex, y) in enumerate(rows):
        s.formula(
            ("eq", i),
            latex,
            (W / 2, y),
            Look(color=DIM, size=50, reveal=ramp(u, 0.06 + 0.1 * i, 0.16 + 0.1 * i)),
        )
    size = 50
    first, second = side_by_side([AMPERE, MAXWELL_TERM], size, W / 2)
    s.formula(
        "ampere",
        AMPERE,
        (first, 505),
        Look(color=DIM, size=size, reveal=ramp(u, 0.36, 0.48)),
    )
    s.formula(
        "maxwell",
        MAXWELL_TERM,
        (second, 505),
        Look(color=E_COL, size=size, reveal=ramp(u, 0.62, 0.78)),
    )
    s.text(
        "added",
        "ធាតុដែលម៉ាក់ស្វិលបន្ថែម",
        (second, 600),
        Look(color=E_COL, size=30, op=ramp(u, 0.8, 0.9)),
    )


def scene_ampere(s, u, t, D):
    wire, compass = (380, 390), (380, 190)
    on = ramp(u, 0.22, 0.34)
    for i, radius in enumerate((70, 130, 200)):
        s.ring(
            ("field", i), wire, (radius, radius), Look(color=B_COL, w=2.5, op=on * 0.7)
        )
        for j, angle in enumerate((45, 135, 225, 315)):
            a = math.radians(angle)
            p = (wire[0] + radius * math.cos(a), wire[1] - radius * math.sin(a))
            # counter-clockwise on screen: current is coming out of the page
            s.poly(
                ("arrow", i, j),
                head_at(p, (-math.sin(a), -math.cos(a))),
                Look(color=B_COL, op=on),
            )
    s.circle("wire", wire, 30, Look(color=COPPER))
    s.circle("current", wire, 8, Look(color=INK, op=on, layer=1))
    s.text("I", "I", (wire[0] + 22, wire[1] + 62), Look(color=INK, size=34, op=on))
    # the needle starts pointing up ("north"), then swings to follow the field
    since = max(0.0, u - 0.22) * D
    swing = math.radians(90) * (1 - math.exp(-5 * since) * math.cos(9 * since))
    d = (-math.sin(swing), -math.cos(swing))
    n = (-d[1], d[0])
    s.circle("compass", compass, 44, Look(color=PANEL, layer=1))
    s.circle("rim", compass, 44, Look(edge=STEEL, w=3, layer=2))
    for name, sign, color in (("tip", 1, RED), ("tail", -1, INK)):
        end = (compass[0] + sign * d[0] * 38, compass[1] + sign * d[1] * 38)
        s.poly(
            name,
            [
                end,
                (compass[0] + n[0] * 7, compass[1] + n[1] * 7),
                (compass[0] - n[0] * 7, compass[1] - n[1] * 7),
            ],
            Look(color=color, layer=3),
        )
    s.text("tag", "អឺស្ទែត · អំពែរ", (930, 240), Look(color=TAG, size=48))
    s.formula("eq", AMPERE, (930, 345), Look(size=76, reveal=ramp(u, 0.5, 0.7)))


def magnet_x(u):
    """Where the magnet's centre is: in, held still, then out again."""
    moved = ramp(u, 0.12, 0.4) - ramp(u, 0.6, 0.88)
    return 190 + 420 * moved


COIL_X = 635  # the middle of the coil


def flux(mid):
    """How much of the magnet's field threads the coil: next to nothing while it
    is far off, most of it once it is inside."""
    return math.exp(-(((mid - COIL_X) / 120) ** 2))


def flux_rate(u):
    """How fast that is changing. A current is made by the change, not by the
    magnet moving: far from the coil the magnet can move all it likes."""
    h = 0.002
    return (flux(magnet_x(u + h)) - flux(magnet_x(u - h))) / (2 * h)


PEAK = max(abs(flux_rate(i / 400)) for i in range(401))


def scene_faraday(s, u, t, D):
    mid = magnet_x(u)
    for i in range(6):
        s.ring(
            ("loop", i), (580 + i * 22, 400), (16, 62), Look(color=COPPER, w=5, layer=3)
        )
    for name, a, b in (
        ("top-1", (580, 338), (580, 280)),
        ("top-2", (580, 280), (1010, 280)),
        ("top-3", (1010, 280), (1010, 345)),
        ("bot-1", (690, 462), (690, 540)),
        ("bot-2", (690, 540), (1010, 540)),
        ("bot-3", (1010, 540), (1010, 495)),
    ):
        s.line(name, a, b, Look(color=COPPER, w=5, layer=-1))
    # the magnet's own field, carried along with it: out of N, round, into S
    north, south = (mid + 100, 400), (mid - 100, 400)
    for k in (1, 2):
        for side in (-1, 1):
            rise = 50 * k * side
            pts = bezier(
                north,
                (north[0] + 60 * k, 400 + rise),
                (south[0] - 60 * k, 400 + rise),
                south,
            )
            look = Look(color=B_COL, w=2.5, op=0.9 - 0.25 * k, layer=1)
            s.curve(("field", k, side), pts, look)
            here, ahead = pts[20], pts[21]
            dx, dy = ahead[0] - here[0], ahead[1] - here[1]
            span = math.hypot(dx, dy)
            s.poly(
                ("field-head", k, side),
                head_at(here, (dx / span, dy / span)),
                Look(color=B_COL, op=look.op, layer=1),
            )
    s.rect("south", (mid - 50, 400), (100, 58), Look(color=SOUTH, layer=1))
    s.rect("north", (mid + 50, 400), (100, 58), Look(color=NORTH, layer=1))
    s.rect("magnet", (mid, 400), (200, 58), Look(edge=INK, w=3, layer=2)).round(4)
    s.text("S", "S", (mid - 50, 400), Look(size=34, layer=5))
    s.text("N", "N", (mid + 50, 400), Look(size=34, layer=5))
    # a galvanometer: a dial with a scale and a pivoting needle. The needle
    # follows how fast the flux through the coil is changing.
    tilt = math.radians(clamp(flux_rate(u) / PEAK, -1, 1) * 45)
    face, pivot = (1010, 420), (1010, 470)
    s.circle("meter", face, 84, Look(color=PANEL))
    s.circle("meter-rim", face, 84, Look(edge=STEEL, w=4, layer=1))
    for i, deg in enumerate(range(-45, 46, 15)):
        a = math.radians(deg)
        centre = deg == 0
        s.line(
            ("tick", i),
            (pivot[0] + 64 * math.sin(a), pivot[1] - 64 * math.cos(a)),
            (pivot[0] + 78 * math.sin(a), pivot[1] - 78 * math.cos(a)),
            Look(color=INK if centre else DIM, w=3 if centre else 2, layer=1),
        )
    s.text("meter-name", "G", (1010, 368), Look(color=DIM, size=28, layer=1))
    s.line(
        "needle",
        pivot,
        (pivot[0] + 72 * math.sin(tilt), pivot[1] - 72 * math.cos(tilt)),
        Look(color=RED, w=4, layer=2),
    )
    s.circle("pivot", pivot, 8, Look(color=STEEL, layer=3))
    s.formula("eq", FARADAY, (W / 2, 150), Look(size=76, reveal=ramp(u, 0.3, 0.52)))
    s.text("tag", "ផារ៉ាដេ", (W / 2, 44), Look(color=TAG, size=46))


def scene_field(s, u, t, D):
    north, south = (490, 360), (790, 360)
    s.rect("south", (715, 360), (150, 64), Look(color=SOUTH, layer=2))
    s.rect("north", (565, 360), (150, 64), Look(color=NORTH, layer=2))
    s.rect("magnet", (640, 360), (300, 64), Look(edge=INK, w=3, layer=4)).round(5)
    s.text("S", "S", (715, 360), Look(size=38, layer=5))
    s.text("N", "N", (565, 360), Look(size=38, layer=5))
    for k in range(1, 5):
        on = ramp(u, 0.05 + 0.08 * k, 0.15 + 0.08 * k)
        for side in (-1, 1):
            rise = 55 * k * side
            pts = bezier(
                north,
                (north[0] - 60 * k, north[1] + rise),
                (south[0] + 60 * k, south[1] + rise),
                south,
            )
            s.curve(
                ("line", k, side),
                pts,
                Look(color=B_COL, w=2.5, op=on * (1.1 - 0.15 * k), layer=1),
            )
            mid, ahead = pts[20], pts[21]
            dx, dy = ahead[0] - mid[0], ahead[1] - mid[1]
            span = math.hypot(dx, dy)
            s.poly(
                ("head", k, side),
                head_at(mid, (dx / span, dy / span)),
                Look(color=B_COL, op=on, layer=1),
            )
    s.formula("eq", NO_MONOPOLE, (W / 2, 150), Look(size=84, reveal=ramp(u, 0.5, 0.7)))
    s.text("tag", "ផារ៉ាដេ", (W / 2, 56), Look(color=TAG, size=48))
    s.text(
        "note",
        "ខ្សែដែនមិនមានទីបញ្ចប់",
        (W / 2, 590),
        Look(color=DIM, size=32, op=ramp(u, 0.72, 0.85)),
    )


PLATES = (600, 680)
WIRE_Y = 400


def capacitor(s, t):
    """A charging capacitor: wire in, two plates, wire out, and charge flowing."""
    s.line("wire-in", (150, WIRE_Y), (PLATES[0] - 7, WIRE_Y), Look(color=COPPER, w=5))
    s.line("wire-out", (PLATES[1] + 7, WIRE_Y), (1130, WIRE_Y), Look(color=COPPER, w=5))
    for i, x in enumerate(PLATES):
        s.rect(("plate", i), (x, WIRE_Y), (14, 200), Look(color=STEEL, layer=2))
    for i in range(8):
        for name, a, b in (("in", 150, PLATES[0] - 14), ("out", PLATES[1] + 14, 1130)):
            x = a + ((i / 8 + t * 0.1) % 1) * (b - a)
            s.circle(("charge", name, i), (x, WIRE_Y), 6, Look(color=YELLOW, layer=3))


RING_RADII = (22, 150)  # wide enough to slide over the plates, which are 200 tall
BAR_X, BAR_H = 1190, 200


def slide(u, a, b):
    """Where the probe ring is, sliding along the wire from `a` to `b`."""
    return 200 + 880 * clamp((u - a) / (b - a))


def real_current(x):
    """1 where a wire runs through a ring at `x`, 0 in the gap between the plates."""
    return 1 - ramp(x, 596, 604) * (1 - ramp(x, 676, 684))


def probe(s, x, patch, show=1.0):
    """A ring sliding along the wire, and a bar for the B that Ampère's law gives
    at the ring: the real current fills it blue; `patch` fills what is missing
    in the gap with Maxwell's displacement current, in orange."""
    real = real_current(x)
    s.ring("probe", (x, WIRE_Y), RING_RADII, Look(w=5, layer=5, op=show))
    s.rect("bar-frame", (BAR_X, WIRE_Y), (44, BAR_H + 6), Look(edge=DIM, w=2, op=show))
    low = WIRE_Y + BAR_H / 2
    height = BAR_H * real
    s.rect(
        "bar-real",
        (BAR_X, low - height / 2),
        (36, max(height, 0.5)),
        Look(color=B_COL, layer=1, op=show if height > 1 else 0),
    )
    extra = BAR_H * (1 - real) * patch
    s.rect(
        "bar-patch",
        (BAR_X, low - height - extra / 2),
        (36, max(extra, 0.5)),
        Look(color=E_COL, layer=1, op=show if extra > 1 else 0),
    )
    s.text(
        "bar-name",
        "B",
        (BAR_X, WIRE_Y - BAR_H / 2 - 32),
        Look(color=B_COL, size=40, op=show),
    )
    s.text(
        "bar-why",
        "តាមអំពែរ",
        (BAR_X, WIRE_Y + BAR_H / 2 + 28),
        Look(color=DIM, size=24, op=show),
    )
    return real


def scene_hole(s, u, t, D):
    """Slide a ring along the wire and ask Ampère's law for B at each place:
    steady where a wire runs through it, zero in the gap, steady again after."""
    capacitor(s, t)
    x = slide(u, 0.12, 0.82)
    real = probe(s, x, 0.0, show=ramp(u, 0.04, 0.12))
    s.text(
        "gap-tag",
        "ក្នុងចន្លោះ៖ គ្មានចរន្ត",
        (W / 2, 232),
        Look(color=RED, size=32, op=(1 - real) * 0.9),
    )
    size = 64
    first_x, second_x = side_by_side([AMPERE, "??"], size, W / 2)
    s.formula("eq", AMPERE, (first_x, 150), Look(size=size))
    s.formula(
        "query",
        "??",
        (second_x + 20, 150),
        Look(color=RED, size=size, reveal=ramp(x, 600, 650)),
    )
    s.text("tag", "កុងដង់សាទ័រ", (W / 2, 52), Look(color=TAG, size=48))


def scene_patch(s, u, t, D):
    capacitor(s, t)
    grow = ramp(u, 0.05, 0.35)
    for i, y in enumerate((WIRE_Y - 55, WIRE_Y, WIRE_Y + 55)):
        s.arrow(
            ("e", i),
            (PLATES[0] + 14, y),
            (PLATES[1] - 14, y),
            Look(color=E_COL, w=2 + 5 * grow, head=18, op=0.25 + 0.75 * grow, layer=4),
        )
    s.text("plus", "+", (PLATES[0] - 38, WIRE_Y - 125), Look(size=24 + 22 * grow))
    s.text("minus", "−", (PLATES[1] + 38, WIRE_Y - 125), Look(size=24 + 22 * grow))
    size = 64
    first, second = side_by_side([AMPERE, MAXWELL_TERM], size, W / 2)
    s.formula("eq", AMPERE, (first, 150), Look(size=size))
    s.formula(
        "term",
        MAXWELL_TERM,
        (second, 150),
        Look(color=E_COL, size=size, reveal=ramp(u, 0.2, 0.45)),
    )
    s.text(
        "e-tag",
        "E ប្តូរ",
        (W / 2, 214),
        Look(color=E_COL, size=32, op=ramp(u, 0.15, 0.3)),
    )
    probe(s, slide(u, 0.45, 0.92), 1.0, show=ramp(u, 0.38, 0.45))
    s.text(
        "disp",
        "displacement current",
        (W / 2, 598),
        Look(color=E_COL, size=34, op=ramp(u, 0.8, 0.92)),
    )


WAVE_LENGTH, WAVE_PERIOD, WAVE_AMP, WAVE_SPAN = 420.0, 3.0, 130.0, 560.0


def scene_wave(s, u, t, D):
    """E and B as one wave: two sine curves in planes at right angles, rising
    and falling together, moving along x. Drawn flat by `science.Camera`. The
    real speed is 3 × 10⁸ m/s, so the wave here crawls, and the film says so."""
    cam = science.Camera(
        azimuth=-55, elevation=25, distance=4000.0, scale=0.8, centre=(W / 2, 330)
    )
    grow = ramp(u, 0.04, 0.22)
    amp = WAVE_AMP * grow + 1.0

    def wave(x):
        return math.sin(2 * math.pi * (x / WAVE_LENGTH - t / WAVE_PERIOD))

    def e_at(x, k=1.0):
        return cam.at((x, 0.0, amp * k * wave(x) + 0.6))

    def b_at(x, k=1.0):
        return cam.at((x, amp * k * wave(x) + 0.6, 0.0))

    lo, hi = -WAVE_SPAN, WAVE_SPAN
    A = WAVE_AMP
    s.poly(
        "plane-e",
        [cam.at(p) for p in ((lo, 0, -A), (hi, 0, -A), (hi, 0, A), (lo, 0, A))],
        Look(color=E_COL, op=0.13 * grow, layer=-3),
    )
    s.poly(
        "plane-b",
        [cam.at(p) for p in ((lo, -A, 0), (hi, -A, 0), (hi, A, 0), (lo, A, 0))],
        Look(color=B_COL, op=0.13 * grow, layer=-3),
    )
    s.line("axis", cam.at((lo, 0, 0)), cam.at((hi, 0, 0)), Look(color=DIM, w=2))
    s.arrow(
        "direction",
        cam.at((hi, 0, 0)),
        cam.at((hi + 110, 0, 0)),
        Look(color=INK, w=4, head=22, op=grow),
    )
    s.formula("c", "c", cam.at((hi + 150, 0, 0)), Look(size=44, op=grow))
    for i in range(-13, 14):
        x = i * 40.0
        strength = clamp(abs(wave(x)) * 2.2) * 0.85 * grow
        s.arrow(
            ("e-stem", i),
            cam.at((x, 0, 0)),
            e_at(x),
            Look(color=E_COL, w=3, head=15, op=strength),
        )
        s.arrow(
            ("b-stem", i),
            cam.at((x, 0, 0)),
            b_at(x),
            Look(color=B_COL, w=3, head=15, op=strength),
        )
    xs = [lo + (hi - lo) * i / 90 for i in range(91)]
    s.curve("b-wave", [b_at(x) for x in xs], Look(color=B_COL, w=6, op=grow, layer=1))
    s.curve("e-wave", [e_at(x) for x in xs], Look(color=E_COL, w=6, op=grow, layer=2))
    s.formula(
        "e-name",
        r"\mathbf{E}",
        cam.at((lo - 10, 0, A + 30)),
        Look(color=E_COL, size=58, op=ramp(u, 0.2, 0.32)),
    )
    s.text(
        "tag-f", "ផារ៉ាដេ", (330, 468), Look(color=TAG, size=38, op=ramp(u, 0.3, 0.4))
    )
    s.formula("eq-f", FARADAY, (330, 540), Look(size=44, reveal=ramp(u, 0.32, 0.5)))
    s.text(
        "tag-a",
        "អំពែរ–ម៉ាក់ស្វិល",
        (950, 66),
        Look(color=TAG, size=38, op=ramp(u, 0.5, 0.6)),
    )
    s.formula(
        "eq-a", AMPERE_ONLY_E, (950, 150), Look(size=44, reveal=ramp(u, 0.52, 0.7))
    )
    s.formula(
        "b-name",
        r"\mathbf{B}",
        cam.at((hi + 30, A + 40, 0)),
        Look(color=B_COL, size=58, op=ramp(u, 0.2, 0.32)),
    )


def scene_constants(s, u, t, D):
    """What ε₀ and μ₀ are, with no numbers: each is one fixed number of nature.
    ε₀ says how hard electricity pushes, μ₀ how hard currents pull."""
    pulse = math.sin(2 * math.pi * t / 1.6)
    # ε₀ on the left
    x, show = 320, ramp(u, 0.04, 0.16)
    s.formula("sym-e", r"\varepsilon_0", (x, 110), Look(size=90, reveal=show))
    s.text(
        "name-e", "ថេរអគ្គិសនី (ពែរមីទីវីតេ)", (x, 195), Look(color=E_COL, size=32, op=show)
    )
    for side in (-1, 1):
        on = ramp(u, 0.1, 0.22)
        s.circle(("charge", side), (x + side * 100, 310), 34, Look(color=E_COL, op=on))
        s.text(
            ("plus", side),
            "+",
            (x + side * 100, 310),
            Look(color="#1b1000", size=48, op=on, layer=3),
        )
        s.arrow(
            ("push", side),
            (x + side * 146, 310),
            (x + side * (146 + 64 + 10 * pulse), 310),
            Look(color=E_COL, w=6, head=24, op=ramp(u, 0.14, 0.26)),
        )
    s.text(
        "says-e",
        "កំណត់កម្លាំងរុញរវាងបន្ទុក",
        (x, 410),
        Look(size=32, op=ramp(u, 0.22, 0.34)),
    )
    # μ₀ on the right
    x, show = 960, ramp(u, 0.42, 0.54)
    s.formula("sym-m", r"\mu_0", (x, 110), Look(size=90, reveal=show))
    s.text(
        "name-m",
        "ថេរម៉ាញេទិច (ជម្រាបសុញ្ញកាស)",
        (x, 195),
        Look(color=B_COL, size=32, op=show),
    )
    for name, y, push in (("upper", 270, 1), ("lower", 360, -1)):
        on = ramp(u, 0.48, 0.6)
        s.line(
            ("wire", name), (x - 190, y), (x + 190, y), Look(color=COPPER, w=9, op=on)
        )
        s.arrow(
            ("current", name),
            (x - 175, y),
            (x - 105, y),
            Look(color=INK, w=4, head=18, layer=3, op=on),
        )
        for side in (-1, 1):
            start = y + push * 12
            s.arrow(
                ("pull", name, side),
                (x + side * 70, start),
                (x + side * 70, start + push * (30 + 4 * pulse)),
                Look(color=B_COL, w=5, head=14, op=ramp(u, 0.52, 0.64)),
            )
    s.text(
        "says-m",
        "កំណត់កម្លាំងទាញរវាងខ្សែចរន្ត",
        (x, 410),
        Look(size=32, op=ramp(u, 0.6, 0.72)),
    )


def scene_speed(s, u, t, D):
    """One line that changes in place: the formula, then the values put in, then
    the product, the root, and the number. Each stage replaces the last where it
    stands, so the eye follows the one equation instead of reading a list."""
    values = (
        ("e", 330, r"\varepsilon_0=8.8541878\times10^{-12}\,\mathrm{F/m}", E_COL),
        ("m", 950, r"\mu_0=4\pi\times10^{-7}\,\mathrm{H/m}", B_COL),
    )
    for name, x, latex, color in values:
        s.formula(
            ("value", name),
            latex,
            (x, 55),
            Look(
                color=color,
                size=34,
                op=1 - ramp(u, 0.46, 0.54),
                reveal=ramp(u, 0.04, 0.14),
            ),
        )
    stages = (
        (0.16, r"c=\frac{1}{\sqrt{\mu_0\varepsilon_0}}"),
        (0.34, r"c=\frac{1}{\sqrt{(4\pi\times10^{-7})(8.8541878\times10^{-12})}}"),
        (0.50, r"c=\frac{1}{\sqrt{1.11265006\times10^{-17}}}"),
        (0.64, r"c=\frac{1}{3.33564095\times10^{-9}}"),
        (0.78, r"c=299\,792\,458\ \mathrm{m/s}"),
    )
    y = 370
    for i, (start, latex) in enumerate(stages):
        gone = (
            0.0
            if i == len(stages) - 1
            else ramp(u, stages[i + 1][0], stages[i + 1][0] + 0.06)
        )
        s.formula(
            ("stage", i),
            latex,
            (W / 2, y),
            Look(
                color=E_COL if i == len(stages) - 1 else INK,
                size=56,
                op=1 - gone,
                reveal=ramp(u, start, start + 0.08),
            ),
        )
    # the values travel into the formula as it fills in
    arrive = ramp(u, 0.3, 0.38) * (1 - ramp(u, 0.46, 0.52))
    for name, x, _, color in values:
        end = 520 if x < W / 2 else 760
        s.arrow(("into", name), (x, 90), (end, 250), Look(color=color, w=3, op=arrive))
    # a box drawn round the answer once it is written, settling in as it appears
    wide, high = cm.measure_math(stages[-1][1], 56)
    framed = ramp(u, 0.88, 0.95)
    s.rect(
        "frame",
        (W / 2, y),
        (wide + 80, high + 50),
        Look(edge=E_COL, w=4, op=framed),
    ).round(10).grow(1 + 0.1 * (1 - framed))


SCENES = (
    ("question", scene_question, 9.0),
    ("ampere", scene_ampere, 9.0),
    ("faraday", scene_faraday, 10.0),
    ("field", scene_field, 9.0),
    ("hole", scene_hole, 10.0),
    ("patch", scene_patch, 10.0),
    ("wave", scene_wave, 13.0),
    ("constants", scene_constants, 13.0),
    ("speed", scene_speed, 12.0),
)


def reading(key):
    """Seconds each word of the line holds the mark, as the reader needs them."""
    return [caption.pace(piece) for piece, _ in caption.chunks(lines.SAY[key])]


def length(k):
    """A scene is as long as its line takes to read, and never shorter than the
    picture needs."""
    key, _, least = SCENES[k]
    return max(least, LEAD + sum(reading(key)) + TAIL)


def said(key, t):
    """Which word is being read at `t` (1-based); 0 before the first, and the
    last stays lit once the line is done."""
    elapsed, count = t - LEAD, 0
    for step in reading(key):
        if elapsed < 0:
            break
        count += 1
        elapsed -= step
    return count


def story(state, emit):
    for k, (key, _, _) in enumerate(SCENES):
        total = round(length(k) * FPS)
        for i in range(total + 1):
            t = length(k) * i / total
            # the caption comes in just after the scene and leaves just before it
            shown = 0.2 <= t <= length(k) - 0.4
            state.update(k=k, t=t, line=key if shown else "", said=said(key, t))
            emit("tick")


def view(frame):
    state = frame.state
    k, t = state["k"], state["t"]
    D = length(k)
    scene = cm.Scene()
    SCENES[k][1](Stage(scene, k, clamp(min(t, D - t) / FADE)), t / D, t, D)
    caption.draw(
        scene,
        lines.SAY.get(state["line"], ""),
        state["said"],
        y=H - 60,
        layer=LAY_CAPTION,
    )
    return scene


cm.explain(
    trace=cm.trace(story, {"k": 0, "t": 0.0, "line": "", "said": 0}),
    view=view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=1 / FPS, opening=0.4, final_hold=0.6),
).render("results/maxwell.mp4", fps=FPS, scale=1.5)

print("wrote results/maxwell.mp4")
