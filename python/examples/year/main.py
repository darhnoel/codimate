"""One year of the Sun, Earth and Moon, as it was in 2025.

    .venv/bin/python python/examples/year/main.py

365 days in 23 seconds, from 1 January round to the next. The motion is real —
eccentricity, the date of closest approach, the Moon's tilted and turning orbit,
its phases, the Earth's leaning axis — all from `sky.py`. The *sizes* are not:
the Earth is a speck 12,000 km across on an orbit 150 million km wide, and
drawn true to scale there would be nothing to see, so the bodies and the Moon's
distance are exaggerated and the film says so.

Small points on the orbit mark where something happens. The year runs until the
Earth reaches one, then stops while a line says what is special about the place,
and runs on.
"""

import json
import math
import sys
from dataclasses import replace
from datetime import timedelta
from pathlib import Path

import caption
import codimate as cm
import lines
import sky
from codimate import science

# Spoken if `speak.py` has recorded the captions, and `--silent` skips the voice.
# The voice is an addition: the film is the same, only paced by reading, without it.
AUDIO = Path(__file__).resolve().parent / "audio"
SILENT = "--silent" in sys.argv
OUT = "results/year-silent.mp4" if SILENT else "results/year.mp4"
TAIL = 0.5                      # a moment after the voice stops, before the next
_spoken = AUDIO / "narration.json"
VOICE = ({entry["text"]: entry for entry in json.loads(_spoken.read_text())}
         if _spoken.exists() and not SILENT else {})

FPS = 60
DAYS_PER_SECOND = 16
DAYS = 365

A_EARTH = 1500.0                # one AU, in drawing units
A_MOON = 420.0                  # exaggerated: really 0.0026 AU
R_SUN, R_EARTH = 220.0, 160.0
R_MOON = R_EARTH * 1737.4 / 6371.0
# Degrees a day that each body turns *on the screen*. Not the real rates — the Sun
# turns every 25 days and the Earth every one, which at 16 days a second is a turn
# every 1.6 s and 16 turns a second — but slow enough to be seen. The film says so.
SUN_SPIN, EARTH_SPIN = 2.8, 7.5
# The Moon's turn *is* real: it keeps one face to the Earth, so it turns once a
# sidereal month, 13.2 degrees a day, which is a gentle 3.5 degrees a frame.
MOON_SPIN = 360.0 / 27.3217

SUN, EARTH, MOON = "#ff8c1a", "#58c4dd", "#b8b8b8"
MONTHS = ("មករា", "កុម្ភៈ", "មីនា", "មេសា", "ឧសភា", "មិថុនា",
          "កក្កដា", "សីហា", "កញ្ញា", "តុលា", "វិច្ឆិកា", "ធ្នូ")
SNAP, FADE, OUT_FADE = 0.02, 0.2, 0.25   # a mark switching; a caption arriving, leaving
HOLDS = {}                           # event name -> seconds; filled in by `story`
ARRIVALS = sky.arrivals()


def where(day):
    """The Earth, in drawing units, on `day`."""
    anomaly, e, perihelion = sky.earth(day)
    return science.Orbit(a=A_EARTH, e=e, periapsis=perihelion).point(anomaly)


MARKS = {name: where(day) for day, name in ARRIVALS}


def khmer(number) -> str:
    return str(number).translate(str.maketrans("0123456789", "០១២៣៤៥៦៧៨៩"))


# --- the algorithm: one moment per frame --------------------------------------

def story(state, emit):
    def read(key):
        """Stop and say what this place is: each word is two beats, the mark
        switching and then holding for as long as the word takes to say. One
        moment per word, not per frame — a held picture costs nothing.

        With a recording, the words are stretched over its length, so the mark
        is on the word being said; the recording starts with the first beat."""
        line = lines.SAY[key]
        pieces = [piece for piece, _ in caption.chunks(line)]
        steps = [caption.pace(piece) for piece in pieces]
        heard = VOICE.get(line.replace(caption.ZWSP, ""))
        if heard:
            stretch = (heard["seconds"] + TAIL) / sum(steps)
            steps = [step * stretch for step in steps]
        state["line"] = key
        for i, step in enumerate(steps):
            state["said"] = i + 1
            HOLDS[f"{key}.{i}!"] = FADE if i == 0 else SNAP
            cue = {"sound": AUDIO / heard["file"]} if heard and i == 0 else {}
            emit(f"{key}.{i}!", **cue)
            HOLDS[f"{key}.{i}"] = max(step - SNAP, 0.01)
            emit(f"{key}.{i}")
        state["line"], state["said"] = "", 0
        HOLDS[f"{key}.out"] = OUT_FADE
        emit(f"{key}.out")

    read("open")
    read("scale")
    read("spin")
    waiting = list(ARRIVALS)
    total = round(DAYS * FPS / DAYS_PER_SECOND)
    for tick in range(total + 1):
        day = DAYS * tick / total
        while waiting and waiting[0][0] <= day:
            # Land on the place itself, not a frame short of it.
            state["day"], state["at"] = waiting[0][0], waiting[0][1]
            emit("frame")
            read(waiting.pop(0)[1])
            state["at"] = ""
        state["day"] = day
        emit("frame")


# --- the view: one moment as a picture ----------------------------------------

