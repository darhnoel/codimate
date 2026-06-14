use codimate::{manim, Color};

// Canvas + text.
pub(crate) const BG: Color = manim::BLACK;
pub(crate) const INK: Color = manim::WHITE;
pub(crate) const MUTED: Color = manim::LIGHT_GRAY;

// Table cells by role.
pub(crate) const CELL_EMPTY: Color = manim::DARKER_GRAY; // not computed yet
pub(crate) const CELL_FILLED: Color = manim::BLUE_E; // settled value
pub(crate) const CELL_CURRENT: Color = manim::ORANGE; // computed this step
pub(crate) const CAND_SKIP: Color = manim::BLUE; // dp[i-1][c] (skip the item)
pub(crate) const CAND_TAKE: Color = manim::TEAL; // value + dp[i-1][c-w] (take it)
pub(crate) const CELL_PATH: Color = manim::GREEN_E; // on the solution backtrack

// Cell borders.
pub(crate) const BORDER: Color = manim::WHITE;
pub(crate) const SUBTLE_BORDER: Color = manim::DARK_GRAY;

// Item list panel.
pub(crate) const ITEM_ACTIVE: Color = manim::ORANGE;
pub(crate) const ITEM_CHOSEN: Color = manim::GREEN;
