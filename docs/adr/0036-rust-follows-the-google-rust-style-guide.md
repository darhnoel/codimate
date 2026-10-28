# ADR 0036 — Rust follows the Google Rust Style Guide

**Status:** Accepted — 2026-10-28. Applied in the same change as this record.

The Engine's Rust crates had no written style rule. `CONTEXT.md` pins only the `cargo fmt`
output, so a review had nothing to point at. This ADR adopts one, records how the code measures
against it, and names the exceptions the code keeps on purpose.

## Decision

The Rust crates follow the [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html).
Where it conflicts with the official Rust Style Guide or the Rust API Guidelines, the Google
guide wins (its §2). The Engine is library code that Skins call, so the guide's API design rules
are binding (its §3).

Sections written for Chrome, Android and Fuchsia do not apply here.

Named exceptions:

- Glob imports are allowed for `use super::*` in `#[cfg(test)]` modules, for `pub use module::*`
  re-exports in crate roots, and for third-party preludes (`pyo3::prelude::*`,
  `wasm_bindgen::prelude::*`, `rayon::prelude::*`).
- `unwrap()` and `expect()` are allowed in tests and build scripts.
- A `#[allow(clippy::…)]` gets a comment saying why.
- Generated code is not hand-edited. `codimate-author/src/generated.rs` comes from
  `tools/generate.py`, and the font table comes from `codimate-fonts/build.rs`. The rules apply
  to what they generate, not to the generated text.

## Rules

Each rule is cited by its guide section name.

1. **Don't use glob imports.** Import the names a file uses. Production code does not write
   `use crate::*;` or `use crate::value::*;`.
2. **Don't implement APIs in non-leaf modules.** A crate root or a `mod.rs` holds module
   declarations, re-exports and tests. Functions and types live in leaf modules named after
   their main item. The one exception is `codimate-fonts`, whose root `include!`s the generated
   font table.
3. **Reporting errors.** `Result` for failures a caller can handle, `Option` for plain absence,
   `panic!` or `expect` for broken invariants. An `expect` message says which invariant broke.
   Exception: the `codimate-author` surface returns `Result` from every operation, because
   `tools/generate.py` emits one dispatcher for all of them. Those 19 signatures stay until the
   generator changes.
4. **Prefer `enum` over `bool` in APIs.** A `bool` argument whose meaning is not clear at the
   call site becomes an enum, or gets a comment.
5. **Accept flexible references.** `&Option<T>` becomes `Option<&T>`, `&Vec<T>` becomes `&[T]`,
   `&String` becomes `&str`, and `&Box<T>` becomes `&T`.
6. **Avoid `Rc` and `Arc`.** `Arc` stays where a cached value is shared across calls, or where a
   closure must be cloned. Any other use needs a reason in review.
7. **Document public APIs.** Every public type, function, method, constant, trait, module and
   the crate carry a `///` or `//!` doc. A public function that can panic says so under
   `# Panics`; one that returns `Err` says so under `# Errors`. Struct fields and enum variants
   are documented where the name does not carry the unit, the coordinate space or the meaning.
   Self-explanatory ones may stay undocumented, as the guide allows for variants.
8. **Public enums and structs are `#[non_exhaustive]` at the error boundary.** The error types
   (`Error`, `Refusal`, `ExportError`, `FormulaError`, `GlyphError`, `SceneTransformError`) are
   non-exhaustive, because Skins and other callers match on them. Every other public enum and
   struct stays exhaustive: the crates ship from one repository at one version (ADR 0033), so
   a new variant should fail to compile wherever it is not yet handled, rather than fall into a
   wildcard arm. Fixed shapes (a colour, a point, a rectangle, the path command set) stay
   exhaustive as the guide allows. `Path` and the `Concrete*` structs are built by struct
   literal in other crates, so they cannot be non-exhaustive until they have constructors. That
   is an open follow-up.
9. **`#[must_use]`** on functions whose result must not be ignored: pure functions, builder
   methods that take and return `self`, and error types.
10. **Unsafe code.** None in the Engine today. A new block needs a `// SAFETY:` comment saying why
    no safe version works, and the benchmark if speed is the reason.
11. **Formatting.** `cargo fmt --check` passes. Do not use `#[rustfmt::skip]` for personal taste.
12. **Naming.** Conversions follow `as_`, `to_` and `into_`. Avoid abbreviations a reader outside
    the project would not know.

## State after adoption (2026-10-28)

Measured with `cargo fmt --check`, `cargo doc --no-deps`, and clippy with the rule lints
enabled. The workspace test suite passes (251 tests), the Python skin tests pass, and
`tools/generate.py --check` and `tools/conformance.py` pass.

| Rule | Where it stands |
|---|---|
| 1 Glob imports | None in production. The three third-party preludes and the test-module `super::*` imports are the exceptions. |
| 2 Non-leaf modules | Every crate root and `mod.rs` holds declarations, re-exports and tests only. |
| 3 Errors | 0 production `unwrap()`. 24 production `expect()`, each with a message. 19 surface functions keep `Result` (see the exception above). |
| 4 Bool arguments | 0. Two were turned into enums: `Outline` for arcs and sectors, `Framing` for the film's overlap check. |
| 5 Flexible references | 0. `fit_box` takes `Option<&Radius>`. |
| 6 Rc and Arc | Shared caches in `codimate-reconcile/src/media.rs`, and closures in `codimate-core` (`Animated`, `TimeCurve`) and `codimate-reconcile/src/motion.rs`. Each is reviewed here; no new use is planned. |
| 7 Public docs | 0 public items undocumented. 213 struct fields and enum variants have no doc (see Open items). |
| 8 Non-exhaustive | 6 error types. Other public types stay exhaustive, as above. |
| 9 `#[must_use]` | Applied to every candidate clippy reports. 0 remain. |
| 10 Unsafe | None. |
| 11 Formatting | Clean. |
| 12 Naming | No violations found. `pub struct Arc` in `codimate-author/src/surface.rs` is the `arc` operation's argument type, named like the other operation types (`Rect`, `Circle`). It does not clash with `std::sync::Arc` in that module. |

## Open items

- Struct fields and enum variants: 213 are undocumented. Most names say what they mean (`x`,
  `width`, `fill`). The ones that need words (`pixels`, `rgba`, `layer`, `closed`, `scale_y`,
  `edge_w`) are the next pass, if the team wants every field documented.
- `Path` and the `Concrete*` structs need constructors before they can be `#[non_exhaustive]`.
- Rule 6: confirm the `Animated` and `TimeCurve` closures are `Arc` by necessity, not habit.
- Rule 3: the surface's `Result` signatures are fixed by the generator. Changing them means
  changing `tools/generate.py`.

## Changing it

Change the rules here, not in a review comment. Re-measure after a change lands, and update the
state table, not the rules.
