//! The errors `shape` can return.

use codimate_fonts::FontId;

/// Why [`shape`] could not produce a [`GlyphBlock`].
#[derive(Debug)]
#[non_exhaustive]
pub enum GlyphError {
    /// No font data for the given `FontId`.
    NoFont(FontId),
    /// Shaping failed: the font could not be read for shaping.
    Shaping(String),
    /// Glyph outline extraction failed.
    Outline(String),
}
