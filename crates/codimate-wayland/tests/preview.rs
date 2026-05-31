use codimate_animation::animation;
use codimate_core::{circle, scene};
use codimate_layout::Viewport;
use codimate_wayland::{preview_frames, PreviewConfig};

#[test]
fn preview_frames_sample_playable_by_elapsed_time() {
    let playable = animation("demo", 1.0, scene().node(circle().radius(10.0)));

    let frames = preview_frames(
        &playable,
        PreviewConfig::new(2.0, Viewport::new(800.0, 600.0)),
    );

    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0].elapsed_seconds, 0.0);
    assert_eq!(frames[1].elapsed_seconds, 0.5);
    assert_eq!(frames[2].elapsed_seconds, 1.0);
    assert_eq!(frames[0].name, "demo");
    assert_eq!(frames[0].viewport, Viewport::new(800.0, 600.0));
    assert_eq!(frames[0].commands.len(), 1);
}
