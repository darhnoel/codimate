use codimate_core::{Color, GlyphBlock, Path, PathNode, Segment, Vec2};
use codimate_fonts::{FontId, FontRegistry};

/// Why [`shape()`] could not produce a [`GlyphBlock`].
#[derive(Debug)]
pub enum GlyphError {
    /// No font data for the given `FontId`.
    NoFont(FontId),
    /// HarfBuzz shaping failed.
    Shaping(String),
    /// Glyph outline extraction failed.
    Outline(String),
}

/// Shape `text` into a block of glyph outlines filled with `fill`.
pub fn shape(
    text: &str,
    font_id: FontId,
    font_size: f32,
    fill: Color,
) -> Result<GlyphBlock, GlyphError> {
    if text.is_empty() {
        return Ok(GlyphBlock::empty());
    }

    let registry = FontRegistry::global();
    let data = registry.data(font_id);
    if data.is_empty() {
        return Err(GlyphError::NoFont(font_id));
    }

    // 1. HarfBuzz shaping — get glyph IDs + positions.
    let hb_face = harfbuzz_rs::Face::new(data, 0);
    let upem = hb_face.upem();

    let mut hb_font = harfbuzz_rs::Font::new(hb_face);
    // Scale so positions are in 26.6 fixed-point pixels
    let hb_scale = (font_size * 64.0) as i32;
    hb_font.set_scale(hb_scale, hb_scale);

    let buffer = harfbuzz_rs::UnicodeBuffer::new().add_str(text);
    let glyph_info = harfbuzz_rs::shape(&hb_font, buffer, &[]);

    let positions = glyph_info.get_glyph_positions();
    let infos = glyph_info.get_glyph_infos();

    // 2. TTF outline extraction — convert each glyph to path segments.
    let ttf_face = ttf_parser::Face::parse(data, 0)
        .map_err(|e| GlyphError::Outline(format!("ttf_face: {e}")))?;
    let glyf = ttf_face
        .tables()
        .glyf
        .as_ref()
        .ok_or_else(|| GlyphError::Outline("no glyf table".into()))?;

    let scale_factor = font_size / upem as f32;
    let inv_64 = 1.0 / 64.0;
    let mut cursor_x: f32 = 0.0;
    let mut glyph_nodes = Vec::new();

    for (info, pos) in infos.iter().zip(positions.iter()) {
        let ox = cursor_x + (pos.x_offset as f32) * inv_64;
        let oy = -(pos.y_offset as f32) * inv_64;
        let advance = (pos.x_advance as f32) * inv_64;

        let gid = ttf_parser::GlyphId(info.codepoint as u16);
        let mut collector = OutlineCollector::new(ox, oy, scale_factor);
        let outlines_ok = glyf.outline(gid, &mut collector).is_some();
        let segs = collector.segments();
        if outlines_ok && !segs.is_empty() {
            glyph_nodes.push(
                PathNode::new()
                    .path(Path {
                        segments: segs,
                        closed: false,
                    })
                    .fill(fill),
            );
        }

        cursor_x += advance;
    }

    Ok(GlyphBlock::from_glyphs_with_width(glyph_nodes, cursor_x))
}

/// Converts TTF outline callbacks into [`Segment`]s, applying offset + y-flip.
struct OutlineCollector {
    segs: Vec<Segment>,
    current: (f32, f32),
    ox: f32,
    oy: f32,
    scale: f32,
}

impl OutlineCollector {
    fn new(ox: f32, oy: f32, scale: f32) -> Self {
        Self {
            segs: Vec::new(),
            current: (0.0, 0.0),
            ox,
            oy,
            scale,
        }
    }

    /// Font-unit (x, y) → screen pixel (x, y), flipping y for y-down coords.
    fn p(&self, x: f32, y: f32) -> Vec2 {
        Vec2::new(self.ox + x * self.scale, self.oy - y * self.scale)
    }

    fn segments(self) -> Vec<Segment> {
        self.segs
    }
}

impl ttf_parser::OutlineBuilder for OutlineCollector {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.current = (p.x, p.y);
        self.segs.push(Segment::MoveTo(p));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let from = Vec2::new(self.current.0, self.current.1);
        let to = self.p(x, y);
        self.current = (to.x, to.y);
        self.segs.push(Segment::Line(from, to));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let from = Vec2::new(self.current.0, self.current.1);
        let ctrl = self.p(x1, y1);
        let to = self.p(x, y);
        self.current = (to.x, to.y);
        self.segs.push(Segment::Quad(from, ctrl, to));
    }

    fn curve_to(&mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32) {
        let from = Vec2::new(self.current.0, self.current.1);
        let c1 = self.p(cx1, cy1);
        let c2 = self.p(cx2, cy2);
        let to = self.p(x, y);
        self.current = (to.x, to.y);
        self.segs.push(Segment::Cubic(from, c1, c2, to));
    }

    fn close(&mut self) {
        self.segs.push(Segment::Close);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
