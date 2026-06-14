# Examples

Use this with the [Daily Workflow](../docs/daily-workflow.md).

## Tier 1 — First win

- `circle-to-square/` — smallest tutorial run; confirms toolchain, export, and
  preview loop.

```bash
cargo run -p codimate-example-circle-to-square
cargo run -p codimate-previewer -- circle-to-square
```

## Tier 2 — Daily authoring templates

- `box-arrow/` — **primitive-first API tutorial** (box + arrow + `t1..t2` flow).
- `swap/` — **canonical copy-template** for new explanations.
- `demo/` — compact authoring tour with the standard split.
- `connection-pulse/` — minimal Connection/Pulse with the standard split.

Canonical split (copy from `swap/`):

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

## Tier 3 — Advanced explainers

- `dijkstra/` — shortest-path settling and relax steps.
- `faster-dijkstra/` — directed SSSP paper explainer.
- `knapsack/` — dynamic programming table fill and backtrack.
- `newton-laws/` — force and action-reaction concepts.
- `matrix-mult/`, `merge-sort/`, `quick-sort/`, `insertion-sort/`,
  `neural-net/`, `three-sum/`, `symspell/`, `hanoi-tower/`, `word-appear/`,
  `khmer-fade-wave/`, `swap-a-b/`, `preview/`.

## New animation checklist

1. Copy `swap/` as your starting point.
2. Rename state/trace/view terms to your concept language.
3. Keep motion timeless (`motion.rs`), durations in `timing.rs`.
4. Keep `main.rs` as the clean authoring chain.
5. Run after each small change.
