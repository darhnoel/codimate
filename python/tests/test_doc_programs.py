"""Check guide programs without exporting video or images."""

from importlib import import_module

import support  # noqa: F401  (puts the local package on the import path)
import codimate as cm
import test_docs


def test_guide_fragments_compile_without_exporting_media():
    test_docs.test_every_fragment_in_the_guide_is_valid_python()


def test_complete_guide_programs_build_their_scenes_without_exporting_media():
    module = import_module("codimate.explain")
    saved_capture = module._capture
    canvas = (cm.width(), cm.height())
    checked = 0
    try:
        for page in test_docs.PAGES:
            for number, code in enumerate(test_docs._blocks(page), 1):
                if test_docs._kind(code) != "program":
                    continue
                captured = []
                cm.canvas(1280, 720)
                module._capture = captured.append
                try:
                    exec(compile(code, f"{page.name}:{number}", "exec"),
                         {"__name__": "__main__"})
                except module._Captured:
                    pass
                assert len(captured) == 1, (page.name, number)
                explanation = captured[0]
                assert explanation.scenes, (page.name, number)
                explanation._payloads()
                checked += 1
    finally:
        module._capture = saved_capture
        cm.canvas(*canvas)
    assert checked >= 2, f"only {checked} complete guide programs checked"


if __name__ == "__main__":
    raise SystemExit(support.run(globals()))
