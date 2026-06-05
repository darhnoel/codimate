//! Render-pipeline test: rasterize a frame into pixels (tiny-skia backend).

use codimate_core::{circle_path, Color, Segment, TextAlign, Vec2};
use codimate_layout::Viewport;
use codimate_render::{rasterize, RenderCommand, RenderFrame};

fn red_circle_frame() -> RenderFrame {
    RenderFrame {
        name: "test".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(640.0, 480.0),
        commands: vec![RenderCommand::Circle {
            x: 320.0,
            y: 240.0,
            radius: 100.0,
            fill: Color::RED,
        }],
    }
}

/// Golden pixel test: a red circle paints red at its center; the far corner
/// keeps the opaque-black background. Both points are far from any AA edge, so
/// the bytes are exact.
#[test]
fn rasterize_paints_circle_over_black_background() {
    let img = rasterize(&red_circle_frame());

    assert_eq!(img.width, 640);
    assert_eq!(img.height, 480);
    assert_eq!(img.pixel(320, 240), (255, 0, 0, 255)); // center is red
    assert_eq!(img.pixel(5, 5), (0, 0, 0, 255)); // corner is black background
}

#[test]
fn rasterize_path_fills_rect_path() {
    let frame = RenderFrame {
        name: "path-test".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(10.0, 10.0),
        commands: vec![RenderCommand::Path {
            segments: vec![
                Segment::Line(Vec2::new(1.0, 1.0), Vec2::new(9.0, 1.0)),
                Segment::Line(Vec2::new(9.0, 1.0), Vec2::new(9.0, 9.0)),
                Segment::Line(Vec2::new(9.0, 9.0), Vec2::new(1.0, 9.0)),
                Segment::Line(Vec2::new(1.0, 9.0), Vec2::new(1.0, 1.0)),
            ],
            closed: true,
            fill: Color::RED,
            stroke_width: 0.0,
            stroke_color: Color::RED,
        }],
    };

    let img = rasterize(&frame);

    // Inside the rect (center) is red
    assert_eq!(img.pixel(5, 5), (255, 0, 0, 255));
    // Outside (corner) is black
    assert_eq!(img.pixel(0, 0), (0, 0, 0, 255));
}

#[test]
fn rasterize_text_draws_glyphs() {
    let frame = RenderFrame {
        name: "text-test".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(64.0, 64.0),
        commands: vec![RenderCommand::Text {
            x: 10.0,
            y: 32.0,
            text: "A".to_string(),
            font_size: 24.0,
            fill: Color::RED,
            align: TextAlign::Left,
        }],
    };

    let img = rasterize(&frame);

    // Background corner stays black
    assert_eq!(img.pixel(0, 0), (0, 0, 0, 255));

    // At least one non-zero pixel means something was drawn
    let has_ink = (0..64).any(|y| (0..64).any(|x| img.pixel(x, y) != (0, 0, 0, 255)));
    assert!(has_ink, "text rasterization produced no visible output");
}

#[test]
fn rasterize_path_fill_and_stroke_produce_distinct_regions() {
    // Filled red rect (4,4)-(20,20) with white stroke width=4.
    // Stroke band: left edge covers x=2..6. Fill covers x=4..20.
    //   x=3,y=8 → stroke only (inside stroke band, outside fill) → white
    //   x=7,y=8 → fill only   (inside fill, outside stroke band) → red
    let frame = RenderFrame {
        name: "stroke-test".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(24.0, 24.0),
        commands: vec![RenderCommand::Path {
            segments: vec![
                Segment::Line(Vec2::new(4.0, 4.0), Vec2::new(20.0, 4.0)),
                Segment::Line(Vec2::new(20.0, 4.0), Vec2::new(20.0, 20.0)),
                Segment::Line(Vec2::new(20.0, 20.0), Vec2::new(4.0, 20.0)),
                Segment::Line(Vec2::new(4.0, 20.0), Vec2::new(4.0, 4.0)),
            ],
            closed: true,
            fill: Color::RED,
            stroke_width: 4.0,
            stroke_color: Color::WHITE,
        }],
    };

    let img = rasterize(&frame);

    // Center is red (fill)
    assert_eq!(img.pixel(12, 12), (255, 0, 0, 255));

    // Stroke-only region: inside stroke band, outside fill
    let pix = img.pixel(3, 8);
    let expected: (u8, u8, u8, u8) = (255, 255, 255, 255);
    assert_eq!(
        pix, expected,
        "stroke-only pixel expected white, got {pix:?}"
    );

    // Fill-only region: inside fill, outside stroke band
    assert_eq!(img.pixel(7, 8), (255, 0, 0, 255));

    // Far corner stays black
    assert_eq!(img.pixel(0, 0), (0, 0, 0, 255));
}

#[test]
fn rasterize_path_cubic_circle() {
    let circle = circle_path(32.0, 32.0, 20.0);
    let frame = RenderFrame {
        name: "circle-path".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(64.0, 64.0),
        commands: vec![RenderCommand::Path {
            segments: circle.segments.clone(),
            closed: circle.closed,
            fill: Color::RED,
            stroke_width: 0.0,
            stroke_color: Color::RED,
        }],
    };

    let img = rasterize(&frame);

    // Centre of the circle is red
    assert_eq!(img.pixel(32, 32), (255, 0, 0, 255));
    // Far corner is black (background)
    assert_eq!(img.pixel(0, 0), (0, 0, 0, 255));
}
