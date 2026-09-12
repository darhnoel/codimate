"""The solar system's measurements, and how 3D gets onto a flat screen."""

import math

# Planet, orbital radius (AU), period (Earth years), radius on screen, colour.
PLANETS = (
    ("Mercury", 0.387, 0.241, 3.5, "#9b8f83"),
    ("Venus", 0.723, 0.615, 5.0, "#d9b06a"),
    ("Earth", 1.000, 1.000, 5.2, "#4a9eff"),
    ("Mars", 1.524, 1.881, 4.2, "#c1573a"),
)

SUN_R, SUN_COLOR = 21.0, "#ffcc33"

# The Sun really covers about 49 AU per Earth year, which would make one turn
# of Earth's helix 24 times longer than it is wide — a straight line with a
# ripple, which is why nobody draws it true.
#
# How far it is compressed decides whether you see planets ORBITING or a
# stretched spring. A coil only closes into a loop when its pitch (travel x
# the planet's period) is shorter than its orbit's diameter:
#
#     travel   Earth pitch/diameter   Mars pitch/diameter
#       2.6      2.60 / 2.0  no         4.89 / 3.0  no
#       1.2      1.20 / 2.0  yes        2.26 / 3.0  yes
#
# So 1.2, a 41x compression, and both planets visibly go round.
TRAVEL = 1.2                    # AU per Earth year, drawn

# The orbital plane is inclined about 60 degrees to the galactic plane. It is
# NOT perpendicular to the direction of travel, and the planets do not trail
# behind the Sun — half of every orbit is ahead of it.
INCLINATION = math.radians(60.0)

YEARS = 3.2
DT = 0.011                      # Earth years per tick
TRAIL = 120                     # ticks of trail: 1.3 years, so Earth closes a loop

# Travel is +x. The orbital plane's normal leans away from it by INCLINATION.
NORMAL = (math.cos(INCLINATION), 0.0, math.sin(INCLINATION))
PLANE_U = (0.0, 1.0, 0.0)
PLANE_V = (-math.sin(INCLINATION), 0.0, math.cos(INCLINATION))

# Where the camera stands. AZIMUTH is the knob for the angle the Sun travels
# across the screen; ELEVATION decides how open the orbits look rather than
# collapsed edge-on. Two things fight: a view along the Sun's path flattens the
# orbits, and a view down the orbital normal hides the travel. These were found
# by sweeping both for a pair that keeps each readable.
#
#   azimuth   travel runs at   orbit stays open
#      105       13 deg            0.92
#      120       27 deg            0.92
#      135       41 deg            0.94
#      150       56 deg            0.97
#
AZIMUTH = math.radians(135.0)
ELEVATION = math.radians(-60.0)


def _camera_axes(azimuth, elevation):
    """Three screen vectors, one per world axis, for an orthographic camera."""
    view = (math.cos(elevation) * math.cos(azimuth),
            math.cos(elevation) * math.sin(azimuth),
            math.sin(elevation))

    def cross(a, b):
        return (a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0])

    right = cross(view, (0.0, 0.0, 1.0))
    size = math.sqrt(sum(c * c for c in right))
    right = tuple(c / size for c in right)
    up = cross(right, view)

    # Screen y grows downwards, so the up vector is negated.
    return tuple((right[i], -up[i]) for i in range(3))


AXIS_X, AXIS_Y, AXIS_Z = _camera_axes(AZIMUTH, ELEVATION)

SCALE = 140.0                   # pixels per AU
SUN_AT = (640.0, 450.0)         # the Sun's fixed place on screen

# The stars are what make the travel visible, so they must not be painted on.
# The camera moves with the Sun, so the sky streams the other way — and a
# nearer star streams faster than a far one, which is parallax and is the only
# honest reason a star would move at all. NEAR and FAR are how much of the
# Sun's screen travel a star gives back; a real star's would be a millionth of
# this, but the Sun's travel is compressed 18x here too.
STARS = 110
STAR_SEED = 11
STAR_NEAR, STAR_FAR = 0.26, 0.03


def travel_on_screen(years):
    """How far the Sun has carried the camera, in pixels."""
    along = TRAVEL * years * SCALE
    return (along * AXIS_X[0], along * AXIS_X[1])

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
