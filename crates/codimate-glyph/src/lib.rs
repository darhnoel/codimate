//! Glyph outlines: text shaped and drawn as paths, ready for the Core.

mod error;
mod shape;

pub use error::GlyphError;
pub use shape::shape;

#[cfg(test)]
mod tests {
    use super::*;
    use codimate_fonts::FontRegistry;

    #[test]
    fn shape_latin_text() {
        let id = FontRegistry::global().char_font('A');
        let block = shape("Hello", id, 20.0, codimate_core::Color::WHITE).unwrap();
        assert!(
            !block.glyphs.is_empty(),
            "should produce at least one glyph"
        );
        assert!(block.width > 0.0);
        assert!(block.height > 0.0);
    }

    #[test]
    fn shape_khmer_text() {
        let id = FontRegistry::global().char_font('ខ');
        let block = shape("ជំរាបសួរ", id, 40.0, codimate_core::Color::WHITE).unwrap();
        assert!(!block.glyphs.is_empty(), "Khmer should produce glyphs");
        assert!(block.width > 0.0);
        assert!(block.height > 0.0);
    }

    #[test]
    fn shape_empty_string() {
        let id = FontRegistry::global().char_font('A');
        let block = shape("", id, 20.0, codimate_core::Color::WHITE).unwrap();
        assert!(block.glyphs.is_empty());
        assert_eq!(block.width, 0.0);
    }
}
