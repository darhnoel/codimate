//! Render-pipeline test: rasterize a frame into pixels (tiny-skia backend).

use codimate_core::{circle_path, Color, Segment, TextAlign, Vec2};
use codimate_fonts::FontRegistry;
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
fn rasterize_japanese_text_uses_real_glyphs_not_missing_boxes() {
    let japanese_frame = RenderFrame {
        name: "japanese-text-test".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(360.0, 96.0),
        commands: vec![RenderCommand::Text {
            x: 20.0,
            y: 58.0,
            text: "私は猫が好きです".to_string(),
            font_size: 32.0,
            fill: Color::RED,
            align: TextAlign::Left,
        }],
    };
    let missing_frame = RenderFrame {
        name: "missing-text-test".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(360.0, 96.0),
        commands: vec![RenderCommand::Text {
            x: 20.0,
            y: 58.0,
            text: "\u{E000}\u{E000}\u{E000}\u{E000}\u{E000}\u{E000}\u{E000}\u{E000}".to_string(),
            font_size: 32.0,
            fill: Color::RED,
            align: TextAlign::Left,
        }],
    };

    let japanese = rasterize(&japanese_frame);
    let missing = rasterize(&missing_frame);

    let has_ink = (0..japanese.height)
        .any(|y| (0..japanese.width).any(|x| japanese.pixel(x, y) != (0, 0, 0, 255)));
    assert!(
        has_ink,
        "Japanese text rasterization produced no visible output"
    );
    assert_ne!(
        japanese.rgba, missing.rgba,
        "Japanese text rendered like repeated missing-glyph boxes"
    );
}

#[test]
fn rasterize_khmer_text_matches_harfbuzz_shaped_paths() {
    let text = "គន្លឹះ";
    let x = 20.0;
    let y = 64.0;
    let font_size = 38.0;
    let font_id = FontRegistry::global().char_font('គ');
    let block = codimate_glyph::shape(text, font_id, font_size, Color::RED)
        .expect("Khmer text should shape through HarfBuzz");

    let text_frame = RenderFrame {
        name: "khmer-text-test".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(220.0, 96.0),
        commands: vec![RenderCommand::Text {
            x,
            y,
            text: text.to_string(),
            font_size,
            fill: Color::RED,
            align: TextAlign::Left,
        }],
    };

    let path_commands = block
        .glyphs
        .iter()
        .map(|glyph| {
            let resolved = glyph.resolve(0.0);
            RenderCommand::Path {
                segments: resolved
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.translate(x, y))
                    .collect(),
                closed: resolved.path.closed,
                fill: resolved.fill,
                stroke_width: resolved.stroke_width,
                stroke_color: resolved.stroke_color,
            }
        })
        .collect();

    let shaped_frame = RenderFrame {
        name: "khmer-shaped-path-test".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(220.0, 96.0),
        commands: path_commands,
    };

    assert_eq!(
        rasterize(&text_frame).rgba,
        rasterize(&shaped_frame).rgba,
        "Khmer Text commands should use shaped glyph outlines"
    );
}

#[test]
fn rasterize_khmer_text_with_ascii_number_uses_font_runs() {
    let khmer = "គន្លឹះ ";
    let number = "8";
    let text = format!("{khmer}{number}");
    let x = 20.0;
    let y = 64.0;
    let font_size = 38.0;

    let khmer_block = codimate_glyph::shape(
        khmer,
        FontRegistry::global().char_font('គ'),
        font_size,
        Color::RED,
    )
    .expect("Khmer run should shape");
    let number_block = codimate_glyph::shape(
        number,
        FontRegistry::global().char_font('8'),
        font_size,
        Color::RED,
    )
    .expect("ASCII number run should shape");

    let text_frame = RenderFrame {
        name: "khmer-number-text-test".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(260.0, 96.0),
        commands: vec![RenderCommand::Text {
            x,
            y,
            text,
            font_size,
            fill: Color::RED,
            align: TextAlign::Left,
        }],
    };

    let path_commands = khmer_block
        .glyphs
        .iter()
        .map(|glyph| (glyph, x))
        .chain(
            number_block
                .glyphs
                .iter()
                .map(|glyph| (glyph, x + khmer_block.width)),
        )
        .map(|(glyph, run_x)| {
            let resolved = glyph.resolve(0.0);
            RenderCommand::Path {
                segments: resolved
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.translate(run_x, y))
                    .collect(),
                closed: resolved.path.closed,
                fill: resolved.fill,
                stroke_width: resolved.stroke_width,
                stroke_color: resolved.stroke_color,
            }
        })
        .collect();

    let shaped_frame = RenderFrame {
        name: "khmer-number-shaped-path-test".to_string(),
        elapsed_seconds: 0.0,
        viewport: Viewport::new(260.0, 96.0),
        commands: path_commands,
    };

    assert_eq!(
        rasterize(&text_frame).rgba,
        rasterize(&shaped_frame).rgba,
        "Khmer plus ASCII numbers should render as separate shaped font runs"
    );
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
