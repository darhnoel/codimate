use codimate_animation::{animation, sequence};
use codimate_core::*;
use codimate_export::{export_mp4, ExportConfig};
use codimate_fonts::FontRegistry;
use codimate_layout::Viewport;

struct GlyphPaths {
    base: Path,
    contours: Vec<Path>,
}

fn main() {
    let viewport = Viewport::new(800.0, 600.0);

    let font_id = FontRegistry::global().char_font('ខ');
    let block = codimate_glyph::shape("ជំរាបសួរ", font_id, 72.0, Color::WHITE).unwrap();

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
    let units = visual_units(&glyphs);

    let n = units.len() as f32;
    let reveal_dur = 2.0;
    let wave_dur = 1.2;
    let stagger = 0.06;
    let wave_total = wave_dur + (n - 1.0) * stagger;
    let unit_span = 1.0 / n.max(1.0);
    let contour_fraction = 0.68;

    let mut reveal_scene = scene();
    for (ui, unit) in units.iter().enumerate() {
        let start = ui as f32 * unit_span;
        let end = start + unit_span;
        let contour_end = start + unit_span * contour_fraction;

        for path in &unit.contours {
            let path = path.clone();
            let revealed = Animated::new(move |t| {
                let local = ((t - start) / (contour_end - start)).clamp(0.0, 1.0);
                path.prefix(local)
            });
            let stroke_color = Animated::new(move |t| {
                let fill_t = ((t - contour_end) / (end - contour_end)).clamp(0.0, 1.0);
                Color {
                    a: 1.0 - fill_t,
                    ..Color::WHITE
                }
            });
            reveal_scene = reveal_scene.node(
                path_node()
                    .path(revealed)
                    .stroke(2.0, stroke_color)
                    .fill(Color::TRANSPARENT),
            );
        }

        let fill_path = unit.base.clone();
        let fill_color = Animated::new(move |t| {
            let fill_t = ((t - contour_end) / (end - contour_end)).clamp(0.0, 1.0);
            Color {
                a: fill_t,
                ..Color::WHITE
            }
        });
        reveal_scene = reveal_scene.node(path_node().path(fill_path).fill(fill_color));
    }
    let reveal_anim = animation("reveal khmer contours", reveal_dur, reveal_scene);

    let mut wave_scene = scene();
    for (ui, unit) in units.iter().enumerate() {
        let ui = ui as f32;
        let base = unit.base.clone();

        let path = Animated::new(move |t| {
            let secs = t * wave_total;
            let local = secs - ui * stagger;
            if local <= 0.0 {
                base.clone()
            } else if local < wave_dur {
                let bounce = if local / wave_dur < 0.5 {
                    (local / wave_dur) * 2.0
                } else {
                    2.0 - (local / wave_dur) * 2.0
                };
                base.clone().translate(0.0, -18.0 * bounce)
            } else {
                base.clone()
            }
        });
        wave_scene = wave_scene.node(path_node().path(path).fill(Color::WHITE));
    }
    let wave_anim = animation("wave khmer contours", wave_total, wave_scene);

    let playable = sequence("KhmerFadeWave", [reveal_anim, wave_anim]);

    let output = std::path::Path::new("results/khmer-fade-wave.mp4");
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let cfg = ExportConfig::new(30.0, viewport).crf(18);
    println!("Exporting {} ...", output.display());
    match export_mp4(&playable, &cfg, output) {
        Ok(()) => println!("Written {}", output.display()),
        Err(e) => eprintln!("Export failed: {e}"),
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
