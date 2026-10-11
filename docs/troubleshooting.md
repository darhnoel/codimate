# Troubleshooting

Start with the symptom below. Most drawing problems can be checked with one
image using `explanation.frame_at(1.5, "check.png")` rather than a full export.

## Installation and tools

### Python cannot import Codimate

Install into the same interpreter you use to run your script:

```bash
python -m pip install codimate
python -c "import codimate; print(codimate.__file__)"
```

If you use a virtual environment, activate it first or use its Python path
for both commands. A source checkout needs a built extension; ordinary users
should install the package rather than copy the `python/codimate` directory.

### FFmpeg is missing or fails to start

Codimate uses FFmpeg from your PATH or the copy bundled with `imageio-ffmpeg`.
Check an explicit executable if you installed one:

```bash
ffmpeg -version
```

You can set `CODIMATE_FFMPEG` to the executable path. Check that the file exists
and can execute. On a machine without system FFmpeg, confirm the
`imageio-ffmpeg` dependency is installed in your Python environment.

### A formula reports that Typst is missing

`scene.formula` requires the `typst` executable on PATH. Text and ordinary
shapes do not. Follow [the formula recipe](recipes.md#add-a-formula).

## The picture is wrong

| Symptom | Check | Fix |
| --- | --- | --- |
| A shape fades instead of moving | Does its name change between events? | Keep its identity stable. |
| Sorting bars change height in place | Are names based on slot indices? | Name moving bars with stable item IDs. |
| Duplicate shape-name error | Are two shapes using the same complete name? | Give them distinct names or put them in differently named groups. |
| `give one of x=` or a similar placement error | Is an axis missing at scene level? | Supply `at=(x, y)` or both axes through `cm.at`. |
| A label or shape is hidden | Do overlapping shapes share a layer? | Put the foreground shape on a higher layer. |
| Accessing event data fails at startup | Is `frame.event` assumed to exist? | Check for `None` in the opening picture. |
| Continuous motion speeds up and slows down repeatedly | Is each sample using default easing? | Use `cm.Rule("*", position="linear")`. |
| A curve or polygon jumps between states | Does its point count change? | Keep the point count fixed across events. |
| Unknown-color error | Is the name supported? | Use a documented color or `#rrggbb`. |

Read [How names work](concepts.md#names-decide-what-moves) and
[coordinate placement](drawing.md#place-a-shape) for worked explanations.

## The result is not where you expected

Output paths are relative to the terminal's current working folder.
`render("results/video.mp4")` creates `results` there, not necessarily beside
the Python script. Use an absolute path if you want a fixed destination.

## The video takes too long to make

First inspect the [timeline and a single image](recipes.md#inspect-a-moment-before-exporting-the-whole-video).
Use shorter durations, a lower frame rate, or a smaller export scale while
iterating. Duration and frame rate affect the number of frames to render.

When developing the engine, use `maturin develop --release`; a debug engine
build is much slower. You do not need Maturin when using an installed wheel.

## The caption or narration is wrong

Codimate displays and plays the text and audio you provide. It does not translate,
validate scientific claims, or automatically align individual words with speech.
Check the model, the caption text, and the event timing separately.

## Still stuck?

Reduce your program to one object and two emitted moments. Include the smallest
reproducing script, your Python and Codimate versions, the error message, and
what you expected when reporting a problem on
[GitHub](https://github.com/darhnoel/codimate/issues).
