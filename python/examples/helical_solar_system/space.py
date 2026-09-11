"""The solar system's measurements, and how 3D gets onto a flat screen."""

import math

# Planet, orbital radius (AU), period (Earth years), radius on screen, colour.
PLANETS = (
    ("Mercury", 0.387, 0.241, 3.5, "#9b8f83"),
    ("Venus", 0.723, 0.615, 5.0, "#d9b06a"),
    ("Earth", 1.000, 1.000, 5.2, "#4a9eff"),
    ("Mars", 1.524, 1.881, 4.2, "#c1573a"),
)

SUN_R, SUN_COLOR = 13.0, "#ffcc33"

# The Sun really covers about 49 AU per Earth year, which would make one turn
# of Earth's helix 24 times longer than it is wide — a straight line with a
# ripple. Compressed 18x so the helix is something you can see.
TRAVEL = 2.6                    # AU per Earth year, drawn

# The orbital plane is inclined about 60 degrees to the galactic plane. It is
# NOT perpendicular to the direction of travel, and the planets do not trail
# behind the Sun — half of every orbit is ahead of it.
INCLINATION = math.radians(60.0)

YEARS = 2.8
DT = 0.011                      # Earth years per tick
TRAIL = 80                      # how many ticks of trail a body keeps

# Travel is +x. The orbital plane's normal leans away from it by INCLINATION.
NORMAL = (math.cos(INCLINATION), 0.0, math.sin(INCLINATION))
PLANE_U = (0.0, 1.0, 0.0)
PLANE_V = (-math.sin(INCLINATION), 0.0, math.cos(INCLINATION))

# Orthographic camera, as three screen vectors — one per world axis. Chosen by
# searching azimuth and elevation for two things at once: a travel direction
# long enough to read (azimuth 120, elevation -60 puts it down-right at 27
# degrees) and an orbit that stays open rather than collapsing edge-on. This
# pair projects the orbital plane's two axes to 0.90 and 0.98 — near-circular.
AXIS_X = (0.866, 0.433)         # the direction of travel: right and down
AXIS_Y = (0.500, -0.750)
AXIS_Z = (0.000, -0.500)
SCALE = 150.0                   # pixels per AU
SUN_AT = (788.0, 480.0)         # the Sun's fixed place on screen

# Fixed stars, so there is something for the system to move against.
STARS = 90
STAR_SEED = 11

# The camera travels with the Sun. It keeps the system framed at a scale worth
# looking at, and it is the frame the helix is most legible in: the Sun sits
# still, the planets wind around it, and the trails stream away behind.

INK = "#e8eef7"
TRAIL_DIM = "#20304a"


def orbit_point(radius, angle):
    """A point on the orbital plane, in world coordinates relative to the Sun."""
    c, s = math.cos(angle) * radius, math.sin(angle) * radius
    return (c * PLANE_U[0] + s * PLANE_V[0],
            c * PLANE_U[1] + s * PLANE_V[1],
            c * PLANE_U[2] + s * PLANE_V[2])


def project(point, camera):
    """World (x, y, z) in AU onto the screen, from a camera travelling with
    the Sun."""
    x = point[0] - camera[0]
    y = point[1] - camera[1]
    z = point[2] - camera[2]
    return (SUN_AT[0] + (x * AXIS_X[0] + y * AXIS_Y[0] + z * AXIS_Z[0]) * SCALE,
            SUN_AT[1] + (x * AXIS_X[1] + y * AXIS_Y[1] + z * AXIS_Z[1]) * SCALE)
