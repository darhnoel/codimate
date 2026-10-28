//! codimate-layout — Slot-based View layout helpers.
//!
//! A `Slot` is a View-only layout position with a top-left point and size.
//! It is not concept state, not a Scene Node, and not a runtime layout pass.
//! Use Slots in View code before reaching for raw coordinates.

use crate::Slot;

// ---------------------------------------------------------------------------
// Viewport
// ---------------------------------------------------------------------------

/// The output size a concrete Scene should be laid out within.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    /// A viewport of `width` by `height` pixels.
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// The viewport itself as a Slot (full screen).
    #[must_use]
    pub fn slot(&self) -> Slot {
        Slot::new(0.0, 0.0, self.width, self.height)
    }
}

/// A concrete Scene paired with viewport layout context.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutFrame {
    pub viewport: Viewport,
    pub scene: codimate_core::ConcreteScene,
}

/// Pure layout boundary: no rendering, no I/O.
#[must_use]
pub fn layout_scene(scene: codimate_core::ConcreteScene, viewport: Viewport) -> LayoutFrame {
    LayoutFrame { viewport, scene }
}
