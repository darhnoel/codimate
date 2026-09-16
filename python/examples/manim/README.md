# Coming from Manim

```bash
.venv/bin/python python/examples/manim/main.py              # all five
.venv/bin/python python/examples/manim/different_rotations.py   # or just one
```

Five of Manim's tutorial scenes, written here. Not to show Codimate winning —
for these it does not — but because they are the shortest way to see what is
actually different, and the difference is not syntax.

| file | Manim scene | what it is really about |
| --- | --- | --- |
| `square_to_circle.py` | `SquareToCircle` | there is no `Create`, `Transform` or `FadeOut` |
| `animate_square_to_circle.py` | `AnimatedSquareToCircle` | `.turn()` is a field the Engine tweens |
| `animate_example.py` | `AnimateExample` | there is nothing to compose — differences happen at once |
| `different_rotations.py` | `DifferentRotations` | **what you put in the payload decides what the motion is** |
| `two_transforms.py` | `TwoTransforms` | a distinction that dissolves, and the one that replaces it |
| `shapes.py` | — | the outlines, walked by distance. No Codimate; checks itself |

This folder breaks the one-`main.py` rule on purpose: it is a set of
translations rather than one explanation, so there is a file per scene and
`main.py` just runs them in order.

## `SquareToCircle`

```python
# manim                                    # codimate
class SquareToCircle(Scene):               @cm.trace()
    def construct(self):                   def square_to_circle(state):
        circle = Circle()                      for step in range(1, STEPS + 1):
        circle.set_fill(PINK, opacity=0.5)         state["drawn"] = step / STEPS
        square = Square()                          cm.emit("draw")
        square.rotate(PI / 4)                  state["shape"] = "circle"
                                               cm.emit("morph")
        self.play(Create(square))              state["shape"] = None
        self.play(Transform(square, circle))   cm.emit("gone")
        self.play(FadeOut(square))
```

**There is no `Transform`.** The square and the circle are the same *name* —
`"shape"` — in two consecutive Scenes, and the morph is what the difference
between them means. Nothing is asked for; it follows.

**There is no `FadeOut` either.** You stop emitting the shape. Leaving *is*
fading, which is also why a shape that pops in and out of a Scene by accident
looks like a ghost — a lesson `rubiks_cube` paid for.

**And no `Create`.** The pen going round the square is continuous motion, and
continuous motion in this library is *sampled* and handed over a step at a
time, exactly as `dharma_wheel` turns and `pendulum` swings. Forty-eight
samples of "how much is drawn" is the whole of it.

## `AnimatedSquareToCircle`

```python
self.play(Create(square))                    state["drawn"] = step / STEPS
self.play(square.animate.rotate(PI / 4))     state["tilt"] = 45.0
self.play(Transform(square, circle))         state["shape"] = "circle"
self.play(square.animate.set_fill(PINK, 0.5))state["filled"] = True
```

Four beats, four states. The rotation is the one that maps almost exactly:
`.turn()` is a field on the shape and the Engine tweens it, so a quarter turn
is one number changing between two Scenes.

It is a beat of its own rather than something applied throughout, because
`.turn()` pivots about the shape's middle — and during the drawing that is the
middle of a part-drawn arc, wandering, not the middle of the square.

## `AnimateExample`

```python
square = Square().set_fill(RED, opacity=1.0)
self.add(square)

self.play(square.animate.set_fill(WHITE))
self.wait(1)
self.play(square.animate.shift(UP).rotate(PI / 3))
```

Two of these need no translating. `self.add(square)` is not an animation, so
the square is simply in the first Scene. `.animate.set_fill(WHITE)` is a colour
in the payload, and the Engine tweens colours like any other number.

The chain is the interesting one. `.shift(UP).rotate(PI / 3)` is Manim
composing two animations so they play together, and there is nothing here to
compose with:

```python
state["lifted"], state["spin"] = True, 60.0
cm.emit("move")
```

Two Scenes differ by a position *and* an angle, so both change over the same
beat. **Everything that differs between two Scenes happens at once** is not a
feature — it is the only thing a difference can mean. Manim needs `AnimationGroup`,
`LaggedStart` and `succession` to say which of those it wants; here "at once"
is free and "one after another" is what a second `cm.emit` is for.

`.wait(1)` is a Scene that differs from the one before it in nothing at all.

## `DifferentRotations`

