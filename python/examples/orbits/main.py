"""Sun, Earth and Moon — the science kit, end to end.

    .venv/bin/python python/examples/orbits/main.py

It starts close on Earth and its Moon, then the camera backs away and pans to
the Sun, until the whole year is one ellipse. Every picture is built from
`codimate.science`: a `Camera`, three `Sphere`s, two `Orbit`s, a `World` that
draws them together, `RingArrows` so the turning can be seen, and `tag`s.
"""

import codimate as cm
from codimate import science

FPS = 60
CLOSE, FAR = 1860.0, 4800.0       # how far back the camera stands
SCALE = 0.677                     # pixels per unit at the target, close up

R_SUN, R_EARTH = 220.0, 160.0
R_MOON = R_EARTH * 1737.4 / 6371.0    # the Moon's real size beside Earth's
YEAR = science.Orbit(a=1500.0, e=0.15, hide=("sun", "earth"))
MONTH_TILT = 22.0                  # the Moon's orbit leans from Earth's equator
AXIAL_TILT = 23.5                  # Earth's axis, fixed in space: that is seasons

SECONDS = {"close": 3.0, "pull": 5.0, "wide": 4.0}
YEAR_DEGREES = 260.0               # how much of the year the film covers
MONTHS = 5                         # laps the Moon makes in that time
SPINS = 3                          # turns Earth makes

SUN, EARTH, MOON = "#ff8c1a", "#58c4dd", "#b8b8b8"


# --- the algorithm: where everything is, one moment per frame ---------------

def story(state, emit):
    # One moment per frame, not fewer. Draw order is settled once per moment,
    # so a ball turning faster than the moments arrive re-sorts in jumps.
    total = round(sum(SECONDS.values()) * FPS)
    pull_from = round(SECONDS["close"] * FPS)
    pull_len = round(SECONDS["pull"] * FPS)
    for tick in range(total):
        done = min(max((tick - pull_from) / pull_len, 0.0), 1.0)
        state["pull"] = cm.ease(done)
        state["year"] = YEAR_DEGREES * tick / total          # mean anomaly
        state["month"] = 360.0 * MONTHS * tick / total
        state["spin"] = 360.0 * SPINS * tick / total
        emit("frame")


# --- the view: one moment as a picture --------------------------------------

def view(frame):
    s = frame.state
    scene = cm.Scene()

    earth_at = YEAR.point(s["year"])
    month = science.Orbit(a=420.0, e=0.2, inclination=MONTH_TILT, around=earth_at,
                          hide=("earth", "moon"))
    moon_at = month.point(s["month"])

    # The camera starts on Earth and ends on the Sun: pan and pull back together.
    pull = s["pull"]
    target = tuple(e * (1 - pull) for e in earth_at)
    cam = science.Camera(scale=SCALE, distance=CLOSE, target=target) \
        .dolly(CLOSE + (FAR - CLOSE) * pull)
    # The Sun is the light, so the lit side follows the planet round its orbit. A
    # high ambient keeps the night side readable: this camera looks at it.
    sunlight = science.Light(source=(0.0, 0.0, 0.0), ambient=0.3)
    world = science.World(cam, light=sunlight)

    bodies = {
        "sun": science.Sphere((0.0, 0.0, 0.0), R_SUN, SUN, "star",
                              spin=s["spin"] * 0.5),
        "earth": science.Sphere(earth_at, R_EARTH, EARTH, spin=s["spin"],
                                tilt=AXIAL_TILT),
        "moon": science.Sphere(moon_at, R_MOON, MOON, tilt=MONTH_TILT),
    }
    world.draw(scene, bodies, {"year": YEAR, "month": month})

    # Arrows riding each equator, turning with the body, hidden behind it.
    science.RingArrows(color="#ffd9a8", count=8, w=3, head=11).draw(
        scene, world, "sun-turns", bodies["sun"], s["spin"] * 0.5)
    science.RingArrows().draw(scene, world, "earth-turns", bodies["earth"], s["spin"])
    science.RingArrows(color="#e2e8f0", span=26, w=2, head=7).draw(
        scene, world, "moon-turns", bodies["moon"], s["spin"] * 1.7)

    # A name standing over each body. They follow their bodies, so two of them
    # meet sooner or later; `clear_of` moves the later one the shortest way out
    # of the earlier ones, and slides it rather than jumping. The Sun's is the
    # one that never gives way. Khmer, so plain words rather than a formula.
    placed = []
    for key, words, color in (("sun", "ព្រះអាទិត្យ", SUN), ("earth", "ផែនដី", EARTH),
                              ("moon", "ព្រះចន្ទ", MOON)):
        body = bodies[key]
        x, y = cam.at((body.centre[0], body.centre[1], body.centre[2] + body.r))
        box = science.tag(words, at=cm.at(x=x, bottom=y - 12), color=color, size=26,
                          formula=False).clear_of(*placed)
        box.draw(scene, ("name", key), layer=science.ABOVE + 10)
        placed.append(box)
    return scene


cm.explain(
    trace=cm.trace(story, {"pull": 0.0, "year": 0.0, "month": 0.0, "spin": 0.0}),
    view=view,
    # Every moment is already a frame apart, so nothing needs easing between.
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=1 / FPS, opening=0.4, final_hold=1.2),
).render("results/orbits.mp4", fps=FPS, scale=1.5)

print("wrote results/orbits.mp4")
