"""Reusable visual backgrounds."""

import support  # noqa: F401
import codimate as cm


def test_dot_grid_is_centered_stable_and_below_content():
    cm.canvas(100, 60)
    try:
        scene = cm.Scene()
        cm.backgrounds.dot_grid(scene, spacing=40, color="blue")
        scene.text("title", "Hello", size=20, at=(50, 30))

        shapes = {shape["item"]: shape for shape in scene._payload()}
        dots = [shape for name, shape in shapes.items() if name.startswith("dot-grid/")]

        in_frame = [dot for dot in dots if 0 <= dot["x"] <= 100 and 0 <= dot["y"] <= 60]
        assert len(in_frame) == 6
        assert {(dot["x"], dot["y"]) for dot in in_frame} == {
            (10.0, 10.0),
            (50.0, 10.0),
            (90.0, 10.0),
            (10.0, 50.0),
            (50.0, 50.0),
            (90.0, 50.0),
        }
        assert all(dot["layer"] == -10 and dot["opacity"] == 0.28 for dot in dots)
        assert shapes["title"]["layer"] > -10

        next_scene = cm.Scene()
        cm.backgrounds.dot_grid(next_scene, spacing=40, color="blue")
        next_dots = {
            shape["item"]: (shape["x"], shape["y"]) for shape in next_scene._payload()
        }
        assert next_dots == {shape["item"]: (shape["x"], shape["y"]) for shape in dots}

        shifted_scene = cm.Scene()
        cm.backgrounds.dot_grid(
            shifted_scene, spacing=40, color="blue", offset=(12, -6)
        )
        shifted_dots = {
            shape["item"]: (shape["x"], shape["y"])
            for shape in shifted_scene._payload()
        }
        for name in next_dots.keys() & shifted_dots.keys():
            assert (
                shifted_dots[name][0] - next_dots[name][0],
                shifted_dots[name][1] - next_dots[name][1],
            ) == (12, -6)
    finally:
        cm.canvas(1280, 720)


def test_dot_grid_can_paint_a_background_under_the_dots():
    cm.canvas(100, 60)
    try:
        scene = cm.Scene()
        cm.backgrounds.dot_grid(scene, spacing=40, background="#111111")
        shapes = {shape["item"]: shape for shape in scene._payload()}

        assert shapes["dot-grid/background"]["kind"] == "rect"
        assert shapes["dot-grid/background"]["layer"] == -11
        assert all(
            shape["layer"] == -10
            for name, shape in shapes.items()
            if name.startswith("dot-grid/") and name != "dot-grid/background"
        )
    finally:
        cm.canvas(1280, 720)


def test_dot_grid_rejects_invalid_spacing():
    for options in ({"spacing": 0}, {"spacing": -1}, {"offset": (1,)}):
        try:
            cm.backgrounds.dot_grid(cm.Scene(), **options)
        except ValueError:
            continue
        raise AssertionError(f"invalid options should fail: {options}")


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
