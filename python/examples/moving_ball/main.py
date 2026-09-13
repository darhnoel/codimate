"""The smallest useful Codimate example: one thing changes position."""

import codimate as cm


@cm.trace()
def move(ball):
    ball["x"] = 980
    cm.emit("move")


def moving_ball_view(frame):
    scene = cm.Scene()
    scene.text("title", "One name. Two positions.", size=34,
               at=(640, 90)).fill("#e8eef7")
    scene.circle("ball", r=34,
                 at=(frame.state["x"], 360)).fill("#f5a623")
    return scene


cm.explain(
    trace=move({"x": 300}),
    view=moving_ball_view,
    timing=cm.Timing(default=2.0, opening=1.0, final_hold=1.5),
).render("results/moving_ball.mp4", fps=60, scale=1.5)

print("wrote results/moving_ball.mp4")
