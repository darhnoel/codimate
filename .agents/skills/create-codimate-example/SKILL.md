---
name: create-codimate-example
description: Create a runnable Codimate example from a user's algorithm prompt. Use when the user asks to visualize an algorithm, turn an algorithm description into a Codimate example, or create an example under examples/<name>. For unknown or custom algorithms, interview the user first until the algorithm and visualization are clear.
---

# Create Codimate Example

Turn a user prompt into a runnable Codimate example in the current Codimate
workspace.

## If The Algorithm Is Unclear

For unknown/custom algorithms, run the interview yourself. Do not depend on a
separate `grill-me` skill being installed.

Ask one question at a time. For each question, give a recommended answer when a
reasonable default exists. Continue until these are clear:

```text
input data
initial state
ordered steps
what changes each step
what should be highlighted
final state or answer
preferred visual layout
```

Use the interview for custom/domain-specific prompts such as:

```text
visualize my weighted mood scheduler
visualize our custom decoder
visualize the fraud scoring handoff
visualize my consensus algorithm
```

Do not interview by default for standard algorithms unless the user asks for
unusual behavior:

```text
binary search
merge sort
quick sort
BFS / DFS
Dijkstra / A*
LRU cache
```

## Output Shape

Create a split example immediately:

```text
examples/<example-name>/
  Cargo.toml
  src/
    lib.rs
    main.rs
    state.rs
    algorithm.rs
    motion.rs
    timing.rs
    view.rs
    builder.rs
    style.rs
```

Use kebab-case for the directory and package suffix:

```text
examples/binary-search
codimate-example-binary-search
```

## Required Mental Model

The example must expose this chain from `lib.rs`:

```rust
pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Binary Search")
        .state(BinarySearch::new(...))
        .view(binary_search_view)
        .algorithm(binary_search_algorithm)
        .motion(binary_search_motion)
        .timing(BinarySearchTiming::default())
        .build()
}
```

Keep responsibilities separate:

```text
state.rs      input data and concept state
algorithm.rs  state -> ordered steps
motion.rs     tweens, paths, easing, pulses; no durations
timing.rs     durations only
view.rs       one step -> scene
builder.rs    explain(...).state(...).view(...).algorithm(...).motion(...).timing(...)
style.rs      Manim palette aliases and local semantic colors
main.rs       standard export plus --1080p60
```

## Implementation Rules

- Use `codimate_core::manim` colors through `style.rs`.
- Prefer small teaching inputs over exhaustive cases.
- Record concept steps, not video-editing commands.
- Use `path_node` for shape morphs; use direct nodes such as `circle()` and
  `rect()` when the shape type stays the same.
- Include `--1080p60` support in `main.rs`.
- Wire the example into:
  - workspace `Cargo.toml`
  - `crates/codimate-previewer/Cargo.toml`
  - `crates/codimate-previewer/src/bin/preview.rs`
  - `examples/README.md`
- Do not commit. Do not run `git add` or `git commit`.

## Validation

Run:

```bash
cargo fmt
cargo check -p codimate-example-<example-name>
cargo check -p codimate-previewer
```

Report the files changed and the exact commands to run:

```bash
cargo run -p codimate-example-<example-name>
cargo run -p codimate-example-<example-name> -- --1080p60
cargo run -p codimate-previewer -- <example-name>
```
