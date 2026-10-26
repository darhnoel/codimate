"""An articulated Character is one named Item with a Scene-local pose."""

import json
import re

import codimate as cm


def raises(call, match):
    try:
        call()
    except ValueError as error:
        assert re.search(match, str(error)), str(error)
    else:
        raise AssertionError("expected ValueError")


def test_character_records_connected_pose_on_its_scene_item():
    scene = cm.Scene()
    guide = scene.character("guide", cm.characters.codimate, size=360, at=(640, 360))
    guide.part("right-arm").turn(-25)
    guide.expression("thinking")
    guide.theme("dark")

    [shape] = scene._payload()
    assert shape["item"] == "guide"
    assert shape["kind"] == "character"
    assert shape["text"] == "codimate:codimate-mascot"
    assert json.loads(shape["pose"]) == {
        "expression": "thinking",
        "joints": {"right-arm": -25.0},
        "theme": "dark",
    }


def test_character_rejects_non_finite_joint_angles():
    guide = cm.Scene().character("guide", cm.characters.codimate, at=(640, 360))
    raises(lambda: guide.part("right-arm").turn(float("inf")), "must be finite")


def test_character_can_cut_between_expressions_for_speech():
    scene = cm.Scene()
    guide = scene.character("guide", cm.characters.codimate, at=(640, 360))
    guide.expression("talk-open", transition="cut")

    [shape] = scene._payload()
    assert json.loads(shape["pose"]) == {
        "expression": "talk-open",
        "expression_transition": "cut",
        "joints": {},
    }


def test_character_rejects_unknown_expression_transition():
    guide = cm.Scene().character("guide", cm.characters.codimate, at=(640, 360))
    raises(lambda: guide.expression("smile", transition="bounce"), "crossfade.*cut")


def test_character_rejects_unknown_theme():
    guide = cm.Scene().character("guide", cm.characters.codimate, at=(640, 360))
    raises(lambda: guide.theme("sepia"), "theme must be 'light' or 'dark'")
