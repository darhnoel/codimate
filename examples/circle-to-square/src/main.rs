use codimate_example_circle_to_square::create;
use codimate_export::{export_frames, export_mp4, ExportConfig};
use codimate_layout::Viewport;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderPreset {
    Standard,
    Hd60,
}

fn main() {
    let preset = render_preset();
    let (playable, viewport) = create();
    let (output, export_cfg) = match preset {
        RenderPreset::Standard => (
            "results/circle-to-square.mp4",
            ExportConfig::new(30.0, viewport).crf(12),
        ),
        RenderPreset::Hd60 => (
            "results/circle-to-square-1080p60.mp4",
            ExportConfig::new(60.0, viewport)
                .output_viewport(Viewport::new(1920.0, 1080.0))
                .crf(12),
        ),
    };
    let frames = export_frames(&playable, export_cfg);

    println!("{} export frames", frames.len());

    std::fs::create_dir_all("results").ok();
    println!("Exporting {output} ...");
    match export_mp4(&playable, &export_cfg, output) {
        Ok(()) => println!("Written {output}"),
        Err(e) => eprintln!("mp4 export skipped: {e}"),
    }
}

fn render_preset() -> RenderPreset {
    let mut preset = RenderPreset::Standard;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--1080p60" | "--1080p" | "--hd60" => preset = RenderPreset::Hd60,
            "--help" | "-h" => {
                println!("usage: cargo run -p codimate-example-circle-to-square -- [--1080p60]");
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown option: {other}");
                eprintln!("usage: cargo run -p codimate-example-circle-to-square -- [--1080p60]");
                std::process::exit(2);
            }
        }
    }
    preset
}
