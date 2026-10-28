//! Manim Community default color space.
//!
//! These constants mirror the named colors from Manim's `manim_colors`
//! reference, represented as normalized RGBA values for Codimate.

use crate::Color;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

/// Manim's black, #000000.
pub const BLACK: Color = rgb(0x00, 0x00, 0x00);
/// Manim's white, #FFFFFF.
pub const WHITE: Color = rgb(0xFF, 0xFF, 0xFF);
/// Manim's darkest grey, #222222.
pub const DARKER_GRAY: Color = rgb(0x22, 0x22, 0x22);
/// Manim's dark grey, #444444.
pub const DARK_GRAY: Color = rgb(0x44, 0x44, 0x44);
/// Manim's grey, #888888.
pub const GRAY: Color = rgb(0x88, 0x88, 0x88);
/// Manim's light grey, #BBBBBB.
pub const LIGHT_GRAY: Color = rgb(0xBB, 0xBB, 0xBB);
/// Manim's blue, #58C4DD.
pub const BLUE: Color = rgb(0x58, 0xC4, 0xDD);
/// Manim's darker blue, #236B8E.
pub const BLUE_E: Color = rgb(0x23, 0x6B, 0x8E);
/// Manim's green, #83C167.
pub const GREEN: Color = rgb(0x83, 0xC1, 0x67);
/// Manim's darker green, #699C52.
pub const GREEN_E: Color = rgb(0x69, 0x9C, 0x52);
/// Manim's yellow, #F7D96F.
pub const YELLOW: Color = rgb(0xF7, 0xD9, 0x6F);
/// Manim's gold, #F0AC5F.
pub const GOLD: Color = rgb(0xF0, 0xAC, 0x5F);
/// Manim's orange, #FF862F.
pub const ORANGE: Color = rgb(0xFF, 0x86, 0x2F);
/// Manim's purple, #9A72AC.
pub const PURPLE: Color = rgb(0x9A, 0x72, 0xAC);
/// Manim's red, #FC6255.
pub const RED: Color = rgb(0xFC, 0x62, 0x55);
/// Manim's darker red, #CF5044.
pub const RED_E: Color = rgb(0xCF, 0x50, 0x44);
/// Manim's teal, #5CD0B3.
pub const TEAL: Color = rgb(0x5C, 0xD0, 0xB3);

/// `color` with its alpha set to `alpha`: 0 is clear, 1 is solid.
#[must_use]
pub const fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}
