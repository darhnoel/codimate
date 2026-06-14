# Codimate Daily Workflow

This is the canonical onboarding path for a beginner Explanation Author.

Goal: go from clone to first custom animation render/preview in **less than 30
minutes**.

## Stage 1 — First win (run before editing)

Run the smallest example exactly once:

```bash
cargo run -p codimate-example-circle-to-square
```

Optional:

```bash
cargo run -p codimate-example-circle-to-square -- --1080p60
cargo run -p codimate-previewer -- circle-to-square
```

What this stage proves:

- your toolchain is working,
- export works,
- preview works,
- you can make a visible change quickly.

## Stage 2 — Primitive-first API tutorial (`box-arrow`)

Learn the minimum visual vocabulary first:

```bash
cargo run -p codimate-example-box-arrow
```

This example teaches three primitives without extra architecture noise:

- draw a box,
- connect box A -> box B with an arrow,
- animate arrow flow only during a time window (`t1..t2`).

## Stage 3 — Real daily template (`swap`)

For new explanations, copy the structure of `examples/swap`.

Canonical split:

```text
state.rs      concept data
algorithm.rs  state -> trace events
view.rs       state + trace event -> scene
motion.rs     timeless movement/style choices
timing.rs     durations and pacing only
builder.rs    explain(...).state(...).view(...).algorithm(...).motion(...).timing(...)
lib.rs        create() public entry
main.rs       render/preview entry
```

Build the canonical template:

```bash
cargo run -p codimate-example-swap
```

## Stage 4 — Build your first custom explanation

1. Copy `examples/swap` into a new example crate.
2. Rename State, Trace Event, and view labels to your concept.
3. Keep `main.rs` as the clean authoring chain.
4. Keep durations only in `timing.rs`.
5. Run after each small change.

## Stage 5 — Iterate in a tight loop

Use this loop for daily work:

1. Change one concept detail (`state.rs` or `algorithm.rs`).
2. Update one visual decision (`view.rs` or `motion.rs`).
3. Run the example.
4. Preview if needed.
5. Repeat.

## Troubleshooting quick checks

- If movement feels wrong, check `motion.rs` first (timeless behavior).
- If pacing feels wrong, check `timing.rs` (durations only).
- If visuals are correct but concept is wrong, check `algorithm.rs`.
- If labels/positions are messy, check `view.rs` and Slot usage.

## See also

- [Codimate README](../README.md)
- [Authoring Model](./authoring-model.md)
- [Examples catalog](../examples/README.md)
- [Domain Context](../CONTEXT.md)
