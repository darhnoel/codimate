//! codimate-core — the two timeless layers behind the One Law.
//!
//! The One Law: an animation is a pure function from time to a visual scene —
//! `f(t) → Scene`. This crate holds the two layers that are *timeless* (no
//! duration) and *pure*:
//!
//! - [`value`] — **Layer 1 (Value)**: how a single value changes over `t`.
//!   `Animated<T>`, `IntoAnimated`, `Lerp`, `tween`, the easing curves, and the
//!   geometry and style leaf types (`Vec2`, `Color`, `Style`, `Segment`, `Path`).
//! - [`scene`] — **Layer 2 (Scene)**: what exists at a moment in time. The
//!   `Node` trait, the node types (`Circle`, `Rect`, `PathNode`, `Text`,
//!   `Connection`, `Pulse`), anchors, and the `Scene` tree.
//!
//! Layer 3 (Composition, where `duration` lives) is in `codimate-animation`.
//!
//! Invariant 6: this crate has zero non-pure dependencies (std only).
//!
//! Everything is re-exported flat (`codimate_core::circle`, `::Animated`, …);
//! the modules exist to mirror the layers, not to change the public surface.

pub mod scene;
pub mod value;

pub use scene::*;
pub use value::*;
