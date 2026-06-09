use codimate_example_logo_animation::{
    explain, logo_animation_algorithm, logo_animation_motion, logo_animation_view, LogoAnimation,
    LogoAnimationTiming,
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

    let explanation = explain("Logo Animation")
        .state(LogoAnimation::default())
        .view(logo_animation_view)
        .algorithm(logo_animation_algorithm)
        .motion(logo_animation_motion)
        .timing(LogoAnimationTiming::default());

    match preset {
        RenderPreset::Standard => explanation.render("results/logo-animation.mp4"),
        RenderPreset::Hd60 => {
            explanation.render_with("results/logo-animation-1080p60.mp4", |viewport| {
                let scale = (1920.0_f32 / viewport.width).min(1080.0 / viewport.height);
                ExportConfig::new(60.0, viewport)
                    .pixel_scale(scale)
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
                println!("usage: cargo run -p codimate-example-logo-animation -- [--1080p60]");
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown option: {other}");
                eprintln!("usage: cargo run -p codimate-example-logo-animation -- [--1080p60]");
                std::process::exit(2);
            }
        }
    }
    preset
}