def view(frame):
    day = frame.state["day"]
    scene = cm.Scene()

    anomaly, e, perihelion = sky.earth(day)
    year = science.Orbit(a=A_EARTH, e=e, periapsis=perihelion,
                         hide=("sun", "earth", "moon"))
    earth_at = year.point(anomaly)

    m_anomaly, m_perigee, m_node = sky.moon(day)
    month = science.Orbit(a=A_MOON, e=sky.MOON_E, inclination=sky.MOON_INCLINATION,
                          periapsis=m_perigee, node=m_node, around=earth_at,
                          hide=("earth", "moon"))
    moon_at = month.point(m_anomaly)

    cam = science.Camera(scale=0.2, distance=4800.0,
                         centre=(cm.width() / 2, 350.0))
    world = science.World(cam, light=science.Light(source=(0.0, 0.0, 0.0),
                                                   ambient=0.22))
    # The axis leans toward longitude 90 and stays pointing there all year; a
    # negative `tilt` is how Sphere says that way round.
    bodies = {
        "sun": science.Sphere((0.0, 0.0, 0.0), R_SUN, SUN, "star",
                              spin=SUN_SPIN * day, detail=2),
        "earth": science.Sphere(earth_at, R_EARTH, EARTH, spin=EARTH_SPIN * day,
                                tilt=-sky.OBLIQUITY),
        "moon": science.Sphere(moon_at, R_MOON, MOON, spin=MOON_SPIN * day, detail=2),
    }
    world.draw(scene, bodies, {"year": year, "month": month})

    # Arrows round the Earth's equator, turning with it: a plain ball shows no
    # spin, and the ring leans with the axis, which is the tilt made visible.
    # Sized to each ball, which is 9 to 46 pixels across: the kit's defaults are
    # for one ten times that.
    science.RingArrows(color="#ffd9a8", count=8, span=34, w=2, head=9).draw(
        scene, world, "sun-turns", bodies["sun"], SUN_SPIN * day)
    science.RingArrows(span=34, w=1.5, head=7).draw(
        scene, world, "earth-turns", bodies["earth"], EARTH_SPIN * day)
    science.RingArrows(color="#e2e8f0", count=4, span=40, w=1, head=3.5).draw(
        scene, world, "moon-turns", bodies["moon"], MOON_SPIN * day)

    # The Earth's axis, stuck out of both poles: seasons are this line staying
    # still while the Earth goes round.
    axis = bodies["earth"].axis
    for key, sign, color in (("north", 1, "#f1f5f9"), ("south", -1, "#64748b")):
        end = tuple(c + sign * 1.6 * R_EARTH * a for c, a in zip(earth_at, axis))
        if cam.hidden_by(end, earth_at, R_EARTH):
            continue
        start = tuple(c + sign * R_EARTH * a for c, a in zip(earth_at, axis))
        scene.line(("axis", key), start=cam.at(start), end=cam.at(end), w=3) \
            .fill(color).on(layer=science.ABOVE)

    # Small points where something happens. The one the Earth has just reached
    # swells; the line that says why is the caption below.
    for name, position in MARKS.items():
        here = name == frame.state["at"]
        scene.circle(("mark", name), r=7 if here else 3.5, at=cam.at(position)) \
            .fill("#f1f5f9" if here else "#64748b").on(layer=-1)

    # Titles, without plates: the date and how far the Earth is from the Sun
    # just then (in real km), stacked and centred at the top of the screen.
    today = sky.START + timedelta(days=min(int(day), DAYS - 1))
    km = math.dist(earth_at, (0.0, 0.0, 0.0)) / A_EARTH * sky.AU_KM / 1e6
    clock = science.tag(f"{khmer(today.day)} {MONTHS[today.month - 1]} "
                        f"{khmer(today.year)}", at=cm.at(x=0, top=18), size=30,
                        formula=False).bare()
    far = science.tag(f"ចម្ងាយពីព្រះអាទិត្យ {khmer(f'{km:.1f}')} លានគីឡូម៉ែត្រ",
                      at=cm.at(x=0, top=60), size=20, color="#93a0b2",
                      formula=False).bare()
    placed = []
    for box, key in ((clock, "date"), (far, "distance")):
        box = replace(box, x=cm.width() / 2)         # centred on the screen
        box.draw(scene, ("hud", key), layer=science.ABOVE + 20)
        placed.append(box)

    # A name over each body. They follow their bodies, so one meets another or
    # the corner sooner or later; `clear_of` moves the later one the shortest
    # way out and slides it rather than jumping.
    for key, words, color in (("sun", "ព្រះអាទិត្យ", SUN), ("earth", "ផែនដី", EARTH),
                              ("moon", "ព្រះចន្ទ", MOON)):
        body = bodies[key]
        x, y = cam.at((body.centre[0], body.centre[1], body.centre[2] + body.r))
        box = science.tag(words, at=cm.at(x=x, bottom=y - 12), color=color, size=26,
                          formula=False).bare().clear_of(*placed)
        box.draw(scene, ("name", key), layer=science.ABOVE + 10)
        placed.append(box)

    caption.draw(scene, lines.SAY.get(frame.state["line"], ""), frame.state["said"],
                 y=cm.height() - 60, layer=science.ABOVE + 30)
    return scene


cm.explain(
    trace=cm.trace(story, {"day": 0.0, "line": "", "said": 0, "at": ""}),
    view=view,
    # Every moment is already a frame apart, so nothing needs easing between.
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=1 / FPS, events=HOLDS, opening=0.4, final_hold=1.5),
).render(OUT, fps=FPS, scale=1.5)

print(f"wrote {OUT}")
