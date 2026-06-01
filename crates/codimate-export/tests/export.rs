use codimate_animation::animation;
use codimate_core::{circle, scene, Color};
use codimate_export::{export_frames, playable_frames, write_raw_frames, ExportConfig};
use codimate_layout::Viewport;

#[test]
fn export_frames_plan_indexed_samples() {
    let playable = animation("demo", 1.0, scene().node(circle().radius(10.0)));

    let frames = export_frames(
        &playable,
        ExportConfig::new(2.0, Viewport::new(800.0, 600.0)),
    );

    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0].index, 0);
    assert_eq!(frames[1].frame.elapsed_seconds, 0.5);
    assert_eq!(frames[2].index, 2);
    assert_eq!(frames[2].frame.name, "demo");
    assert_eq!(frames[2].frame.viewport, Viewport::new(800.0, 600.0));
    assert_eq!(frames[2].frame.commands.len(), 1);
}

#[test]
fn playable_frames_yields_correct_count_and_timestamps() {
    let playable = animation("test", 2.0, scene().node(circle().radius(5.0)));

    // 2 second duration, 4 fps -> 9 frames (0.0, 0.25, ..., 2.0)
    let frames: Vec<_> =
        playable_frames(&playable, ExportConfig::new(4.0, Viewport::new(10.0, 10.0))).collect();

    assert_eq!(frames.len(), 9);
    for (i, frame) in frames.iter().enumerate() {
        let expected = (i as f32) / 4.0;
        assert!(
            (frame.elapsed_seconds - expected).abs() < 1e-6,
            "frame {} expected elapsed {:.4}, got {:.4}",
            i,
            expected,
            frame.elapsed_seconds,
        );
    }
}

#[test]
fn playable_frames_includes_final_frame_exactly_at_duration() {
    let playable = animation("test", 1.0, scene().node(circle().radius(5.0)));

    let frames: Vec<_> =
        playable_frames(&playable, ExportConfig::new(3.0, Viewport::new(10.0, 10.0))).collect();

    let last = frames.last().unwrap();
    assert!((last.elapsed_seconds - 1.0).abs() < 1e-6);
}

#[test]
fn write_raw_frames_writes_correct_rgba_byte_count() {
    let playable = animation("demo", 1.0, scene().node(circle().radius(5.0)));
    let viewport = Viewport::new(4.0, 4.0);
    let config = ExportConfig::new(2.0, viewport);

    let mut buf = Vec::new();
    write_raw_frames(playable_frames(&playable, config), &mut buf).unwrap();

    // 3 frames * 4*4 pixels * 4 bytes = 192 bytes
    assert_eq!(buf.len(), 3 * 4 * 4 * 4);
}

#[test]
fn write_raw_frames_counts_correctly_with_multiple_frames() {
    use codimate_render::RenderCommand;

    let commands = vec![RenderCommand::Rect {
        x: 0.0,
        y: 0.0,
        width: 2.0,
        height: 2.0,
        fill: Color::RED,
    }];
    let frame = codimate_render::RenderFrame {
        name: "test".into(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(2.0, 2.0),
        commands,
    };

    let mut buf = Vec::new();
    write_raw_frames(vec![frame; 5], &mut buf).unwrap();

    // 5 frames * 2*2*4 = 80 bytes
    assert_eq!(buf.len(), 80);
}

#[test]
fn export_mp4_produces_valid_mp4() {
    let playable = animation(
        "demo",
        0.5,
        scene().node(circle().radius(5.0).fill(Color::RED)),
    );
    let config = ExportConfig::new(10.0, Viewport::new(32.0, 32.0));

    let path = "/tmp/codimate-test-export.mp4";
    let result = codimate_export::export_mp4(&playable, &config, path);
    let exists = std::path::Path::new(path).exists();
    if exists {
        let _ = std::fs::remove_file(path);
    }

    // Accept either success or "encoder not found" (ffmpeg may not be on PATH
    // in all environments); anything else is a genuine failure.
    match result {
        Ok(()) => assert!(exists),
        Err(codimate_export::ExportError::EncoderNotFound) => {
            eprintln!("note: ffmpeg not found, skipping mp4 verification");
        }
        Err(other) => panic!("unexpected export error: {other}"),
    }
    // Clean up if the file somehow exists but result failed
    if exists {
        let _ = std::fs::remove_file(path);
    }
}
