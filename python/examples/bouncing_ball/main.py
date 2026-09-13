"""A bouncing ball whose sampled positions come from gravity and collisions."""

import math

import codimate as cm


DT = 1 / 30
DURATION = 6.0
GRAVITY = 900.0
FLOOR = 560.0
RESTITUTION = 0.72
REST_SPEED = 90.0


def advance(ball):
    """Advance one frame, resolving a floor collision at its exact time."""
    if ball["resting"]:
        return True

    y = ball["y"]
    velocity = ball["velocity"]
    next_y = y + velocity * DT + 0.5 * GRAVITY * DT**2

    if next_y < FLOOR:
        ball["y"] = next_y
        ball["velocity"] = velocity + GRAVITY * DT
        return False

    time_to_impact = (
        -velocity + math.sqrt(velocity**2 + 2 * GRAVITY * (FLOOR - y))
    ) / GRAVITY
    impact_velocity = velocity + GRAVITY * time_to_impact
    if impact_velocity < REST_SPEED:
        ball["y"] = FLOOR
        ball["velocity"] = 0.0
        ball["resting"] = True
        return True

    rebound_velocity = -RESTITUTION * impact_velocity
    remaining = DT - time_to_impact

    ball["y"] = FLOOR + rebound_velocity * remaining + 0.5 * GRAVITY * remaining**2
    ball["velocity"] = rebound_velocity + GRAVITY * remaining
    return False


@cm.trace()
def bounce(ball):
    for _ in range(round(DURATION / DT)):
        resting = advance(ball)
        cm.emit("settle" if resting else "tick")
        if resting:
            break


def bouncing_ball_view(frame):
    scene = cm.Scene()
    scene.text("title", "Gravity. Collision. Energy loss.", size=34,
               at=(640, 90)).fill("#e8eef7")
    scene.line("floor", start=(220, FLOOR + 36), end=(1060, FLOOR + 36),
               w=4).fill("#465161")
    scene.circle("ball", r=34,
                 at=(640, frame.state["y"])).fill("#f5a623")
    return scene


cm.explain(
    trace=bounce({"y": 180.0, "velocity": 0.0, "resting": False}),
    view=bouncing_ball_view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=DT, opening=1.0, final_hold=1.5),
).render("results/bouncing_ball.mp4", fps=60, scale=1.5)

print("wrote results/bouncing_ball.mp4")
