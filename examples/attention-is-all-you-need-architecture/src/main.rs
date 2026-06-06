use codimate_example_attention_is_all_you_need_architecture::{
    explain, transformer_architecture_algorithm, transformer_architecture_motion,
    transformer_architecture_view, TransformerArchitecture, TransformerArchitectureTiming,
};
use codimate_export::{export_frame_jpeg, ExportConfig};
use codimate_layout::Viewport;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderMode {
    Standard,
    Hd60,
    DebugJpgScenes { start: usize, end: usize },
}

fn main() {
    let mode = render_mode();
    let timing = TransformerArchitectureTiming::default();

    let explanation = explain("Attention Is All You Need Architecture")
        .state(TransformerArchitecture::new())
        .view(transformer_architecture_view)
        .algorithm(transformer_architecture_algorithm)
        .motion(transformer_architecture_motion)
        .timing(timing);

    match mode {
        RenderMode::Standard => {
            explanation.render("results/attention-is-all-you-need-architecture.mp4")
        }
        RenderMode::Hd60 => explanation.render_with(
            "results/attention-is-all-you-need-architecture-1080p60.mp4",
            |viewport| {
                ExportConfig::new(60.0, viewport)
                    .output_viewport(Viewport::new(1920.0, 1080.0))
                    .crf(12)
            },
        ),
        RenderMode::DebugJpgScenes { start, end } => {
            let (play, viewport) = explanation.build();
            export_debug_scene_jpegs(
                play.as_ref(),
                viewport,
                timing,
                start,
                end,
                "results/attention-is-all-you-need-architecture-frames",
            );
        }
    }
}

fn render_mode() -> RenderMode {
    let mut mode = RenderMode::Standard;
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--1080p60" | "--1080p" | "--hd60" => mode = RenderMode::Hd60,
            "--debug-jpg-scenes" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    match parse_scene_range(&args[i]) {
                        Some((start, end)) => mode = RenderMode::DebugJpgScenes { start, end },
                        None => usage_and_exit(2, "scene range must look like 1-3 or all"),
                    }
                } else {
                    mode = RenderMode::DebugJpgScenes {
                        start: 1,
                        end: scene_count(),
                    };
                }
            }
            "--help" | "-h" => {
                usage_and_exit(0, "");
            }
            other => {
                usage_and_exit(2, &format!("unknown option: {other}"));
            }
        }
        i += 1;
    }
    mode
}

fn parse_scene_range(input: &str) -> Option<(usize, usize)> {
    if input == "all" {
        return Some((1, scene_count()));
    }

    let (start, end) = input.split_once('-')?;
    let start = start.parse::<usize>().ok()?;
    let end = end.parse::<usize>().ok()?;
    if start == 0 || end < start {
        return None;
    }
    Some((start, end))
}

fn export_debug_scene_jpegs(
    play: &dyn codimate_animation::Playable,
    viewport: Viewport,
    timing: TransformerArchitectureTiming,
    start: usize,
    end: usize,
    output_dir: impl AsRef<Path>,
) {
    let durations = scene_durations(timing);
    let capped_end = end.min(durations.len());
    if start > durations.len() {
        eprintln!(
            "scene range starts at {start}, but this example has only {} scenes",
            durations.len()
        );
        std::process::exit(2);
    }

    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).unwrap_or_else(|e| {
        eprintln!("failed to create {}: {e}", output_dir.display());
        std::process::exit(1);
    });

    let mut cursor = 0.0;
    for (index, duration) in durations.iter().enumerate() {
        let scene_number = index + 1;
        let scene_start = cursor;
        let scene_end = cursor + duration;
        cursor = scene_end;

        if scene_number < start || scene_number > capped_end {
            continue;
        }

        let end_sample = if scene_number == durations.len() {
            scene_end
        } else {
            (scene_end - 0.001).max(scene_start)
        };

        for (label, seconds) in [("start", scene_start), ("end", end_sample)] {
            let path = output_dir.join(format!("scene-{scene_number:02}-{label}.jpg"));
            match export_frame_jpeg(play, seconds, viewport, &path, 92) {
                Ok(()) => println!("Written {}", path.display()),
                Err(e) => {
                    eprintln!("failed to write {}: {e}", path.display());
                    std::process::exit(1);
                }
            }
        }
    }
}

fn scene_durations(timing: TransformerArchitectureTiming) -> Vec<f32> {
    let mut durations = Vec::with_capacity(18);
    durations.extend([timing.short; 3]);
    durations.extend([timing.normal; 14]);
    durations.push(timing.final_reveal);
    durations
}

fn usage_and_exit(code: i32, message: &str) -> ! {
    if !message.is_empty() {
        eprintln!("{message}");
    }
    println!(
        "usage: cargo run -p codimate-example-attention-is-all-you-need-architecture -- [--1080p60 | --debug-jpg-scenes [START-END|all]]"
    );
    println!("examples:");
    println!("  --debug-jpg-scenes");
    println!("  --debug-jpg-scenes 1-3");
    println!("  --debug-jpg-scenes all");
    std::process::exit(code);
}

fn scene_count() -> usize {
    scene_durations(TransformerArchitectureTiming::default()).len()
}
