"""Finding ffmpeg, and mixing a film's sounds for it."""

from __future__ import annotations



def _mixing(sounds, first: int):
    """ffmpeg inputs and a filter graph that lays each clip at its start and
    sums them. `first` is the input number of the first clip. `normalize=0`
    keeps every clip at its recorded level — amix otherwise divides by the
    number of inputs, and the voice fades as the film gets longer."""
    inputs = [arg for _, clip in sounds for arg in ("-i", clip)]
    ids = range(first, first + len(sounds))
    delays = "".join(f"[{i}:a]adelay={round(start * 1000)}:all=1[d{i}];"
                     for i, (start, _) in zip(ids, sounds))
    summed = "".join(f"[d{i}]" for i in ids)
    return inputs, f"{delays}{summed}amix=inputs={len(sounds)}:normalize=0[a]"


def _ffmpeg() -> str:
    """The ffmpeg the render uses, for the jobs done around it."""
    import os
    import shutil

    _find_encoder()
    found = os.environ.get("CODIMATE_FFMPEG") or shutil.which("ffmpeg")
    if not found:
        raise RuntimeError("this needs ffmpeg — install it, or set CODIMATE_FFMPEG")
    return found


def _find_encoder() -> None:
    """Make sure the engine can find an ffmpeg to run.

    Rendering needs ffmpeg, which pip cannot install. Rather than let
    `pip install codimate` succeed and then fail on someone's first render —
    the worst moment to learn about a prerequisite — fall back to the static
    build that ships with `imageio-ffmpeg`.

    A system ffmpeg is preferred: it is usually newer, hardware-accelerated,
    and the one the author already expects to be used. The bundled copy is a
    safety net, not the default.

    Silent when nothing is found; the engine raises its own error naming both
    the install and the override.
    """
    import os
    import shutil

    if os.environ.get("CODIMATE_FFMPEG") or shutil.which("ffmpeg"):
        return

    try:
        import imageio_ffmpeg
    except ImportError:
        return

    try:
        os.environ["CODIMATE_FFMPEG"] = imageio_ffmpeg.get_ffmpeg_exe()
    except Exception:  # noqa: BLE001 — a broken fallback must not mask the real error
        pass
