use codimate_example_knapsack::{
    explain, knapsack_algorithm, knapsack_motion, knapsack_view, Knapsack, KnapsackTiming,
};
use codimate_export::ExportConfig;
use codimate_layout::Viewport;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderPreset {
    Standard,
    Hd60,
}

fn main() {
    let preset = render_preset();

    let explanation = explain("Knapsack")
        .state(Knapsack::new())
        .view(knapsack_view)
        .algorithm(knapsack_algorithm)
        .motion(knapsack_motion)
        .timing(KnapsackTiming::default());

    match preset {
        RenderPreset::Standard => explanation.render("results/knapsack.mp4"),
        RenderPreset::Hd60 => explanation.render_with("results/knapsack-1080p60.mp4", |viewport| {
            ExportConfig::new(60.0, viewport)
                .output_viewport(Viewport::new(1920.0, 1080.0))
                .crf(12)
        }),
    }
}

fn render_preset() -> RenderPreset {
    let mut preset = RenderPreset::Standard;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--1080p60" | "--1080p" | "--hd60" => preset = RenderPreset::Hd60,
            "--help" | "-h" => {
                println!("usage: cargo run -p codimate-example-knapsack -- [--1080p60]");
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown option: {other}");
                eprintln!("usage: cargo run -p codimate-example-knapsack -- [--1080p60]");
                std::process::exit(2);
            }
        }
    }
    preset
}
