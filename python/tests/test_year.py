"""`examples/year/sky.py` against the sky: the dates it must reproduce.

Times are from the 2025 almanac (UTC). Nothing renders here.
"""

import importlib.util
import math
from pathlib import Path

import support  # noqa: F401  (puts `codimate` on the import path)
from codimate import science

_spec = importlib.util.spec_from_file_location(
    "sky", Path(__file__).resolve().parents[1] / "examples" / "year" / "sky.py")
sky = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(sky)


def earth_at(day):
    m, e, p = sky.earth(day)
    return science.Orbit(a=1.0, e=e, periapsis=p).point(m)


def lon(p):
    return math.degrees(math.atan2(p[1], p[0])) % 360.0


def off(a, b):
    return abs((a - b + 180.0) % 360.0 - 180.0)


def test_the_earth_is_at_the_right_longitude_on_each_equinox_and_solstice():
    """Heliocentric longitude is the Sun's, plus half a turn."""
    for day, want in ((78.376, 180.0),     # 20 Mar 09:01
                      (171.113, 270.0),    # 21 Jun 02:42
                      (264.763, 0.0),      # 22 Sep 18:19
                      (354.627, 90.0)):    # 21 Dec 15:03
        assert off(lon(earth_at(day)), want) < 0.5, day


def test_closest_approach_is_in_the_first_days_of_january():
    def distance(i):
        return math.dist(earth_at(i / 100), (0, 0, 0))

    assert abs(min(range(0, 1000), key=distance) / 100 - 3.56) < 1.5, \
        "real: 4 Jan 13:28"
    assert abs(max(range(15000, 20000), key=distance) / 100 - 183.87) < 1.5, \
        "real: 3 Jul 20:55"


def test_the_moon_is_new_and_full_when_it_should_be():
    def elongation(day):
        m, perigee, node = sky.moon(day)
        here = earth_at(day)
        orbit = science.Orbit(a=0.00257, e=sky.MOON_E, inclination=sky.MOON_INCLINATION,
                              periapsis=perigee, node=node, around=here)
        seen = tuple(a - b for a, b in zip(orbit.point(m), here))
        return (lon(seen) - lon(tuple(-x for x in here))) % 360.0

    for day, want in ((28.525, 0.0),       # new, 29 Jan 12:36
                      (42.579, 180.0),     # full, 12 Feb 13:53
                      (12.94, 180.0)):     # full, 13 Jan 22:27
        assert off(elongation(day), want) < 5.0, day


def test_the_moons_orbit_does_not_stay_where_it_was():
    """Node back by ~19 degrees a year, perigee forward by ~41."""
    _, p0, n0 = sky.moon(0.0)
    _, p1, n1 = sky.moon(365.25)
    assert 15.0 < off(n0, n1) < 23.0
    assert 35.0 < off(p0 + n0, p1 + n1) < 46.0, "longitude of perigee"


def test_the_arrivals_come_in_the_order_of_the_year():
    got = sky.arrivals()
    assert [name for _, name in got] == [
        "perihelion", "march", "june", "aphelion", "september", "december"]
    days = dict((name, day) for day, name in got)
    assert abs(days["march"] - 78.376) < 0.1, "20 Mar 09:01"
    assert abs(days["june"] - 171.113) < 0.1, "21 Jun 02:42"
    assert abs(days["december"] - 354.627) < 0.1, "21 Dec 15:03"
    assert abs(days["perihelion"] - 3.56) < 1.5 and abs(days["aphelion"] - 183.87) < 1.5
