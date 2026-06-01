use codimate_animation::animation;
use codimate_core::*;
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;

fn main() {
    let box_a = rect()
        .x(50.0)
        .y(150.0)
        .width(100.0)
        .height(60.0)
        .fill(Color {
            r: 0.2,
            g: 0.4,
            b: 0.9,
            a: 1.0,
        });

    let box_b = rect()
        .x(400.0)
        .y(150.0)
        .width(100.0)
        .height(60.0)
        .fill(Color {
            r: 0.1,
            g: 0.7,
            b: 0.3,
            a: 1.0,
        });

    let conn = connection(
        box_a.anchor(AnchorKind::Right),
        box_b.anchor(AnchorKind::Left),
    )
    .stroke(3.0, Color::WHITE)
    .arrow(12.0);

    let pulse = pulse_on(conn.clone(), tween(0.0, 1.0))
        .radius(6.0)
        .fill(Color::CYAN);

    let play = animation(
        "connection-pulse",
        2.0,
        scene().node(box_a).node(box_b).node(conn).node(pulse),
    );

    let viewport = Viewport::new(600.0, 360.0);
    let cfg = ExportConfig::new(30.0, viewport).crf(10);

    println!("Exporting connection-pulse.mp4 …");
    match export_mp4(&play, &cfg, "connection-pulse.mp4") {
        Ok(()) => println!("Written connection-pulse.mp4"),
        Err(e) => eprintln!("mp4 export skipped: {e}"),
    }
}
