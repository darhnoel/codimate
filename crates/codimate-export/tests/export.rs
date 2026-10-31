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
    let config = ExportConfig::new(10.0, Viewport::new(32.0, 20.0))
        .output_viewport(Viewport::new(64.0, 64.0));

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

#[test]
fn frames_of_a_long_film_land_on_the_instants_they_are_shown_at() {
    use codimate_export::FrameTimes;

    // Accumulating `1 / fps` in f32 drew frame i of this film up to 102 ms
    // away from i / 60, and made 36,008 frames of 36,001.
    let times: Vec<f32> = FrameTimes::new(600.0, 60.0).collect();
    assert_eq!(times.len(), 36_001);
    for (i, at) in times.iter().enumerate() {
        assert_eq!(*at, (i as f64 / 60.0) as f32, "frame {i}");
    }
    assert_eq!(*times.last().unwrap(), 600.0);
}

#[test]
fn a_duration_a_hair_over_a_whole_frame_gets_no_extra_frame() {
    use codimate_export::FrameTimes;

    // 1.1_f32 * 30 is 33.0000007: 33 frames after the first, not 34.
    let times: Vec<f32> = FrameTimes::new(1.1, 30.0).collect();
    assert_eq!(times.len(), 34);
    assert_eq!(*times.last().unwrap(), 1.1);
}

#[test]
fn frame_times_at_the_edges() {
    use codimate_export::FrameTimes;

    let all = |duration: f32, fps: f32| FrameTimes::new(duration, fps).collect::<Vec<_>>();
    assert_eq!(all(0.0, 30.0), vec![0.0]);
    assert_eq!(all(-1.0, 30.0), Vec::<f32>::new());
    assert_eq!(all(2.0, 0.0), vec![0.0, 2.0]);
    // Shorter than a frame: the start and the end, as before.
    assert_eq!(all(0.00001, 30.0), vec![0.0, 0.00001]);
    // Off the grid: the last frame still lands on the end.
    assert_eq!(all(0.5, 3.0), vec![0.0, 1.0 / 3.0, 0.5]);

    let mut times = FrameTimes::new(1.0, 4.0);
    assert_eq!(times.size_hint(), (5, Some(5)));
    times.next();
    assert_eq!(times.size_hint(), (4, Some(4)));
}

/// A unique path in the temporary folder, so tests running at once never share one.
fn temp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("codimate-{}-{name}", std::process::id()))
}

#[test]
fn a_finished_mp4_has_its_index_at_the_front_and_leaves_no_partial_file() {
    let playable = animation(
        "demo",
        0.5,
        scene().node(circle().radius(5.0).fill(Color::RED)),
    );
    let config = ExportConfig::new(10.0, Viewport::new(32.0, 20.0));
    let path = temp("faststart.mp4");

    match codimate_export::export_mp4(&playable, &config, &path) {
        Ok(()) => {}
        Err(codimate_export::ExportError::EncoderNotFound) => return,
        Err(other) => panic!("unexpected export error: {other}"),
    }
    let bytes = std::fs::read(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    // The top-level boxes, in order: `moov` must come before `mdat`.
    let mut boxes = Vec::new();
    let mut at = 0;
    while at + 8 <= bytes.len() {
        let size = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        boxes.push(String::from_utf8_lossy(&bytes[at + 4..at + 8]).to_string());
        if size < 8 {
            break;
        }
        at += size;
    }
    let position = |kind: &str| boxes.iter().position(|b| b == kind).unwrap();
    assert!(position("moov") < position("mdat"), "boxes: {boxes:?}");
    assert!(!path
        .with_file_name(format!(
            ".partial-{}",
            path.file_name().unwrap().to_string_lossy()
        ))
        .exists());
}

#[test]
fn a_failed_render_keeps_the_previous_video_and_leaves_no_partial_file() {
    let playable = animation("demo", 0.5, scene().node(circle().radius(5.0)));
    let config = ExportConfig::new(10.0, Viewport::new(32.0, 20.0));
    // ffmpeg cannot choose a container for this extension, so it fails.
    let path = temp("keep.not-a-container");
    std::fs::write(&path, b"the previous video").unwrap();

    let result = codimate_export::export_mp4(&playable, &config, &path);
    let kept = std::fs::read(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    match result {
        Err(codimate_export::ExportError::EncoderNotFound) => return,
        Err(codimate_export::ExportError::EncoderFailed(_)) => {}
        other => panic!("expected the encoder to fail, got {other:?}"),
    }
    assert_eq!(kept, b"the previous video");
    let partial = path.with_file_name(format!(
        ".partial-{}",
        path.file_name().unwrap().to_string_lossy()
    ));
    assert!(!partial.exists());
}
