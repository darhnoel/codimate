# Plan: many Skins over one Authoring Core

How Codimate becomes one library in several languages, starting with Python and
JavaScript/TypeScript (codimate-js). Terms are in [CONTEXT.md](../CONTEXT.md);
the decisions and their reasons are ADRs 0030–0034.

## The shape

```
  Python Skin   JavaScript Skin   C++ Skin   …        generated from the Interface Manifest; no logic
        \             |              |       /
         ──────────── the Waist: plain data in and out ────────────
                              |
                    Authoring Core (Rust)                 all authoring logic, once
                              |
                        Engine (Rust)                     tween, draw, encode
```

- **Interface Manifest** (`api/surface.toml`): every public function, argument,
  type and default. Python names are snake_case; JavaScript names are the same
  names in camelCase, by one rule, no exceptions. Keyword arguments become one
  trailing options object; tuples become arrays.
- **Generator** (`tools/generate/`): Manifest → each Skin's declarations, a Rust
  trait the Core must implement, and the API reference page.
- **Skins** record Builder calls (with the author's file and line) and send each
  Scene across the Waist in one call; Queries (`measure`, Slots, `ease`) cross
  one at a time. What a Skin writes by hand: `trace`/`emit`, calling the
  author's functions, files, fonts, playback.
- **Waist** (`crates/codimate-waist/`): build a Scene from recorded calls, answer
  a Query, build a film. WebAssembly for JavaScript; one C interface for native
  languages.

## Layout (one repository, one version: ADR 0033)

```
api/surface.toml            the Interface Manifest
crates/codimate-author/     the Authoring Core
crates/codimate-waist/      the Waist (absorbs codimate-py and codimate-wasm glue)
crates/…                    the Engine, unchanged
skins/python/               moved from python/; still `codimate` on PyPI
skins/js/                   `codimate` on npm
conformance/NNN_name/       name.py · name.ts · expected.film.json
tools/generate/
```

## Tests (TDD, through the interface)

| Layer | Checks | Written |
|---|---|---|
| 1. Conformance Examples | an explanation in every language → film data identical, byte for byte, to the committed `expected.film.json` | first, for every feature |
| 2. Waist tests (Rust) | recorded calls → Scene data or errors naming the author's line | for each Core behaviour |
| 3. Core unit tests | only the hard algorithms (text layout, anchor maths) | where earned |
| 4. Generator tests | Manifest → expected Skin code | when the generator changes |

The Core, not each language, writes film data, so bytes can match. Python's
existing tests stay green throughout the migration, as the safety net.

Every change: Conformance Example (red everywhere) → Waist test (red) → Core
until both pass → regenerate Skins → green in every language. No Core code
without a failing test first.

## JavaScript specifics

- **State** is a typed object (`type Ball = { x: number }`), TypeScript's
  counterpart of the dataclass (ADR 0021). `trace` deep-copies it at every
  `emit`, keeping class instances' classes; `snapshot:` works as in Python.
  `emit("move", { x })` is Python's `emit("move", x=x)`. Items keep identity by
  `id`. No `async` algorithm or view: Python has no counterpart.
- **Fonts** by family (ADR 0034): pages add their own with `cm.fonts.add`;
  DejaVu Sans Mono, Inter, JetBrains Mono and Noto Sans JP are built in and
  load only when used.
- **Output:** `film.play(canvas)` (live, scrubbable, step by event) is the
  browser's main use. MP4 through ffmpeg in Node, through WebCodecs in the
  browser. Sound export in Node first.

## Order of work

Each slice is a vertical cut: Conformance Example → Manifest → Core → Python
switched to the Waist (its tests green) → Python's old code for the slice
deleted.

1. **Slice 1, the whole hourglass, thinly:** Waist (three functions), generator,
   recorder in both Skins; `canvas`, `Scene`, `rect`, `circle`, `text`, `fill`,
   `trace`/`emit`, `explain`, browser play and MP4. Conformance #1: a ball moves
   left to right with a label, identical from Python and TypeScript. A
   benchmark on a large film guards recording speed.
2. Lines, arrows, paths.
3. Groups.
4. Anchors and Slots (`row`, `column`, placement).
5. Rule and Timing.
6. Text presets.
7. Whatever remains of the core Surface.

**Later, each its own decision:** Science Kit, Previewer, viewer, compact index,
a C++ Skin.

## Where the prototype fits

The `codimate-js` branch already has: rustybuzz in place of HarfBuzz (every
crate builds for WebAssembly), `FontRegistry::provide`, a `codimate-wasm` crate
that plays a saved film, and `Explanation.export_web`. Slice 1 reshapes
`codimate-wasm` into the Waist and `export_web`'s format into the film data the
Conformance Examples compare.
