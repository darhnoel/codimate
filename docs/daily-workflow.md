# Daily Workflow

This is the canonical onboarding path for Codimate. The success metric is that a
new contributor can clone the repository, follow the steps below, and get from
zero to a real render in less than 30 minutes.

## Stage 1: First win

Start with `circle-to-square`. It is the smallest Rust example in the
repository, so it proves your toolchain and render path work before you look at
the fuller template.

```bash
cargo run -p codimate-example-circle-to-square --release
```

You should get a rendered result under `results/`.

## Stage 2: Real template

Move to `swap` next. It is the canonical split-template example, with
`state.rs`, `algorithm.rs`, `view.rs`, `motion.rs`, `timing.rs`, `builder.rs`,
and the facade entrypoints already separated the way new examples should be.

```bash
cargo run -p codimate-example-swap --release
```

Once both examples run, continue with the tutorial and the reference docs as
needed.
