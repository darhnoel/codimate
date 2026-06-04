use codimate_core::{manim, Color};

fn hex(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

#[test]
fn manim_palette_matches_documented_hex_values() {
    assert_eq!(manim::BLACK, hex(0x00, 0x00, 0x00));
    assert_eq!(manim::WHITE, hex(0xFF, 0xFF, 0xFF));
    assert_eq!(manim::BLUE, hex(0x58, 0xC4, 0xDD));
    assert_eq!(manim::BLUE_E, hex(0x23, 0x6B, 0x8E));
    assert_eq!(manim::GREEN, hex(0x83, 0xC1, 0x67));
    assert_eq!(manim::YELLOW, hex(0xF7, 0xD9, 0x6F));
    assert_eq!(manim::RED, hex(0xFC, 0x62, 0x55));
    assert_eq!(manim::TEAL, hex(0x5C, 0xD0, 0xB3));
}

#[test]
fn manim_palette_supports_alpha_variants() {
    assert_eq!(
        manim::with_alpha(manim::BLUE, 0.6),
        Color {
            a: 0.6,
            ..manim::BLUE
        }
    );
}
