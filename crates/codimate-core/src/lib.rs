//! codimate-core — the two timeless layers behind the One Law.
//!
//! The One Law: an animation is a pure function from time to a visual scene —
//! `f(t) → Scene`. This crate holds the layers that are *timeless* (no
//! duration) and *pure*:
//!
//! - [`value`] — **Layer 1 (Value)**: how a single value changes over `t`.
//!   `Animated<T>`, `IntoAnimated`, `Lerp`, `tween`, and the leaf types
//!   (`Vec2`, `Color`, `Style`).
//! - [`path`] — **Geometry**: `Segment`, `Path`, and shape constructors
//!   (`rect_path`, `circle_path`, `polygon_path`, `ellipse_path`, …).
//! - [`easing`] — **Easing curves**: `ease_in`, `ease_out`, `ease_in_out`, `back`.
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

pub mod builder;
pub mod easing;
pub mod manim_palette;
pub mod path;
pub mod scene;
pub mod value;

pub use builder::*;
pub use easing::*;
pub use manim_palette as manim;
pub use path::*;
pub use scene::*;
pub use value::*;