```python
self.play(
    left_square.animate.rotate(PI), Rotate(right_square, angle=PI), run_time=2
)
```

The most useful of the three, because what it exists to show is what Codimate
is built around.

`.animate` interpolates a shape's *points* from where they start to where they
end. Half way through a half turn every corner is half way to its opposite one,
so the square collapses to a point and opens out again. `Rotate` turns the
shape, so it stays square. Both are one call, and from the call you cannot tell
which you have written.

Here they are visibly different things, and the difference is which field
changes:

```python
scene.polygon("left",  _turned(LEFT, spin))          # corners in the payload
scene.polygon("right", corners(centre=RIGHT)).turn(spin)   # angle in the payload
```

The left square's corners are in the payload, so the reconciler interpolates
them one at a time and reproduces Manim's collapse precisely. The right
square's angle is a field the Engine spins, so it turns. Same half turn, same
two seconds, and the trace is a single line:

```python
state["spin"] = 180.0
cm.emit("turn")
```

That is the whole lesson of the folder in one picture: **what you put in the
payload decides what the motion is.** It is the same choice `bubble_sort` makes
when it names bars after values rather than after slots, and the same one that
made a Rubik's cube come apart mid-turn when its polygons were named after
facelets instead of stickers.

## `TwoTransforms`

```python
self.play(Transform(a, b))              # `a` stays, wearing b's shape
self.play(ReplacementTransform(a, b))   # `b` takes a's place in the scene
```

Two calls that look identical on screen. The difference is bookkeeping: after
`Transform(a, b)` the thing on screen is still `a`, so the next call is
`Transform(a, c)` and the fade is `FadeOut(a)`. After `ReplacementTransform`
it is `b`, so it is `ReplacementTransform(b, c)` and `FadeOut(c)`. Pick the
wrong one and nothing *looks* wrong — you just hold a handle that no longer
points at the picture.

**Neither exists here, because the name is the identity.** There is no registry
of objects to keep in step with the drawing, so there is nothing to fall out of
step. Both of Manim's methods are the same line:

```python
scene.polygon("shape", outline(whatever_it_is_now))
```

The choice Codimate makes you make instead is on the other side of the picture,
and the example shows both halves side by side:

```python
scene.polygon("kept", ...)                    # one name — it morphs
scene.polygon(("fresh", state["beat"]), ...)  # a new name — it cross-fades
```

Keep the name and consecutive Scenes share something, so the Engine has
something to interpolate and the outline morphs. Change it and nothing is
shared: one shape enters, the one before it leaves, and entering and leaving
are fades. Same three shapes, same three beats, two completely different
animations — and the only difference is what you called them.

That is a decision about what the thing *is*, which is the decision this
library asks for everywhere, and which Manim's two methods are not about.

## The one technique worth stealing

A path that grows by **adding points** cannot be animated: two polygons with
different point counts do not interpolate, and the later one stands for the
whole beat ([ADR 0010](../../../docs/adr/0010-polygons-and-the-payload.md)).

A path that keeps **ninety-six points throughout** and spreads them over a
longer and longer run animates perfectly, and at the end the last point lands
back on the first. That is how anything gets drawn on here.

The second half is less obvious and matters as much. Describing a square as

```python
r(a) = HALF / max(abs(cos(a)), abs(sin(a)))
```

and stepping `a` evenly is the natural thing to write and it is **not smooth**:
the radius stretches toward the corners, so equal angles cover more perimeter
there, and the pen visibly hurries through the corners and dawdles along the
flats. `shapes.py` walks the four sides at a constant rate instead, and asserts
that the tip's travel per sample is identical to within a rounding error.

## Where Manim is better, plainly

`Create`, `Transform` and `FadeOut` are one call each. Here they are a loop, a
naming convention and an omission. For *two shapes and three verbs*, a library
built around shapes-and-verbs will always be shorter, and pretending otherwise
would be the wrong lesson to take from this folder.

What Codimate is shaped for is the case these scenes do not have: an actual
program, whose state the trace walks through and whose picture the view is a
pure function of. `bubble_sort` is eight lines of algorithm and the animation
falls out. There is no algorithm here, so you are paying for machinery you are
not using.

The other difference shows up later. Because motion is derived from names
rather than named at the call site, changing what the picture *is* does not
mean rewriting what the picture *does* — which is worth nothing in a
forty-line scene and worth a great deal in a four-minute explanation.
