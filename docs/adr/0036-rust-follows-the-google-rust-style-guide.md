# ADR 0036 — Rust follows the Google Rust Style Guide

**Status:** Accepted — 2026-10-28.

The Engine's Rust crates had no written style rule. `CONTEXT.md` pins only the `cargo fmt`
output, so a review had nothing to point at. This ADR adopts one, and records how the code
measures against it on the day it was adopted.

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

## Rules

Each rule is cited by its guide section name.

1. **Don't use glob imports.** Import the names a file uses. Production code does not write
   `use crate::*;` or `use crate::value::*;`.
2. **Don't implement APIs in non-leaf modules.** A crate root or a `mod.rs` holds declarations,
   re-exports and small glue. Functions live in leaf modules.
3. **Reporting errors.** `Result` for failures a caller can handle, `Option` for plain absence,
   `panic!` or `expect` for broken invariants. An `expect` message says which invariant broke.
4. **Prefer `enum` over `bool` in APIs.** A `bool` argument whose meaning is not clear at the
   call site becomes an enum, or gets a comment.
5. **Accept flexible references.** `&Option<T>` becomes `Option<&T>`, `&Vec<T>` becomes `&[T]`,
   `&String` becomes `&str`, and `&Box<T>` becomes `&T`.
6. **Avoid `Rc` and `Arc`.** `Arc` stays where a cached value is shared across calls, or where a
   closure must be cloned. Any other use needs a reason in review.
7. **Document public APIs.** Public items get `///` docs. A public function that can panic says
   so under `# Panics`; one that returns `Err` says so under `# Errors`.
8. **Public enums and structs are `#[non_exhaustive]`** when other crates or Skins match on or
   build them, unless the shape is fixed (a colour, a rectangle), which the guide allows.
9. **`#[must_use]`** on functions whose result must not be ignored, and on error types.
10. **Unsafe code.** None in the Engine today. A new block needs a `// SAFETY:` comment saying why
    no safe version works, and the benchmark if speed is the reason.
11. **Formatting.** `cargo fmt --check` passes. Do not use `#[rustfmt::skip]` for personal taste.
12. **Naming.** Conversions follow `as_`, `to_` and `into_`. Avoid abbreviations a reader outside
    the project would not know.

## State at adoption (2026-10-28)

Measured with `cargo fmt --check` and clippy with the rule lints enabled (`missing_docs`,
`unwrap_used`, `expect_used`, `exhaustive_enums`, `exhaustive_structs`, `glob_imports`,
`ptr_arg`, `borrowed_box`, `new_without_default`, `should_implement_trait`,
`len_without_is_empty`, `wrong_self_convention`, and pedantic). The numbers are a baseline, not
a gate.

| Rule | Where it stands |
|---|---|
| 11 Formatting | Clean. |
| 1 Glob imports | 12 in production: `use crate::*;` in seven `codimate-reconcile` modules (pen, segment, shape, media, camera, explanation, motion), and `use crate::value::*;` in five `codimate-core` scene modules. |
| 2 Non-leaf modules | Crate roots and `scene/mod.rs` with functions: layout 40, scene 25, animation 35, export 24, fonts 19, math 19, glyph 9. |
| 3 Errors | 25 production `unwrap()` and `expect()` calls, 8 of them in `codimate-reconcile/src/media.rs`. 19 functions wrap a return value in `Result` or `Option` that is never a failure or never empty. |
| 4 Bool arguments | 2 in production: `arc_at(..., sector: bool)` and `covered(..., focused: bool)`. |
| 5 Flexible references | 1: `fit_box(size: &Option<Radius>)` in `codimate-author/src/surface.rs`. |
| 6 Rc and Arc | Shared caches in `codimate-reconcile/src/media.rs`, and closures in `codimate-core/src/value.rs` and `codimate-reconcile/src/motion.rs`. Not yet reviewed use by use. |
| 7 Public docs | `missing_docs` reports 361 items. |
| 8 Non-exhaustive | 0 of 129 public enums and structs. |
| 9 `#[must_use]` | 120 candidate functions and methods, and 83 methods returning `Self`. |
| 10 Unsafe | None. |
| 12 Naming | `codimate-author/src/surface.rs` defines `pub struct Arc`, which reads as `std::sync::Arc`. |

Open decisions, not settled by this ADR: which crates count as public API for rules 7 and 8, and
whether `Animated`'s `Arc<dyn Fn>` is within rule 6.

## Changing it

Change the rules here, not in a review comment. Re-measure after a slice lands, and update the
table, not the rules.
