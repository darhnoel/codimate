use codimate::{export_mp4, ExportConfig};
use codimate_example_box_arrow::create;

fn main() {
    let (play, viewport) = create();
    let cfg = ExportConfig::new(30.0, viewport).crf(12);

    std::fs::create_dir_all("results").ok();
    println!("Exporting results/box-arrow.mp4 ...");
    match export_mp4(&play, &cfg, "results/box-arrow.mp4") {
        Ok(()) => println!("Written results/box-arrow.mp4"),
        Err(e) => eprintln!("mp4 export skipped: {e}"),
    }
}
