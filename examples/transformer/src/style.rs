use codimate_core::{manim, Color};

pub(crate) const BG: Color = manim::BLACK;
pub(crate) const INK: Color = manim::WHITE;
pub(crate) const WIRE: Color = manim::with_alpha(manim::WHITE, 0.92);
pub(crate) const BOX_STROKE: Color = manim::WHITE;
pub(crate) const RESIDUAL_WIRE: Color = manim::with_alpha(manim::TEAL, 0.96);
pub(crate) const POS_WIRE: Color = manim::with_alpha(manim::WHITE, 0.86);
pub(crate) const CONTAINER_FILL: Color = manim::DARKER_GRAY;
pub(crate) const CONTAINER_STROKE: Color = manim::DARK_GRAY;
pub(crate) const FLOW_HIGHLIGHT: Color = manim::BLUE;
pub(crate) const BLOCK_HIGHLIGHT: Color = manim::YELLOW;
pub(crate) const BRIDGE_HIGHLIGHT: Color = manim::TEAL;
pub(crate) const PULSE_C: Color = FLOW_HIGHLIGHT;
pub(crate) const PULSE_D: Color = manim::PURPLE;
pub(crate) const PULSE_X: Color = manim::RED;
