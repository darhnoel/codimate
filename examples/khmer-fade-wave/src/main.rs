use codimate_animation::animation;
use codimate_core::*;
use codimate_export::{export_mp4, ExportConfig};
use codimate_fonts::FontRegistry;
use codimate_layout::Viewport;

fn main() {
    let viewport = Viewport::new(800.0, 600.0);

    // 1. Shape Khmer text into glyph paths
    let font_id = FontRegistry::global().char_font('ខ');
    let block = codimate_glyph::shape("ជំរាបសួរ", font_id, 72.0, Color::WHITE).unwrap();

    // 2. Center the block in the viewport
    let ox = (viewport.width - block.width) / 2.0;
    let oy = viewport.height / 2.0;

    // 3. Pre-compute rest & wave paths (glyph-by-glyph)
    let glyphs: Vec<(Path, Path)> = block
        .glyphs
        .iter()
        .map(|g| {
            let resolved = g.resolve(0.0);
            let rest = resolved.path.translate(ox, oy);
            let wave = rest.clone().translate(0.0, -18.0);
            (rest, wave)
        })
        .collect();

    // 4. Timing constants
    let fade_dur = 1.0;
    let wave_dur = 1.2;
    let stagger = 0.06;
    let total = fade_dur + wave_dur + (glyphs.len() as f32 - 1.0) * stagger;

    // 5. Scene: each glyph fades in, then waves with stagger
    let mut s = scene();
    for (i, (rest, wave)) in glyphs.iter().enumerate() {
        let i = i as f32;
        let rest = rest.clone();
        let wave = wave.clone();
        let wave_start = fade_dur + i * stagger;

        let path = Animated::new({
            let total = total;
            move |t| {
                let secs = t * total;
                if secs < wave_start {
                    rest.clone()
                } else if secs < wave_start + wave_dur {
                    let local = (secs - wave_start) / wave_dur;
                    let bounce = if local < 0.5 { local * 2.0 } else { 2.0 - local * 2.0 };
                    Path::lerp(rest.clone(), wave.clone(), bounce)
                } else {
                    rest.clone()
                }
            }
        });
        let fill = Animated::new(move |t| {
            let secs = t * total;
            if secs < fade_dur {
                Color::lerp(Color::TRANSPARENT, Color::WHITE, secs / fade_dur)
            } else {
                Color::WHITE
            }
        });
        s = s.node(path_node().path(path).fill(fill));
    }

    let playable = animation("KhmerFadeWave", total, s);

    // 6. Export
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
