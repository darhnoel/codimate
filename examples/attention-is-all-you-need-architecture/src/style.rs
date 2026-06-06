use codimate_core::{manim, Color};

pub(crate) const BG: Color = manim::BLACK;
pub(crate) const INK: Color = manim::WHITE;
pub(crate) const MUTED: Color = manim::LIGHT_GRAY;
pub(crate) const PANEL: Color = manim::DARKER_GRAY;
pub(crate) const ENCODER: Color = manim::BLUE_E;
pub(crate) const DECODER: Color = manim::PURPLE;
pub(crate) const EMBEDDING: Color = manim::RED_E;
pub(crate) const POSITION: Color = manim::TEAL;
pub(crate) const ATTENTION: Color = manim::ORANGE;
pub(crate) const NORM: Color = manim::YELLOW;
pub(crate) const FEED_FORWARD: Color = manim::GREEN_E;
pub(crate) const OUTPUT: Color = manim::RED;
pub(crate) const MEMORY: Color = manim::TEAL;
pub(crate) const HIGHLIGHT: Color = manim::YELLOW;
pub(crate) const WIRE: Color = manim::with_alpha(manim::WHITE, 0.86);
pub(crate) const CONTEXT_WIRE: Color = manim::with_alpha(manim::WHITE, 0.28);
pub(crate) const RESIDUAL: Color = manim::with_alpha(manim::TEAL, 0.95);

pub(crate) fn alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}
