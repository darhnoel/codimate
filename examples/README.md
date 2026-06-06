# Examples

- `circle-to-square/` — smallest tutorial example: a non-filled circle moves
  and morphs into a square, then returns to a circle.
- `demo/` — compile-checked authoring example that resolves a named animation,
  samples preview/export frames, lays them out, and produces renderer-neutral
  commands.
- `connection-pulse/` — minimal Connection/Pulse example using the standard
  State, Algorithm, View, Motion, Timing, and Builder split.
- `dijkstra/` — Dijkstra's shortest path on a small weighted graph: each step
  settles the closest unsettled node and relaxes its edges, growing the
  shortest-path tree while distance labels update.
- `faster-dijkstra/` — explainer for Duan, Mao, Mao, Shu, and Yin's directed
  SSSP paper: bounded multi-source shortest paths, pivot selection, recursive
  batches, and the `O(m log^(2/3) n)` result.
- `knapsack/` — 0/1 knapsack dynamic programming: the value table is filled one
  cell at a time as `max(skip, take)`, then backtracked to highlight which
  items make the optimal solution.
- `newton-laws/` — Newton's three laws of motion: inertia, force-driven
  acceleration, and equal/opposite action-reaction force pairs.
- `attention-is-all-you-need-architecture/` — original Transformer architecture
  from the paper, progressively revealed from a sequence-to-sequence mental
  model to encoder, decoder, residual paths, cross-attention, Linear, Softmax,
  and output probabilities.
- `swap-a-b/` — minimal layout-authoring example: fixed visual slots stay in
  place while stable concept items `A` and `B` swap their slot mapping.

Run it with:

```bash
cargo run -p codimate-example-attention-is-all-you-need-architecture
cargo run -p codimate-example-attention-is-all-you-need-architecture -- --1080p60
cargo run -p codimate-example-attention-is-all-you-need-architecture -- --debug-jpg-scenes
cargo run -p codimate-previewer -- attention-is-all-you-need-architecture
```
