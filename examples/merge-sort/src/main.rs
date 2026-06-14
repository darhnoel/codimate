use codimate::ExportConfig;
use codimate::Viewport;
use codimate_example_merge_sort::{
    explain, merge_sort_algorithm, merge_sort_motion, merge_sort_view, MergeSort, MergeSortTiming,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderPreset {
    Standard,
    Hd60,
}

fn main() {
    let preset = render_preset();

    let explanation = explain("Merge Sort")
        .state(MergeSort::new([38, 27, 43, 3, 9, 82, 10, 15]))
        .view(merge_sort_view)
        .algorithm(merge_sort_algorithm)
        .motion(merge_sort_motion)
        .timing(MergeSortTiming::default());

    match preset {
        RenderPreset::Standard => explanation.render("results/merge-sort.mp4"),
        RenderPreset::Hd60 => {
            explanation.render_with("results/merge-sort-1080p60.mp4", |viewport| {
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
                println!("usage: cargo run -p codimate-example-merge-sort -- [--1080p60]");
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown option: {other}");
                eprintln!("usage: cargo run -p codimate-example-merge-sort -- [--1080p60]");
                std::process::exit(2);
            }
        }
    }
    preset
}
