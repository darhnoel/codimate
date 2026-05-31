//! codimate-layout — layout pass boundary.
//!
//! The real taffy integration belongs here. This first slice only establishes
//! the crate boundary and the pure data shape renderers can consume.

use codimate_core::ConcreteScene;

/// The output size a concrete Scene should be laid out within.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// A concrete Scene paired with viewport layout context.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutFrame {
    pub viewport: Viewport,
    pub scene: ConcreteScene,
}

/// Pure layout boundary: no rendering, no I/O.
pub fn layout_scene(scene: ConcreteScene, viewport: Viewport) -> LayoutFrame {
    LayoutFrame { viewport, scene }
}
