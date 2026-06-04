# Codimate Authoring Model

Codimate is fastest when you do not start by drawing frames.

Start with the concept:

```text
State -> Algorithm -> Trace -> View -> Scene -> Timing -> Video
```

The short version:

```rust
explain("Merge Sort")
    .state(MergeSort::new([38, 27, 43, 3, 9, 82, 10, 15]))
    .view(merge_sort_view)
    .algorithm(merge_sort_algorithm)
    .motion(merge_sort_motion)
    .timing(MergeSortTiming::default())
    .render("results/merge-sort.mp4");
```

This chain is the mental model. The examples currently implement it locally so
we can prove the shape before promoting it into a shared public API.

## Why This Works

The One Law is still:

```text
f(t) -> Scene
```

The authoring model explains where `f` comes from.

You describe the concept's behavior and appearance:

```text
algorithm: State -> Trace
view:      State + Trace Event -> Scene
timing:    Trace Event -> duration
```

Codimate composes those pieces into a pure animation. Change the data or the
algorithm, and the video regenerates from the concept instead of being manually
re-edited.

## The File Split

Use this split for examples and new explanations:

```text
src/
  state.rs      concept data
  algorithm.rs  concept logic -> trace
  view.rs       state/trace -> scene
  motion.rs     timeless movement and style choices
  timing.rs     durations and pacing
  builder.rs    explain(...).state(...).view(...).algorithm(...).motion(...).timing(...)
  lib.rs        small public surface and create()
  main.rs       the clean authoring chain
```

## Responsibilities

`state.rs`

Define the concept data. This should read like the subject, not like the
renderer.

```rust
pub struct MergeSort {
    values: [i32; N],
}
```

`algorithm.rs`

Turn the State into a Trace. The Trace is the explanation script. For merge
sort, events are comparisons and writes. For matrix multiplication, events are
row-column dot products. For neural nets, events are signal groups.

```rust
pub fn merge_sort_algorithm(state: MergeSort) -> MergeTrace {
    // Run the real concept logic and record meaningful events.
}
```

`view.rs`

Project one trace moment into a Scene. The View chooses layout, labels, colors,
highlighting, and which Nodes appear.

```rust
fn step_scene(step: &MergeStep, motion: MergeSortMotion) -> Scene {
    // Show this concept event as a visual scene.
}
```

`motion.rs`

Name reusable movement choices. Motion is timeless: easing curves, movement
paths, pulse/reveal choices, and style transitions. It must not contain
durations.

```rust
pub struct MergeSortMotion;

impl MergeSortMotion {
    pub(crate) fn move_value<T>(
        self,
        from: impl IntoAnimated<T>,
        to: impl IntoAnimated<T>,
    ) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(ease_in_out)
    }
}
```

`timing.rs`

Keep pacing here. If a duration appears in `state.rs`, `algorithm.rs`, `view.rs`,
or `motion.rs`, it is probably in the wrong place.

```rust
pub struct MergeSortTiming {
    pub overview: f32,
    pub step: f32,
    pub transition: f32,
    pub final_hold: f32,
}
```

## Build A New Explanation

1. Name the concept.
2. Define State.
3. Write the Algorithm as real concept logic.
4. Record a Trace of meaningful events.
5. Write a View for each kind of Trace Event.
6. Put movement choices in Motion.
7. Put durations in Timing.
8. Keep `main.rs` as the clean `explain(...)` chain.

## Good Trace Events

Good events come from the concept:

- `Compare { left, right }`
- `Swap { left, right }`
- `ChoosePivot { index }`
- `ComputeCell { row, col }`
- `FireToHidden { hidden }`
- `FireToOutput { output }`

Weak events come from video editing:

- `ShowBoxAtFrame12`
- `MoveThingForTwoSeconds`
- `MakeItBlueNow`
- `Keyframe3`

If the event would still make sense in a written explanation of the concept, it
probably belongs in the Trace.

## Examples To Copy

- `examples/swap`: smallest complete model.
- `examples/matrix-mult`: clean State -> Trace -> View for a math concept.
- `examples/merge-sort`: algorithm trace with buffers and transitions.
- `examples/quick-sort`: partition events and swap motion.
- `examples/neural-net`: grouped signal firing from left to right.

## Guardrails

- The final animation must still obey `f(t) -> Scene`.
- Algorithm produces the Trace; View does not invent concept logic.
- Motion has no duration.
- Timing owns duration.
- Scene Nodes stay pure data.
- Renderer/export code stays outside the authoring model.
