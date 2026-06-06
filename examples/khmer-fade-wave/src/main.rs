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
        .map(|gn| {
            let resolved = gn.resolve(0.0);
            let base = resolved.path.translate(ox, oy);
            let contours = base.split_contours();
            GlyphPaths { base, contours }
        })
        .collect();

    let n = glyphs.len() as f32;
    let reveal_dur = 1.2;
    let wave_dur = 1.2;
    let stagger = 0.06;
    let wave_total = wave_dur + (n - 1.0) * stagger;
    let glyph_reveal_span = 1.0 / n.max(1.0);

    let mut reveal_scene = scene();
    for (gi, glyph) in glyphs.iter().enumerate() {
        let start = gi as f32 * glyph_reveal_span;
        let end = start + glyph_reveal_span;
        for path in &glyph.contours {
            let path = path.clone();
            let revealed = Animated::new(move |t| {
                let local = ((t - start) / (end - start)).clamp(0.0, 1.0);
                path.prefix(local)
            });
            reveal_scene = reveal_scene.node(
                path_node()
                    .path(revealed)
                    .stroke(2.0, Color::WHITE)
                    .fill(Color::TRANSPARENT),
            );
        }
    }
    let reveal_anim = animation("reveal khmer contours", reveal_dur, reveal_scene);

    let mut wave_scene = scene();
    for (gi, glyph) in glyphs.iter().enumerate() {
        let gi = gi as f32;
        let base = glyph.base.clone();

        let path = Animated::new(move |t| {
            let secs = t * wave_total;
            let local = secs - gi * stagger;
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
