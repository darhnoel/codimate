# Codimate

**Turn your Python code into an animation.**

Tell Codimate what happens. Draw what it looks like. Codimate makes the movement.
You bring the idea; Codimate helps you show it.

![A blue ball moving from left to right](https://raw.githubusercontent.com/darhnoel/codimate/main/docs/media/ball.gif)

## Install

You need Python 3.9 or newer and basic Python knowledge.

```bash
python -m pip install codimate
```

Work in any folder. You do not need to install Rust or clone this repository.
FFmpeg is provided through a Python dependency if it is not already installed.
Typst is only needed when you add mathematical formulas.

## Make your first animation

[Start the tutorial →](https://github.com/darhnoel/codimate/blob/main/docs/tutorial.md)

You will copy a small program, make the ball move, and change it one step at a time.

[Read the documentation online](https://darhnoel.github.io/codimate/).

<details markdown="1">
<summary>For contributors</summary>

To work on Codimate itself:

```bash
git clone https://github.com/darhnoel/codimate
cd codimate
python3 -m venv .venv
.venv/bin/python -m pip install maturin
.venv/bin/maturin develop --release
.venv/bin/python python/tests/run.py
```

Build the documentation locally with `python docs/build_site.py` after installing
`markdown`, `pygments`, and `pdoc`. Open `site/index.html` to read it.
The [architecture decisions](https://github.com/darhnoel/codimate/tree/main/docs/adr) explain the engine's design.

[MIT license](https://github.com/darhnoel/codimate/blob/main/LICENSE). Bundled fonts keep [their own licenses](https://github.com/darhnoel/codimate/blob/main/crates/codimate-fonts/fonts/LICENSES/README.md).

</details>
