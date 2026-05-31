use codimate_animation::{animation, sequence};
use codimate_core::{circle, rect, scene, tween, Color};
use codimate_export::{export_frames, ExportConfig};
use codimate_layout::Viewport;
use codimate_wayland::{preview_frames, PreviewConfig};

fn main() {
    let intro = animation(
        "intro",
        1.0,
        scene().node(
            circle()
                .x(tween(40.0, 240.0))
                .y(120.0)
                .radius(tween(16.0, 48.0))
                .fill(Color::RED),
        ),
    );

    let outro = animation(
        "outro",
        1.0,
        scene().node(
            rect()
                .x(80.0)
                .y(tween(180.0, 260.0))
                .width(220.0)
                .height(80.0)
                .fill(Color {
                    r: 0.1,
                    g: 0.4,
                    b: 1.0,
                    a: 1.0,
                }),
        ),
    );

    let demo = sequence("demo", [intro, outro]);
    let viewport = Viewport::new(800.0, 600.0);
    let preview = preview_frames(&demo, PreviewConfig::new(30.0, viewport));
    let export = export_frames(&demo, ExportConfig::new(30.0, viewport));

    println!(
        "{} preview frames, {} export frames",
        preview.len(),
        export.len()
    );
    println!(
        "first preview frame: {} render command(s)",
        preview[0].commands.len()
    );
}
