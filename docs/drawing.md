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
| A poseable character | `scene.character` | Supply a rigged SVG or use the Codimate mascot. |
| A photograph or bitmap | `scene.image` | Supply a PNG or JPEG file. |
| A highlighter stroke | `scene.marker` | `scene.marker("note", w=260, h=70, at=(640, 300))` |
| Highlighted words | `scene.highlight` | `band, words = scene.highlight("key", "pure in t", size=32)` |

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

## Choose a font

Text uses the built-in DejaVu Sans Mono until you name another. Set one font
for the whole film, before building scenes, and override it on any text:

```python
cm.font("fonts/Inter-Regular.ttf")       # every text: a .ttf or .otf file

scene.text("title", "Bubble sort", size=48, font="fonts/Inter-Bold.ttf")
scene.text("code", "x = 1", size=24, font="Menlo")   # an installed font, by name
```

A name with a `/` or a font file's extension is a file; anything else is an
installed font, found by family or face name (`"Georgia Italic"`). Prefer a
file kept in your project for a film you will share: an installed font may not
be on the next machine. Bold and italic are their own file or name.
Characters the font lacks, such as Khmer or Chinese, fall back to the built-in
fonts. `cm.measure` and `scene.highlight` take the same `font=`, so boxes and
markers fit the words. A font that will not load is a `ValueError` that names
it.
Text glyphs stay upright when a text handle is turned.

## Pose a character

Use `scene.svg` for a picture that moves as one piece. Use `scene.character` for
an SVG with joints you want to pose between scenes:

```python
scene.rect("paper", w=1280, h=720, at=(640, 360)).fill("white").on(layer=-1)
guide = scene.character("guide", cm.characters.codimate,
                         size=320, at=(640, 360))
guide.part("right-arm").turn(-25)
guide.expression("thinking")
guide.theme("light")  # use "dark" when the scene background is dark
```

The bundled mascot is `cm.characters.codimate`. For your own character, pass an
SVG file path instead. Its root must have one
`<g id="..." data-codimate-rig="1">`. Add
`data-pivot="x y"` to a joint group that also has an `id`; the pivot uses the
SVG root's `viewBox` coordinates. Nest joint groups so a parent's turn carries
its children. Put alternate faces in groups marked
`data-expression="name"`; optionally choose the starting face with
`data-default-expression="name"` on the rig root. Convert SVG text to paths.
Use `guide.theme("dark")` for a dark scene and `guide.theme("light")` for a
light scene. A joint group can set `data-dark-fill="#rrggbb"` to change only
its paint in dark scenes; the ordinary SVG fill is used in light scenes.

Set the pose again in each Scene. Joint angles are degrees from the neutral
pose, and Codimate moves between them. Keep the character's shape name the same
in each Scene so Codimate can match it.

Expressions crossfade by default. For speech or other fast shape changes, use
`guide.expression("mouth-open", transition="cut")` to switch instantly at the
midpoint of the Scene transition.

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
