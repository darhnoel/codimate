use codimate_core::{Path, Segment, Vec2};
use codimate_fonts::FontRegistry;
use codimate_layout::Viewport;

use crate::style::LETTER_BLUE;

pub const LOGO_LETTER: &str = "ខ";
pub const SOURCE_LETTER: &str = "K";

#[derive(Clone, Debug)]
pub struct LogoAnimation {
    pub source_letter: &'static str,
    pub letter: &'static str,
    pub card_w: f32,
    pub card_h: f32,
    pub card_radius: f32,
    pub border_w: f32,
    pub font_size: f32,
}

impl Default for LogoAnimation {
    fn default() -> Self {
        Self {
            source_letter: SOURCE_LETTER,
            letter: LOGO_LETTER,
            card_w: 220.0,
            card_h: 240.0,
            card_radius: 18.0,
            border_w: 4.0,
            font_size: 158.0,
        }
    }
}

#[derive(Clone)]
pub struct LogoAnimationState {
    pub concept: LogoAnimation,
    pub viewport: Viewport,
    pub center_x: f32,
    pub center_y: f32,
    pub source_letter_path: Path,
    pub letter_path: Path,
    pub key_centers: Vec<(f32, f32)>,
    pub spacebar_center: (f32, f32),
}

impl LogoAnimationState {
    pub fn shape(concept: LogoAnimation, viewport: Viewport) -> Self {
        let center_x = viewport.width / 2.0;
        let center_y = viewport.height / 2.0;
        let glyph_center_y = center_y - 4.0;
        let mut source = shape_centered_letter(
            concept.source_letter,
            concept.font_size,
            center_x,
            glyph_center_y,
        );
        let target =
            shape_centered_letter(concept.letter, concept.font_size, center_x, glyph_center_y);
        source = match_visual_height(source, &target, center_x, glyph_center_y);

        let top_row_y = center_y + 83.0;
        let start_x = center_x - 48.0;
        let key_centers = (0..5)
            .map(|i| (start_x + i as f32 * 24.0, top_row_y))
            .collect();

        Self {
            concept,
            viewport,
            center_x,
            center_y,
            source_letter_path: source,
            letter_path: target,
            key_centers,
            spacebar_center: (center_x, center_y + 96.0),
        }
    }
}

fn shape_centered_letter(
    letter: &'static str,
    font_size: f32,
    center_x: f32,
    center_y: f32,
) -> Path {
    let letter_char = letter
        .chars()
        .next()
        .expect("logo-animation letter must not be empty");
    let font_id = FontRegistry::global().char_font(letter_char);
    let block = codimate_glyph::shape(letter, font_id, font_size, LETTER_BLUE)
        .unwrap_or_else(|e| panic!("logo-animation glyph shape failed: {e:?}"));

    let mut path = block
        .glyphs
        .first()
        .expect("logo-animation glyph shape must produce a path")
        .resolve(0.0)
        .path;

    if let Some((xmin, ymin, xmax, ymax)) = path.bounding_box() {
        let glyph_cx = (xmin + xmax) / 2.0;
        let glyph_cy = (ymin + ymax) / 2.0;
        path = path.translate(center_x - glyph_cx, center_y - glyph_cy);
    }

    path
}

fn match_visual_height(path: Path, target: &Path, center_x: f32, center_y: f32) -> Path {
    let Some((_, source_min_y, _, source_max_y)) = path.bounding_box() else {
        return path;
    };
    let Some((_, target_min_y, _, target_max_y)) = target.bounding_box() else {
        return path;
    };
    let source_h = source_max_y - source_min_y;
    let target_h = target_max_y - target_min_y;
    if source_h <= 0.0 || target_h <= 0.0 {
        return path;
    }

    scale_path(path, Vec2::new(center_x, center_y), target_h / source_h)
}

fn scale_path(path: Path, center: Vec2, scale: f32) -> Path {
    Path {
        segments: path
            .segments
            .into_iter()
            .map(|segment| scale_segment(segment, center, scale))
            .collect(),
        closed: path.closed,
    }
}

fn scale_point(point: Vec2, center: Vec2, scale: f32) -> Vec2 {
    Vec2::new(
        center.x + (point.x - center.x) * scale,
        center.y + (point.y - center.y) * scale,
    )
}

fn scale_segment(segment: Segment, center: Vec2, scale: f32) -> Segment {
    match segment {
        Segment::MoveTo(p) => Segment::MoveTo(scale_point(p, center, scale)),
        Segment::Line(a, b) => {
            Segment::Line(scale_point(a, center, scale), scale_point(b, center, scale))
        }
        Segment::Quad(a, c, b) => Segment::Quad(
            scale_point(a, center, scale),
            scale_point(c, center, scale),
            scale_point(b, center, scale),
        ),
        Segment::Cubic(a, c1, c2, b) => Segment::Cubic(
            scale_point(a, center, scale),
            scale_point(c1, center, scale),
            scale_point(c2, center, scale),
            scale_point(b, center, scale),
        ),
        Segment::Close => Segment::Close,
    }
}
