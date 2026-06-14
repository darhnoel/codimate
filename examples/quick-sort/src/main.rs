use codimate::ExportConfig;
use codimate::Viewport;
use codimate_example_quick_sort::{
    explain, quick_sort_algorithm, quick_sort_motion, quick_sort_view, QuickSort, QuickSortTiming,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderPreset {
    Standard,
    Hd60,
}

fn main() {
    let preset = render_preset();

    let explanation = explain("Quick Sort")
        .state(QuickSort::new([38, 27, 43, 3, 9, 82, 10, 15]))
        .view(quick_sort_view)
        .algorithm(quick_sort_algorithm)
        .motion(quick_sort_motion)
        .timing(QuickSortTiming::default());

    match preset {
        RenderPreset::Standard => explanation.render("results/quick-sort.mp4"),
        RenderPreset::Hd60 => {
            explanation.render_with("results/quick-sort-1080p60.mp4", |viewport| {
                ExportConfig::new(60.0, viewport)
                    .output_viewport(Viewport::new(1920.0, 1080.0))
                    .crf(12)
            })
        }
    }
}

fn render_preset() -> RenderPreset {
    let mut preset = RenderPreset::Standard;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--1080p60" | "--1080p" | "--hd60" => preset = RenderPreset::Hd60,
            "--help" | "-h" => {
                println!("usage: cargo run -p codimate-example-quick-sort -- [--1080p60]");
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown option: {other}");
                eprintln!("usage: cargo run -p codimate-example-quick-sort -- [--1080p60]");
                std::process::exit(2);
            }
        }
    }
    preset
}
