"""Rebuild the tutorial pictures and animation from its actual Codimate scenes.

Run explicitly with: python docs/examples/build_ball_media.py
"""

import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path

import codimate as cm
from codimate import preview

HERE = Path(__file__).resolve().parent
MEDIA = HERE.parent / "media"


@dataclass
class Ball:
    x: float


def picture(name, x, radius, color):
    def view(frame):
        scene = cm.Scene()
        scene.circle("ball", r=radius, at=(frame.state.x, 180)).fill(color)
        return scene

    explanation = cm.explain(trace=cm.trace(lambda ball, emit: None, Ball(x)),
                             view=view)
    explanation.frame_at(0, str(MEDIA / name))


def main():
    MEDIA.mkdir(exist_ok=True)
    explanation = preview.build(HERE / "ball.py")
    explanation.frame_at(0, str(MEDIA / "ball-start.png"))
    explanation.frame_at(3, str(MEDIA / "ball-end.png"))
    picture("ball-orange.png", 540, 30, "orange")
    picture("ball-big.png", 540, 50, "orange")
    picture("ball-middle.png", 320, 50, "orange")
    with tempfile.TemporaryDirectory() as folder:
        video = str(Path(folder) / "ball.mp4")
        explanation.render(video, index=False)
        subprocess.run([
            "ffmpeg", "-y", "-v", "error", "-i", video,
            "-filter_complex",
            "fps=20,split[a][b];[a]palettegen[p];[b][p]paletteuse",
            "-loop", "0", str(MEDIA / "ball.gif"),
        ], check=True)


if __name__ == "__main__":
    main()
