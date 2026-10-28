//! Shaping text into outlines: HarfBuzz rules for the glyphs, TrueType outlines for the shapes.

use codimate_core::{Color, GlyphBlock, Path, PathNode, Segment, Vec2};
use codimate_fonts::{FontId, FontRegistry};

use crate::error::GlyphError;

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

    // 1. Shaping — glyph IDs + positions. rustybuzz is HarfBuzz ported to Rust,
    // so the same code builds for the browser (WebAssembly) and the server.
    let index = registry.index(font_id);
    let face = rustybuzz::Face::from_slice(data, index)
        .ok_or_else(|| GlyphError::Shaping(format!("cannot read font {font_id:?}")))?;
    let upem = face.units_per_em() as i64;

    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(text);
    let glyph_buffer = rustybuzz::shape(&face, &[], buffer);

    let positions = glyph_buffer.glyph_positions();
    let infos = glyph_buffer.glyph_infos();

    // rustybuzz answers in font units. HarfBuzz used to be asked for 26.6
    // fixed-point pixels and rounded each value there, so do the same rounding,
    // the way HarfBuzz does it (`em_mult`, then round half up): text lands on
    // exactly the 1/64 px it always did, and golden images stay put.
    let scale_26_6 = (font_size * 64.0) as i64;
    let em_mult = (scale_26_6 << 16) / upem;
    let to_26_6 = |v: i32| ((v as i64 * em_mult + 32768) >> 16) as f32;

    // 2. TTF outline extraction — convert each glyph to path segments.
    // `outline_glyph` reads TrueType and CFF outlines alike, so an `.otf`
    // draws as well as a `.ttf`.
    let ttf_face = ttf_parser::Face::parse(data, index)
        .map_err(|e| GlyphError::Outline(format!("ttf_face: {e}")))?;

    let scale_factor = font_size / upem as f32; // font units to pixels, for outlines
    let inv_64 = 1.0 / 64.0;
    let mut cursor_x: f32 = 0.0;
    let mut glyph_nodes = Vec::new();

    for (info, pos) in infos.iter().zip(positions.iter()) {
        let ox = cursor_x + to_26_6(pos.x_offset) * inv_64;
        let oy = -to_26_6(pos.y_offset) * inv_64;
        let advance = to_26_6(pos.x_advance) * inv_64;

        let gid = ttf_parser::GlyphId(info.glyph_id as u16);
        let mut collector = OutlineCollector::new(ox, oy, scale_factor);
        let outlines_ok = ttf_face.outline_glyph(gid, &mut collector).is_some();
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
