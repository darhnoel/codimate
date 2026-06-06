use std::fmt;

use crate::PathNode;

/// A block of shaped glyph paths.
///
/// Produced by both [`codimate_math::formula()`] (LaTeX → paths) and
/// [`codimate_glyph::shape()`] (text → paths). Each glyph is a first-class
/// [`PathNode`] so it can be tweened, styled, or transformed independently.
///
/// `width`/`height` describe the overall bounding box. Glyph paths are stored
/// as-is (caller positions them); the bounding box is computed from their
/// geometry.
#[derive(Clone)]
pub struct GlyphBlock {
    pub glyphs: Vec<PathNode>,
    pub width: f32,
    pub height: f32,
}

impl fmt::Debug for GlyphBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GlyphBlock")
            .field("glyph_count", &self.glyphs.len())
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl GlyphBlock {
    /// Construct from positioned glyph path nodes, computing the bounding box.
    pub fn from_glyphs(glyphs: Vec<PathNode>) -> Self {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for g in &glyphs {
            let resolved = g.resolve(0.0);
            if let Some((xmin, ymin, xmax, ymax)) = resolved.path.bounding_box() {
                min_x = min_x.min(xmin);
                min_y = min_y.min(ymin);
                max_x = max_x.max(xmax);
                max_y = max_y.max(ymax);
            }
        }
        Self {
            glyphs,
            width: if max_x > min_x { max_x - min_x } else { 0.0 },
            height: if max_y > min_y { max_y - min_y } else { 0.0 },
        }
    }

    /// An empty `GlyphBlock`.
    pub fn empty() -> Self {
        Self {
            glyphs: Vec::new(),
            width: 0.0,
            height: 0.0,
        }
    }
}
