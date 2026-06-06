use codimate_export::{export_mp4, ExportConfig};

fn main() {
    let (playable, viewport) = codimate_example_khmer_fade_wave::create();
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
