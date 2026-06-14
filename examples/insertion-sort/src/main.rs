use codimate::ExportConfig;
use codimate::Viewport;
use codimate_example_insertion_sort::{
    explain, insertion_sort_algorithm, insertion_sort_motion, insertion_sort_view, InsertionSort,
    InsertionSortTiming,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderPreset {
    Standard,
    Hd60,
}

fn main() {
    let preset = render_preset();

    let explanation = explain("ការតម្រៀបដោយបញ្ចូល")
        .state(InsertionSort::new([8, 3, 5, 1, 7, 4, 6, 2]))
        .view(insertion_sort_view)
        .algorithm(insertion_sort_algorithm)
        .motion(insertion_sort_motion)
        .timing(InsertionSortTiming::default());

    match preset {
        RenderPreset::Standard => explanation.render("results/insertion-sort.mp4"),
        RenderPreset::Hd60 => {
            explanation.render_with("results/insertion-sort-1080p60.mp4", |viewport| {
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
                println!("usage: cargo run -p codimate-example-insertion-sort -- [--1080p60]");
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown option: {other}");
                eprintln!("usage: cargo run -p codimate-example-insertion-sort -- [--1080p60]");
                std::process::exit(2);
            }
        }
    }
    preset
}
