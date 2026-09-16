"""Every scene in this folder, in the order Manim's tutorial introduces them.

    python python/examples/manim/main.py

Each scene is its own runnable file — this only exists because the rest of the
repository is one explanation per folder with one `main.py`, and a set of
translations is not that. Run a single one directly when you are reading it:

    python python/examples/manim/different_rotations.py
"""

import runpy
from pathlib import Path

SCENES = ("square_to_circle", "animate_square_to_circle", "animate_example",
          "different_rotations", "two_transforms")

HERE = Path(__file__).resolve().parent

for scene in SCENES:
    runpy.run_path(str(HERE / f"{scene}.py"), run_name="__main__")
