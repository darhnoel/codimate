"""Text presets are reusable, named groups of ordinary Text shapes."""

import support  # noqa: F401
import codimate as cm
import unicodedata
from codimate.scene import covered
from codimate.text import TextPreset, TextPresetStyle
from codimate.text.preset import TextPreset as PresetModuleTextPreset


def test_text_package_exports_the_presets_and_keeps_the_root_alias():
    assert TextPreset is PresetModuleTextPreset is cm.TextPreset
    assert TextPresetStyle is cm.TextPresetStyle


def test_named_text_helpers_choose_the_preset_without_a_selector_string():
    for name in (
        "words_up",
        "words_down",
        "letters_up",
        "letters_down",
        "words_left",
        "words_right",
        "words_up_blur",
    ):
        preset = getattr(cm.text, name)("title", "Hello")
        assert isinstance(preset, cm.TextPreset)
        assert preset.preset == name


def _by_name(scene):
    return {shape["item"]: shape for shape in scene._payload()}


def test_words_up_keeps_identity_and_moves_from_below():
    preset = cm.TextPreset(
        "title",
        "Hello world",
        "words_up",
        style=cm.TextPresetStyle(size=40),
        at=(300, 200),
    )
    hidden, shown = cm.Scene(), cm.Scene()
    preset.draw(hidden, state="hidden")
    preset.draw(shown, state="shown")
    before, after = _by_name(hidden), _by_name(shown)

    assert set(before) == set(after) == {"title/unit/0", "title/unit/1"}
    assert before["title/unit/0"]["y"] == after["title/unit/0"]["y"] + 48
    assert before["title/unit/0"]["opacity"] == 0
    assert after["title/unit/0"]["opacity"] == 1
    assert [rule.pattern for rule in preset.rules()] == ["title/unit/*"]


def test_words_are_centered_by_line_and_can_draw_inside_a_group():
    preset = cm.TextPreset("title", "Hello\nworld", "words_down", at=(0, 0))
    scene = cm.Scene()
    preset.draw(scene.group("panel", at=(400, 300)), state="shown")
    shapes = _by_name(scene)

    assert shapes["panel/title/unit/0"]["x"] == shapes["panel/title/unit/1"]["x"]
    assert shapes["panel/title/unit/0"]["y"] < shapes["panel/title/unit/1"]["y"]


def test_letters_keep_combining_marks_together():
    preset = cm.TextPreset("title", "ខ្មែរ Á", "letters_up", at=(0, 0))
    scene = cm.Scene()
    preset.draw(scene, state="shown")
    text = [shape["text"] for shape in scene._payload()]

    assert any(value.startswith("ខ") for value in text)
    assert not any(unicodedata.category(value[0]).startswith("M") for value in text)
    assert "Á" in text
    assert " " not in text


def test_words_up_blur_has_soft_and_sharp_states():
    preset = cm.TextPreset("title", "Soft words", "words_up_blur", at=(0, 0))
    hidden, blurred, shown = cm.Scene(), cm.Scene(), cm.Scene()
    preset.draw(hidden, state="hidden")
    preset.draw(blurred, state="blurred")
    preset.draw(shown, state="shown")
    before, middle, after = map(_by_name, (hidden, blurred, shown))

    assert set(before) == set(middle) == set(after)
    assert middle["title/blur/0/0"]["opacity"] == 0.12
    assert middle["title/unit/0"]["opacity"] == 0
    assert after["title/unit/0"]["opacity"] == 1
    assert after["title/blur/0/0"]["opacity"] == 0
    assert len(preset.rules()) == 9
    scene = cm.Scene()
    preset.draw(scene, state="blurred")
    assert covered(scene) == [], "blur copies are visual texture, not labels"


def test_preset_and_state_names_are_checked():
    for value in ("words-up", "letters_up_blur"):
        try:
            cm.TextPreset("title", "Hello", value)
        except ValueError as error:
            assert value in str(error)
        else:
            raise AssertionError(f"unknown preset {value!r} should be refused")

    preset = cm.TextPreset("title", "Hello", "words_up")
    try:
        preset.draw(cm.Scene(), state="blurred")
    except ValueError as error:
        assert "hidden" in str(error) and "shown" in str(error)
    else:
        raise AssertionError("blurred is only a state for words_up_blur")


def test_all_seven_presets_have_the_expected_entrance_direction():
    directions = {
        "words_up": (0, 48),
        "words_down": (0, -48),
        "letters_up": (0, 48),
        "letters_down": (0, -48),
        "words_left": (48, 0),
        "words_right": (-48, 0),
        "words_up_blur": (0, 48),
    }
    for name, (dx, dy) in directions.items():
        preset = cm.TextPreset(name, "Hi all", name, at=(300, 200))
        hidden, shown = cm.Scene(), cm.Scene()
        preset.draw(hidden, state="hidden")
        preset.draw(shown, state="shown")
        before, after = _by_name(hidden), _by_name(shown)
        key = f"{name}/unit/0"
        assert before[key]["x"] == after[key]["x"] + dx
        assert before[key]["y"] == after[key]["y"] + dy


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
