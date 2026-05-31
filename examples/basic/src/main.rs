use codimate_animation::{animation, sequence};
use codimate_core::{circle, path_node, circle_path, rect_path, rect, scene, tween, Color};
use codimate_export::{export_frames, export_mp4, ExportConfig};
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

    let morph = animation(
        "morph",
        1.0,
        scene().node(
            path_node()
                .path(tween(circle_path(400.0, 300.0, 80.0), rect_path(300.0, 200.0, 200.0, 200.0)))
                .fill(Color::RED),
        ),
    );

    let demo = sequence("demo", [intro, outro, morph]);
    let viewport = Viewport::new(800.0, 600.0);
    let export_cfg = ExportConfig::new(30.0, viewport);
    let preview = preview_frames(&demo, PreviewConfig::new(30.0, viewport));
    let export = export_frames(&demo, export_cfg);

    println!(
        "{} preview frames, {} export frames",
        preview.len(),
        export.len()
    );
    println!(
        "first preview frame: {} render command(s)",
        preview[0].commands.len()
    );

    println!("Exporting demo.mp4 …");
    match export_mp4(&demo, &export_cfg, "demo.mp4") {
        Ok(()) => println!("Written demo.mp4"),
        Err(e) => eprintln!("mp4 export skipped: {e}"),
    }
}
