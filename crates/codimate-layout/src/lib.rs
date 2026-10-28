//! codimate-layout — Slot-based View layout helpers.
//!
//! A `Slot` is a View-only layout position with a top-left point and size.
//! It is not concept state, not a Scene Node, and not a runtime layout pass.
//! Use Slots in View code before reaching for raw coordinates.

mod boxes;
mod labels;
mod scene;
mod slot;
mod stack;

pub use boxes::*;
pub use labels::*;
pub use scene::*;
pub use slot::*;
pub use stack::*;
