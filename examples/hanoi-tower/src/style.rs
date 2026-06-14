use crate::DISK_COUNT;
use codimate::{manim, Color};

pub(crate) const BG: Color = manim::BLACK;
pub(crate) const PANEL: Color = manim::DARKER_GRAY;
pub(crate) const PANEL_BORDER: Color = manim::DARK_GRAY;
pub(crate) const INK: Color = manim::WHITE;
pub(crate) const MUTED: Color = manim::LIGHT_GRAY;
pub(crate) const ACCENT: Color = manim::BLUE;
pub(crate) const TARGET: Color = manim::GREEN_E;
pub(crate) const MOVING: Color = manim::with_alpha(manim::YELLOW, 0.58);
pub(crate) const DISK_STROKE: Color = manim::WHITE;

pub(crate) const DISK_COLORS: [Color; DISK_COUNT + 1] = [
    Color::TRANSPARENT,
    manim::PURPLE,
    manim::ORANGE,
    manim::BLUE_E,
    manim::GREEN_E,
];
