"""A swinging pendulum — continuous motion from sampled physics.

    .venv/bin/python python/examples/pendulum/main.py

The simulation advances at a fixed time step and records one moment per tick.
Codimate connects those closely spaced moments with linear motion; it does not
need a pendulum-specific animation primitive.
"""

import math

import codimate as cm


# --- the simulation ---------------------------------------------------------

DT = 1 / 30
DURATION = 10.0

GRAVITY = 9.81
LENGTH_METRES = 1.0
DAMPING = 0.08

PIVOT = (640.0, 145.0)
ROD_LENGTH = 240.0
SECTOR_RADIUS = 92.0
SECTOR_STEPS = 16

HIGH_AMPLITUDE = (92, 208, 132)
LOW_AMPLITUDE = (239, 92, 90)
# Mild damping takes the amplitude from 50 degrees to about 34 degrees during
# this lesson. Stretch that observed loss across the full colour range so the
# change reads on screen; this affects only the display, never the physics.
VISIBLE_DECAY = 0.33


def mix_color(start, end, progress):
    channels = (
        round(a + (b - a) * progress)
        for a, b in zip(start, end)
    )
    return "#" + "".join(f"{channel:02x}" for channel in channels)


def point_on_swing(angle, radius):
    return (
        PIVOT[0] + radius * math.sin(angle),
        PIVOT[1] + radius * math.cos(angle),
    )


def sector_points(angle):
    # A fixed corner count lets Codimate morph the filled sector every tick.
    return [PIVOT] + [
        point_on_swing(angle * step / SECTOR_STEPS, SECTOR_RADIUS)
        for step in range(SECTOR_STEPS + 1)
    ]


class Pendulum:
    def __init__(self):
        self.angle = math.radians(50.0)
        self.angular_velocity = 0.0
        self.time = 0.0
        self.inspecting = False

    def step(self):
        acceleration = (
            -(GRAVITY / LENGTH_METRES) * math.sin(self.angle)
            - DAMPING * self.angular_velocity
        )

        # Semi-implicit Euler: update velocity before position. It is nearly
        # as small as Euler's method, but behaves much better for oscillation.
        self.angular_velocity += acceleration * DT
        self.angle += self.angular_velocity * DT
        self.time += DT

    def bob_position(self):
        return point_on_swing(self.angle, ROD_LENGTH)

    def amplitude(self):
        """The turning angle a frictionless pendulum has at this energy."""
        kinetic = 0.5 * (LENGTH_METRES * self.angular_velocity) ** 2
        potential = GRAVITY * LENGTH_METRES * (1.0 - math.cos(self.angle))
        ratio = (kinetic + potential) / (GRAVITY * LENGTH_METRES)
        return math.acos(1.0 - min(max(ratio, 0.0), 2.0))


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def swing(pendulum):
    inspected = False
    for _ in range(round(DURATION / DT)):
        previous_velocity = pendulum.angular_velocity
        pendulum.step()

        # At the first far-side turning point, stop the physics long enough to
        # inspect how the angle is measured. Separate events give the camera
        # time to move in, hold the close-up, and return to the overview.
        reached_turning_point = (
            not inspected
            and previous_velocity < 0.0 <= pendulum.angular_velocity
        )
        if reached_turning_point:
            pendulum.inspecting = True
            cm.emit("zoom_in")
            cm.emit("inspect_angle")
            pendulum.inspecting = False
            cm.emit("zoom_out")
            inspected = True
            continue

        cm.emit("tick")
    cm.emit("done")


# --- the view ---------------------------------------------------------------


def pendulum_view(frame):
    scene = cm.Scene()
    hud = scene.overlay()
    pendulum = frame.state
    bob = pendulum.bob_position()
    initial_amplitude = math.radians(50.0)
    decay = 1.0 - min(pendulum.amplitude() / initial_amplitude, 1.0)
    color_progress = min(decay / VISIBLE_DECAY, 1.0)
    angle_color = mix_color(HIGH_AMPLITUDE, LOW_AMPLITUDE, color_progress)

    # The title is a fixed heading, independent of the camera underneath it.
    hud.text("title", "A Simple Pendulum", size=38,
             at=(640, 62)).fill("#e8eef7")

    # A quiet equilibrium marker makes the shrinking amplitude visible.
    scene.line("equilibrium", start=PIVOT, end=(PIVOT[0], PIVOT[1] + ROD_LENGTH + 48),
               w=1).fill("#303947")
    scene.polygon("angle_sector", sector_points(pendulum.angle)).fill(angle_color,
                  edge=angle_color, edge_w=2).on(opacity=0.55)
    scene.line("string", start=PIVOT, end=bob, w=5).fill("#aab4c3").on(layer=1)
    scene.circle("pivot", r=10, at=PIVOT).fill("#e8eef7").on(layer=2)
    scene.circle("bob", r=30, at=bob).fill("#f5a623").on(layer=3)
    angle_text = f"{abs(math.degrees(pendulum.angle)):.2f} deg"
    label_position = point_on_swing(
        pendulum.angle / 2.0,
        SECTOR_RADIUS + 34,
    )
    label_width, label_height = cm.measure(angle_text, 20)
    angle_value = scene.group("angle_value", at=label_position)
    angle_value.rect("background", w=label_width + 24, h=label_height + 14,
                     at=(0, 0)).fill("#17202d", edge=angle_color,
                                     edge_w=1.5).round(8).on(layer=4)
    angle_value.text("text", angle_text, size=20,
                     at=(0, 0)).fill("#f4f7fb").on(layer=5)

    if pendulum.inspecting:
        scene.focus("angle_sector", "angle_value/background",
                    pad=45, least=300)
    else:
        scene.focus()

    # Keep the same named HUD shapes in every scene. Opacity changes produce
    # clean fades and preserve identity through the camera transition.
    overview_opacity = 0.0 if pendulum.inspecting else 1.0
    hud.formula(
        "equation",
        r"\ddot{\theta}=-\frac{g}{L}\sin(\theta)-c\dot{\theta}",
        size=27,
        at=(640, 505),
    ).fill("#8f9bad").on(opacity=overview_opacity)
    hud.text("reading", f"t = {pendulum.time:4.1f} s", size=24,
             at=(640, 560)).fill("#aab4c3").on(opacity=overview_opacity)

    message = "10 seconds of simulated motion"
    message_width, message_height = cm.measure(message, 22)
    done_opacity = 1.0 if frame.is_("done") else 0.0
    hud.rect("done_box", w=message_width + 36, h=message_height + 22,
             at=(640, 625)).fill("#17202d", edge="#6fce88",
                                 edge_w=2).round(10).on(opacity=done_opacity)
    hud.text("done", message, size=22,
             at=(640, 625)).fill("#6fce88").on(opacity=done_opacity)

    return scene


# --- motion and timing ------------------------------------------------------

# The simulation already contains the acceleration and curved trajectory.
# Linear motion joins its dense samples without adding stop-start easing.
cm.explain(
    trace=swing(Pendulum()),
    view=pendulum_view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(
        default=DT,
        events={
            "zoom_in": 1.0,
            "inspect_angle": 1.5,
            "zoom_out": 1.0,
            "done": 0.8,
        },
        opening=1.0,
        final_hold=2.0,
    ),
).render("results/pendulum.mp4", fps=60, scale=1.5)

print("wrote results/pendulum.mp4")
