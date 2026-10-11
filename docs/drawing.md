# Drawing and layout

Use this page when you know what you want to show and need to put it on screen.
The snippets are drawing fragments: use them inside your `view(frame)` function.

## Choose a shape

| What you want | Method | Example |
| --- | --- | --- |
| A bar, box, or background | `scene.rect` | `scene.rect("bar", w=100, h=200, at=(640, 360))` |
| A dot, disc, or ring | `scene.circle` | `scene.circle("dot", r=30, at=(640, 360))` |
| A label | `scene.text` | `scene.text("title", "Hello", size=36, at=(640, 80))` |
| A line between points | `scene.line` | `scene.line("line", start=(200, 360), end=(900, 360), w=4)` |
| A force or direction | `scene.arrow` | `scene.arrow("force", start=(640, 400), end=(640, 200))` |
| A triangle or other outline | `scene.polygon` | Supply a list of `(x, y)` corners. |
| A smooth plotted line | `scene.curve` | Supply sample points on the curve. |
| An arc or sector | `scene.arc` | Supply a radius and a pair of angles. |
| A formula | `scene.formula` | Supply a LaTeX string; requires Typst. |
| Vector artwork | `scene.svg` | Supply an SVG file. |
| A photograph or bitmap | `scene.image` | Supply a PNG or JPEG file. |

All drawing methods are available on groups as well as scenes.
For argument details, see [Reference: Drawing](reference.md#drawing).

## Place a shape

The default canvas is **1280 × 720**. Coordinates start at the top-left:

```text
(0, 0) ───────────────────────→ x
  │
  │           (640, 360)
  │              center
  ↓
  y                         (1280, 720)
```

`at=(x, y)` places the shape by its center:

```python
scene.circle("dot", r=30, at=(640, 360)).fill("orange")
```

Use `cm.at` when an edge should stay fixed. This bar grows upward from y=550:

```python
scene.rect("bar", w=100, h=200,
           at=cm.at(x=640, bottom=550)).fill("blue")
```

A label whose top is at y=580:

```python
scene.text("label", "Count", size=28,
           at=cm.at(x=640, top=580)).fill("white")
```

At scene level, specify both axes. Inside a group, omitted coordinates use
the group's own point. Give only one of `y`, `top`, or `bottom` per placement.

## Style a shape

The drawing method returns a handle. Chain styling methods onto it:

```python
scene.rect("card", w=240, h=120, at=(640, 360)).fill(
    "#243046", edge="white", edge_w=2
).round(12)
```

| Method | Purpose |
| --- | --- |
| `.fill("blue")` | Set the fill or line color. |
| `.fill("none", edge="blue", edge_w=4)` | Draw an outline without a fill. |
| `.round(12)` | Round rectangle corners. |
| `.on(layer=3, opacity=0.5)` | Set drawing order and transparency. |
| `.turn(30)` | Rotate supported geometry by 30 degrees. |
| `.grow(1.5)` | Scale a shape. |
| `.write(reveal=0.5, pen=2)` | Reveal supported formulas or imported SVG paths. |

Use named colors such as `blue`, `orange`, `white`, `grey`, or hex colors
such as `#087FC4`. Do not assume every CSS color name is supported.
Text glyphs stay upright when a text handle is turned.

## Put related shapes in a group

A group is useful for a bar with a label, a planet with a name, or a car with wheels.
Place the group once, then draw its children relative to that point:

```python
bar = scene.group("sales", at=(640, 550), w=160)
bar.rect("body", h=210, at=cm.at(bottom=0)).fill("blue")
bar.text("label", "Sales", size=28, at=cm.at(top=20)).fill("white")
```

The complete shape names are `sales/body` and `sales/label`.
Keep the group name stable so the object retains its identity between moments.

## Use local coordinates inside a group

In the example above, the group's point is `(640, 550)` on the canvas.
Inside the group:

| Local placement | Canvas meaning |
| --- | --- |
| `at=(0, 0)` | The group's point, `(640, 550)`. |
| `bottom=0` | Put the bar's bottom at y=550. |
| `top=20` | Put the label's top at y=570. |
| `at=(-50, -100)` | Move 50 left and 100 up from the group point. |

Positive y goes downward in both coordinate systems. A negative local y moves
upward; it does not mean a negative canvas coordinate.

## Arrange a row or column

`cm.row` supplies places for items from left to right. A row's slots anchor at
bottom-center, which is useful for bars sharing a baseline:

```python
for slot, category in cm.row(("A", "B", "C"), gap=40, size=120,
                             at=cm.at(y=550)):
    bar = scene.group(category, slot)
    bar.rect("body", h=200, at=cm.at(bottom=0)).fill("blue")
    bar.text("label", category, size=28, at=cm.at(top=20)).fill("white")
```

Here `size=120` sets each slot's width. `gap=40` is the space between slots.
A slot is layout data, not a visible box.

`cm.column` stacks slots vertically and anchors them at their center:

```python
for slot, label in cm.column(("Input", "Compute", "Output"), gap=24,
                            size=90, at=cm.at(x=640)):
    card = scene.group(label, slot)
    card.rect("body", w=240, h=70).fill("#243046").round(8)
    card.text("label", label, size=28).fill("white")
```

## Control which shape covers another

Higher layers draw on top. Set layers explicitly when overlapping matters:

```python
scene.rect("background", w=1280, h=720, at=(640, 360)).fill(
    "#0B1420"
).on(layer=-10)
scene.circle("planet", r=80, at=(640, 360)).fill("blue").on(layer=1)
scene.text("name", "Earth", size=28, at=(640, 360)).fill("white").on(layer=2)
```

Within a layer, names determine drawing order. Creation order alone does not
ensure that a label appears above a shape.

## Draw a curve or import artwork

A curve passes through your supplied samples:

```python
scene.curve("wave", [(200, 360), (400, 250), (600, 360), (800, 470)],
            w=4).fill("blue")
```

Keep the sample count the same across events if you want the curves to animate
smoothly between states. The same point-count rule applies to polygons.

```python
scene.image("photo", "photo.png", size=(320, 240), at=(640, 360))
scene.svg("logo", "logo.svg", size=180, at=(640, 360))
```

These snippets require your own files. SVGs import as geometry; images remain
bitmaps. For supported SVG features, see [the reference](reference.md#drawing).

## Canvas size and export resolution

Set the canvas before constructing your explanation:

```python
cm.canvas(1280, 720)
```

Choose output resolution at export:

```python
explanation.render("results/video.mp4", fps=60, scale=1.5)
```

A 1280 × 720 canvas at scale 1.5 produces 1920 × 1080 pixels. Your view's
coordinates stay the same. To make a square composition, change the canvas
itself, for example `cm.canvas(1024, 1024)`.

## Next steps

[Recipes](recipes.md) cover highlighting, formulas, audio, camera framing, and
inspection. [Reference](reference.md) lists the complete API, including the
science kit for projected planets and orbits. Codimate is a 2D renderer;
projected scientific diagrams do not provide general 3D solid rendering.
