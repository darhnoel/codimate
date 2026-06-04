use codimate_example_demo::create;
use codimate_export::{export_frames, export_mp4, ExportConfig};

fn main() {
    let (demo, viewport) = create();
    let export_cfg = ExportConfig::new(30.0, viewport);
    let export = export_frames(&demo, export_cfg);

    println!("{} export frames", export.len());

    std::fs::create_dir_all("results").ok();
    println!("Exporting results/demo.mp4 …");
    match export_mp4(&demo, &export_cfg, "results/demo.mp4") {
        Ok(()) => println!("Written results/demo.mp4"),
        Err(e) => eprintln!("mp4 export skipped: {e}"),
    }
}
