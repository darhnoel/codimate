# Codimate

Codimate is for making small educational animations from code.

If you can explain an idea as steps, Codimate helps you turn those steps into a
video. You describe the data, the steps, what each step should look like, how it
moves, and how long it lasts.

```text
state -> algorithm -> steps -> view -> motion -> timing -> video
```

## Start Here

Run the smallest example first:

```bash
cargo run -p codimate-example-circle-to-square
```

For 1080p at 60 fps:

```bash
cargo run -p codimate-example-circle-to-square -- --1080p60
```

Or preview it:

```bash
cargo run -p codimate-previewer -- circle-to-square
```

The example draws an outlined circle, morphs it into a square, then morphs it
back into a circle. Open `examples/circle-to-square/src/lib.rs` and start by
changing one thing:

```text
circle_path(...)       where the circle starts
rect_path(...)         where the square appears
manim::BLUE            outline color
1.4                    duration of each morph
ShapeStep              steps in the explanation
```

## The Shape

Every example should have a `create()` function that reads like the animation
recipe:

```rust
pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Circle to Square")
        .state(ShapeDemo)
        .view(circle_to_square_view)
        .algorithm(circle_to_square_algorithm)
        .motion(circle_to_square_motion)
        .timing(ShapeTiming::default())
        .build()
}
```

Those pieces mean:

```text
state      input data
algorithm  turns data into steps
steps      the moments worth showing
view       draws one step as a Scene
motion     tweens, paths, easing, pulses
timing     durations
```

Use `path_node` when one shape must transform into another shape. Use plain
`circle()` or `rect()` when the shape type stays the same.

## Copy Next

After `circle-to-square`, look at these examples:

- `examples/demo` — tiny tour of circle, rectangle, and path animation
- `examples/swap` — values moving through a temporary slot
- `examples/merge-sort` — algorithm steps becoming a full explanation
- `examples/matrix-mult` — repeated computation steps
- `examples/neural-net` — signal flow through layers

For the normal project layout, split the pieces like this:

```text
state.rs      data
algorithm.rs  data -> steps
motion.rs     tweens, paths, easing
timing.rs     durations
view.rs       step -> scene
builder.rs    wires everything together
main.rs       calls create() or render()
```

Deeper context:

- [Authoring Model](docs/authoring-model.md)
- [Domain Context](CONTEXT.md)
