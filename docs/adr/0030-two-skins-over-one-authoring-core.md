# ADR 0030 — Two Skins over one Authoring Core

**Status:** Accepted — 2026-10-24. Supersedes ADR 0008's "Python is the only Authoring Surface"; keeps its per-event versus per-frame split and its data-structure boundary.

ADR 0008 rejected two front doors because every feature would ship twice and one side would rot. codimate-js needs a second front door with the same interface, so instead of two implementations there is one: the parts of the Authoring Surface that only build data (Scene, Group and shape builders, anchors and Slots, Rule and Timing, text presets) move into a Rust **Authoring Core**, and Python (PyO3) and JavaScript (wasm-bindgen) each expose it through a thin **Skin** that keeps only what must run in that language: calling the author's algorithm and view, `emit`, and files. A change is made once, in the Core, and both languages have it. ADR 0008's objection to Rust-backed handles does not apply: the Core builds flat data, never `Animated<T>` closures.

## Considered Options

- **Two hand-written Surfaces checked by conformance tests.** Drift is caught but every feature is still written twice: ADR 0008's problem with a safety net.
- **TypeScript generated from Python.** One direction only, and Python's dynamic features do not translate.

## Consequences

- The Python package's public API does not change; its existing tests become the specification the Core must pass.
- The Science Kit, Previewer, viewer and compact index are outside the first Core. Each is decided separately.
