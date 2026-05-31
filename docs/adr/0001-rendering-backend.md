# ADR 0001 — Rendering backend: CPU raster (tiny-skia) + ffmpeg pipe

**Status:** Accepted — 2026-05-31

## Context
Codimate's pipeline produces a renderer-neutral `RenderFrame { viewport, commands: Vec<RenderCommand> }` per frame (Codex's work). Nothing rasterizes to pixels yet. We need to pick a rendering library. Constraints: **minimum dependencies**, efficient, and must not contradict the One Law (`f(t) → Scene`, pure and stateless).

Reference point: **noon** (Manim-inspired, Rust) uses `bevy_ecs` + `nannou` (→ `wgpu`/GPU). That is a heavy GPU + game-engine stack, and — more importantly — it is **retained-mode and stateful**, which fights Codimate's pure-function model.

## Decision
- **Rasterizer: `tiny-skia`** — a single pure-Rust crate (no C/C++, no system libraries, builds anywhere). It draws anti-aliased paths/fills/strokes to an in-memory RGBA `Pixmap`. It is Skia's raster pipeline ported to Rust, so quality and speed are production-grade. It maps directly onto our existing `RenderCommand` stream and the `Renderer` trait Codex defined.
- **Video export: pipe raw RGBA frames straight to `ffmpeg` stdin** (`-f rawvideo -pix_fmt rgba`). `ffmpeg` is an external runtime tool, **not a compile-time Rust dependency** — so the Rust dependency count for full video stays ~1. Raw-pipe is also the most efficient encode path (no per-frame PNG). The `png` crate stays optional, for single-frame debug snapshots only.
- **Rejected for now:** `skia-safe` (heavy native build, sandbox-risky), `wgpu`/`vello` (large GPU stack, nondeterministic AA across drivers), `nannou`/Bevy (huge + stateful, contradicts the model).

## Consequences
- **Determinism → golden-image tests.** Pure `f(t)` + CPU raster = identical bytes every run, so we can assert pixel colors as regression tests (matches the project's TDD ethos). GPU AA varies by driver and would break this.
- **Free parallelism.** Frames are independent pure functions of `t`, so long renders parallelize across threads with no shared state — this is our answer to "long render" performance, not stateful checkpointing.
- **Not locked in.** Because rasterization sits behind the `Renderer` trait, a GPU or full-Skia backend can be **added** later without breaking anything. GPU is an optimization, not an architecture.
- `tiny-skia` lives in `codimate-render` only; `codimate-core` stays std-only (Invariant 6). CONTEXT.md crate map should be updated: `codimate-render` = "tiny-skia raster, Renderer trait" (was "skia-safe").
