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

## Status

Done on the `codimate-js` branch, one commit per slice, each passing
`tools/check.py` (Rust, Python and JavaScript tests, every Conformance Example
byte for byte in both languages, the Waist's cost):

| Slice | Into the Core |
|---|---|
| 1 | the hourglass: Waist, Manifest, generator, both Skins; rect, circle, text, fill, film data |
| 2 | line, arrow, polygon, curve, arc |
| 3 | group |
| 4 | ngon, star, row, column, measure, ease |
| 5 | Rule |
| 6 | turn, grow, on, round, write |
| 7 | text presets |
| 8 | formula, image, svg, focus, overlay |
| 9 | markers and highlights (Python's random, bit for bit) |
| 10 | axes |
| 11 | the Explanation's timeline, chapters and covered labels |
| last | `python/` to `skins/python/`; codimate-py and codimate-wasm into `codimate-waist` |

Stay in the Python Skin, decided: the Science Kit, the Previewer and viewer,
the index, the contact sheet, and sound mixing. Timing stays in each Skin: it
is an extension point an author subclasses.

Follow-ups: per-language examples in the Manifest's docs (the JavaScript ones
are the Python originals); an npm release; sound in the JavaScript Skin.

## Where the prototype fits

Before slice 1 the branch had: rustybuzz in place of HarfBuzz (every
crate builds for WebAssembly), `FontRegistry::provide`, a `codimate-wasm` crate
that plays a saved film, and `Explanation.export_web`. Slice 1 reshapes
`codimate-wasm` into the Waist and `export_web`'s format into the film data the
Conformance Examples compare.
