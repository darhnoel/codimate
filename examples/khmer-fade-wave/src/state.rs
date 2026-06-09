use codimate_core::{Color, Path};
use codimate_fonts::FontRegistry;
use codimate_layout::Viewport;

pub const TEXT: &str = "ជម្រាបសួរ";

#[derive(Clone, Debug)]
pub struct KhmerFadeWave {
    pub text: &'static str,
    pub font_size: f32,
}

impl Default for KhmerFadeWave {
    fn default() -> Self {
        Self {
            text: TEXT,
            font_size: 72.0,
        }
    }
}

#[derive(Clone)]
pub struct GlyphPaths {
    pub base: Path,
    pub contours: Vec<Path>,
}

#[derive(Clone)]
pub struct KhmerFadeWaveState {
    pub units: Vec<GlyphPaths>,
}

impl KhmerFadeWaveState {
    pub fn shape(concept: &KhmerFadeWave, viewport: Viewport) -> Self {
        let first_char = concept
            .text
            .chars()
            .next()
            .expect("khmer-fade-wave text must not be empty");
        let font_id = FontRegistry::global().char_font(first_char);
        let block =
            codimate_glyph::shape(concept.text, font_id, concept.font_size, Color::WHITE).unwrap();

        let ox = (viewport.width - block.width) / 2.0;
        let oy = viewport.height / 2.0;

        let glyphs: Vec<GlyphPaths> = block
            .glyphs
            .iter()
            .map(|glyph| {
                let resolved = glyph.resolve(0.0);
                let base = resolved.path.translate(ox, oy);
                let contours = base.split_contours();
                GlyphPaths { base, contours }
            })
            .collect();

        Self {
            units: visual_units(&glyphs),
        }
    }
}

fn visual_units(glyphs: &[GlyphPaths]) -> Vec<GlyphPaths> {
    let mut units = Vec::new();
    let mut index = 0;

    if glyphs.len() >= 2 {
        units.push(combine_glyphs(&glyphs[0..2]));
        index = 2;
    }

    for glyph in &glyphs[index..] {
        units.push(GlyphPaths {
            base: glyph.base.clone(),
            contours: glyph.contours.clone(),
        });
    }

    units
}

fn combine_glyphs(glyphs: &[GlyphPaths]) -> GlyphPaths {
    let mut base = Path {
        segments: Vec::new(),
        closed: false,
    };
    let mut contours = Vec::new();

    for glyph in glyphs {
        base.segments.extend(glyph.base.segments.iter().copied());
        contours.extend(glyph.contours.iter().cloned());
    }

    GlyphPaths { base, contours }
}
