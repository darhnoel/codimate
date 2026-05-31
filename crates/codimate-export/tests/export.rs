use codimate_animation::animation;
use codimate_core::{circle, scene};
use codimate_export::{export_frames, ExportConfig};

#[test]
fn export_frames_plan_indexed_samples() {
    let playable = animation("demo", 1.0, scene().node(circle().radius(10.0)));

    let frames = export_frames(&playable, ExportConfig::new(2.0));

    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0].index, 0);
    assert_eq!(frames[1].elapsed_seconds, 0.5);
    assert_eq!(frames[2].index, 2);
}
