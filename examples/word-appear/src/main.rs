use codimate::{export_mp4, ExportConfig};
use codimate_example_word_appear::{create, create_hd, create_scene, create_scene_hd};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderPreset {
    Standard,
    Hd60,
}

fn main() {
    let (preset, scene_filter) = parse_args();

    let (play, viewport) = match (preset, scene_filter) {
        (RenderPreset::Hd60, Some(name)) => create_scene_hd(name),
        (RenderPreset::Hd60, None) => create_hd(),
        (RenderPreset::Standard, Some(name)) => create_scene(name),
        (RenderPreset::Standard, None) => create(),
    };

    let (output, cfg) = match preset {
        RenderPreset::Standard => {
            let filename = match scene_filter {
                Some(name) => format!("results/word-appear-{name}.mp4"),
                None => "results/word-appear.mp4".to_string(),
            };
            (filename, ExportConfig::new(30.0, viewport).crf(10))
        }
        RenderPreset::Hd60 => {
            let suffix = match scene_filter {
                Some(name) => format!("-{name}"),
                None => String::new(),
            };
            // Scene already renders at 1920x1080 natively — no output_viewport needed
            (
                format!("results/word-appear{suffix}-1080p60.mp4"),
                ExportConfig::new(60.0, viewport).crf(12),
            )
        }
    };

    std::fs::create_dir_all("results").ok();
    println!("Exporting {output} …");
    match export_mp4(&play, &cfg, &output) {
        Ok(()) => println!("Written {output}"),
        Err(e) => eprintln!("mp4 export skipped: {e}"),
    }
}

fn parse_args() -> (RenderPreset, Option<&'static str>) {
    let mut preset = RenderPreset::Standard;
    let mut scene_filter: Option<&'static str> = None;
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--1080p60" | "--1080p" | "--hd60" => preset = RenderPreset::Hd60,
            "--scene" => {
                i += 1;
                if i < args.len() {
                    scene_filter = Some(Box::leak(args[i].clone().into_boxed_str()));
                }
            }
            "--help" | "-h" => {
                println!(
                    "usage: cargo run -p codimate-example-word-appear -- [--1080p60] [--scene <name>]"
                );
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown option: {other}");
                eprintln!(
                    "usage: cargo run -p codimate-example-word-appear -- [--1080p60] [--scene <name>]"
                );
                std::process::exit(2);
            }
        }
        i += 1;
    }
    (preset, scene_filter)
}
