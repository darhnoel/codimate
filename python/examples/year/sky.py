"""Where the Sun, Earth and Moon are on a given day of 2025 — and nothing about
how they are drawn.

Mean orbital elements from Meeus, *Astronomical Algorithms* (2nd ed., ch. 25 and
47), as functions of `T`, Julian centuries since J2000. Each is an angle, in
degrees, measured from the March equinox along the ecliptic. Fed to Kepler's
equation they put the Earth within a degree of the truth and the Moon within a
degree or two (the Moon is pulled about by the Sun; its largest wobbles, the
evection and the variation, are left out).
"""

import math
from datetime import date

START = date(2025, 1, 1)
JD_START = 2460676.5            # Julian date of 2025-01-01 00:00 UTC
AU_KM = 149_597_870.7
OBLIQUITY = 23.4393             # Earth's axis from the ecliptic's pole
MOON_E = 0.0549
MOON_INCLINATION = 5.145        # the Moon's orbit from the ecliptic


def centuries(day: float) -> float:
    """Julian centuries since J2000 at `day` days after the start."""
    return (JD_START + day - 2451545.0) / 36525.0


def earth(day: float) -> "tuple[float, float, float]":
    """`(mean anomaly, eccentricity, longitude of perihelion)` of the Earth's
    orbit round the Sun — the last as the Earth sees it from the Sun."""
    t = centuries(day)
    anomaly = 357.52911 + 35999.05029 * t - 0.0001537 * t * t
    e = 0.016708634 - 0.000042037 * t
    perihelion = 102.93735 + 1.71946 * t       # Sun's 282.9373, turned half a round
    return anomaly % 360.0, e, perihelion % 360.0


def moon(day: float) -> "tuple[float, float, float]":
    """`(mean anomaly, argument of perigee, longitude of the node)` of the
    Moon's orbit round the Earth. The perigee runs round once in 8.85 years and
    the node backwards once in 18.6, so over a year each has moved by tens of
    degrees, and the orbit it draws is not the one it started with."""
    t = centuries(day)
    longitude = 218.3164477 + 481267.88123421 * t
    anomaly = 134.9633964 + 477198.8675055 * t
    node = 125.0445479 - 1934.1362891 * t
    perigee = longitude - anomaly              # longitude of perigee
    return anomaly % 360.0, (perigee - node) % 360.0, node % 360.0


def arrivals(days: int = 365) -> "list[tuple[float, str]]":
    """When the Earth reaches each place on its orbit worth stopping at, as
    `(day, name)` in order: closest and furthest from the Sun, and the four
    longitudes where the seasons turn."""
    from codimate import science

    def at(day):
        anomaly, e, perihelion = earth(day)
        orbit = science.Orbit(a=1.0, e=e, periapsis=perihelion)
        x, y, _ = orbit.point(anomaly)
        return anomaly, math.degrees(math.atan2(y, x)) % 360.0

    found, step = [], 0.05
    before = at(0.0)
    for i in range(1, int(days / step) + 1):
        now = at(i * step)
        for name, longitude in (("march", 180.0), ("june", 270.0),
                                ("september", 0.0), ("december", 90.0)):
            gone = (before[1] - longitude + 180.0) % 360.0 - 180.0
            come = (now[1] - longitude + 180.0) % 360.0 - 180.0
            if gone < 0 <= come:
                found.append(((i - 1 + -gone / (come - gone)) * step, name))
        for name, anomaly in (("perihelion", 0.0), ("aphelion", 180.0)):
            gone = (before[0] - anomaly + 180.0) % 360.0 - 180.0
            come = (now[0] - anomaly + 180.0) % 360.0 - 180.0
            if gone < 0 <= come:
                found.append(((i - 1 + -gone / (come - gone)) * step, name))
        before = now
    return sorted(found)
