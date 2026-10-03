"""`examples/maxwell`: the film builds, and every equation in it typesets.

Nothing renders here. Building the film runs the view for every moment, which
catches a clash of names or a bad shape; measuring each formula runs it through
the LaTeX bridge, which caught `\\partial` (the bridge writes a Typst symbol the
installed Typst does not have). Needs `typst` on PATH, like the film itself.
"""

import importlib.util
from pathlib import Path

import support  # noqa: F401  (puts `codimate` on the import path)
import codimate as cm
from codimate import preview

FILM = Path(__file__).resolve().parents[1] / "examples" / "maxwell" / "main.py"
_built = []


def film():
    if not _built:
        _built.append(preview.build(FILM))
    return _built[0]


def test_every_equation_in_the_film_typesets():
    formulas = {}
    for scene in film().scenes:
        for shape in scene._shapes.values():
            if shape.kind == "formula":
                formulas[shape.text] = shape.size
    assert len(formulas) >= 10, "the film writes its equations as formulas"
    for latex, size in formulas.items():
        width, height = cm.measure_math(latex, size)
        assert width > 0 and height > 0, latex


def test_the_film_lasts_as_long_as_its_scenes_and_ends_still():
    exp = film()
    assert 60 < exp.duration < 140, "a film of eight scenes, not a clip"
    last = exp.trace.events[-1].state
    assert last["k"] == 8 and last["said"] > 0, "the last line is read to the end"


def test_each_scene_names_a_caption_that_exists():
    spec = importlib.util.spec_from_file_location("lines", FILM.parent / "lines.py")
    lines = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(lines)
    keys = {event.state["line"] for event in film().trace.events} - {""}
    assert keys <= set(lines.SAY) and len(keys) == 9


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
