use codimate_example_transformer::create;
use codimate_export::{export_mp4, ExportConfig};

fn main() {
    let (play, viewport) = create();
    let cfg = ExportConfig::new(30.0, viewport).crf(12);

    std::fs::create_dir_all("results").ok();
    println!("Exporting results/transformer.mp4 …");
    match export_mp4(&play, &cfg, "results/transformer.mp4") {
        Ok(()) => println!("Written results/transformer.mp4"),
        Err(e) => eprintln!("mp4 export skipped: {e}"),
    }
}
