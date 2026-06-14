# Codimate

Codimate is for making small educational animations from code.

If you can explain an idea as steps, Codimate helps you turn those steps into a
video. You describe the data, the steps, what each step should look like, how it
moves, and how long it lasts.

```text
state -> algorithm -> steps -> view -> motion -> timing -> video
```

## Start Here

Follow the canonical onboarding path:

- [Daily Workflow](docs/daily-workflow.md) — beginner path to first custom
  animation in less than 30 minutes.

Two-stage beginner flow:

1. **First win:** run `circle-to-square`
2. **Real template:** build from `swap`

```bash
cargo run -p codimate-example-circle-to-square
cargo run -p codimate-example-box-arrow
cargo run -p codimate-example-swap
cargo run -p codimate-previewer -- circle-to-square
```

The `circle-to-square` example is the fast confidence check.
The `box-arrow` example is the primitive-first API tutorial (box + arrow + t1..t2 flow).
The `swap` example is the canonical daily authoring template.

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

Use `primitive_path(...)` when one shape must transform into another shape.
Use plain `circle()` or `rect()` when the shape type stays the same.

Minimal scene authoring pattern:

```rust
use codimate::*;

let s = scene()
    .add(
        primitive_path(rect_path(0.0, 0.0, 1280.0, 720.0))
            .style(Style::new().fill(Color::BLACK)),
    )
    .add(primitive_path(rect_path(540.0, 300.0, 200.0, 100.0)).fill(Color::WHITE));
```

## Copy Next

After the two-stage start, move through examples by intent:

- `examples/box-arrow` — primitive-first API tutorial (draw box, connect arrow, animate flow)
- `examples/swap` — canonical daily template (copy this split first)
- `examples/demo` — compact authoring tour
- `examples/merge-sort` — algorithm-heavy trace
- `examples/matrix-mult` — repeated compute flow
- `examples/neural-net` — layered signal flow

Canonical module split for new explanations:

```text
state.rs      concept data
algorithm.rs  concept logic -> trace
view.rs       state + trace event -> scene
motion.rs     timeless movement/style
timing.rs     durations only
builder.rs    explain(...).state(...).view(...).algorithm(...).motion(...).timing(...)
lib.rs        create() entry
main.rs       run/preview/export entry
```

Deeper context:

- [Daily Workflow](docs/daily-workflow.md)
- [Authoring Model](docs/authoring-model.md)
- [Examples](examples/README.md)
- [Domain Context](CONTEXT.md)
