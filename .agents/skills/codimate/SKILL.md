---
name: codimate
description: Create runnable Codimate examples from user prompts with minimal token usage. Use when the user asks to visualize a concept/algorithm, create an example under examples/<name>, or learn Codimate primitives (box, arrow, timed flow) quickly.
---

# Codimate Skill

Turn a user prompt into a runnable Codimate example in this workspace.

## Hard Rule: Signature-First Discovery (Token Budget Mode)

Do **not** digest large files by default. Find API signatures first, then read
only tiny targeted ranges.

1. Use `grep` to find symbol signatures (`fn`, `struct`, `trait`, `impl`, `use`).
2. Prefer reading **20–80 lines around the matched signature**.
3. Stop reading once the needed signature and call pattern are clear.
4. Reuse known canonical examples (`box-arrow`, `swap`, `circle-to-square`) rather
   than scanning many files.
5. Do not read entire files unless strictly necessary for correctness.

Priority lookup order:

- `examples/box-arrow` (primitive-first)
- `examples/swap` (canonical split template)
- `examples/circle-to-square` (smallest run path)

## If The Algorithm Is Unclear

For unknown/custom algorithms, run the interview yourself (no dependency on
external skills). Ask one question at a time and include a recommended answer
when a default exists.

Resolve these before coding:

```text
input data
initial state
ordered steps
what changes each step
what to highlight
final state/answer
preferred visual layout
```

Do not interview by default for standard algorithms unless the user requests
unusual behavior.

## Output Shape

Create a split example:

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

Use kebab-case naming:

```text
examples/binary-search
codimate-example-binary-search
```

## Required Mental Model

`lib.rs` must expose:

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

Responsibilities:

```text
state.rs      input data and concept state
algorithm.rs  state -> ordered steps
motion.rs     timeless movement/style; no durations
timing.rs     durations only
view.rs       one step -> scene
builder.rs    explain(...).state(...).view(...).algorithm(...).motion(...).timing(...)
style.rs      local semantic colors
main.rs       standard export plus --1080p60
```

## Implementation Rules

- Use **facade crate** imports for beginner surface: `use codimate::*;`
- In `Cargo.toml`, prefer `codimate = { path = "../../crates/codimate" }`
  and only add extra crates when required by specialized features.
- Prefer small teaching inputs over exhaustive cases.
- Record concept steps, not video-editing commands.
- For shape morphs, use path primitives; for same-shape updates, use direct nodes.
- Include `--1080p60` support in `main.rs`.
- Wire new examples into:
  - workspace `Cargo.toml`
  - `crates/codimate-previewer/Cargo.toml`
  - `crates/codimate-previewer/src/bin/preview.rs`
  - `examples/README.md`
  - `examples.manifest.toml`
- Do not commit. Do not run `git add` or `git commit`.

## Validation

Run:

```bash
cargo fmt
cargo check -p codimate-example-<example-name>
cargo check -p codimate-previewer
```

Report changed files and runnable commands:

```bash
cargo run -p codimate-example-<example-name>
cargo run -p codimate-example-<example-name> -- --1080p60
cargo run -p codimate-previewer -- <example-name>
```
