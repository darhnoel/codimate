use codimate::{manim, Color};

// Canvas + text.
pub(crate) const BG: Color = manim::BLACK;
pub(crate) const INK: Color = manim::WHITE;
pub(crate) const MUTED: Color = manim::LIGHT_GRAY;
pub(crate) const STROKE: Color = manim::WHITE;

// Node fills by role.
pub(crate) const NODE_FAR: Color = manim::DARKER_GRAY; // distance still ∞
pub(crate) const NODE_FRONTIER: Color = manim::BLUE; // reachable, not yet settled
pub(crate) const NODE_SETTLED: Color = manim::GREEN_E; // final distance known
pub(crate) const NODE_CURRENT: Color = manim::ORANGE; // being settled this step

// Edges by role.
pub(crate) const EDGE_DIM: Color = manim::DARK_GRAY; // not in the tree
pub(crate) const EDGE_TREE: Color = manim::GREEN; // current best-path tree
pub(crate) const EDGE_RELAX: Color = manim::YELLOW; // relaxed this step

// Distance label highlight when a value drops this step.
pub(crate) const DIST_IMPROVED: Color = manim::GOLD;
