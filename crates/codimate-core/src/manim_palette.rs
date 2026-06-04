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

pub const BLACK: Color = rgb(0x00, 0x00, 0x00);
pub const WHITE: Color = rgb(0xFF, 0xFF, 0xFF);
pub const DARKER_GRAY: Color = rgb(0x22, 0x22, 0x22);
pub const DARK_GRAY: Color = rgb(0x44, 0x44, 0x44);
pub const GRAY: Color = rgb(0x88, 0x88, 0x88);
pub const LIGHT_GRAY: Color = rgb(0xBB, 0xBB, 0xBB);
pub const BLUE: Color = rgb(0x58, 0xC4, 0xDD);
pub const BLUE_E: Color = rgb(0x23, 0x6B, 0x8E);
pub const GREEN: Color = rgb(0x83, 0xC1, 0x67);
pub const GREEN_E: Color = rgb(0x69, 0x9C, 0x52);
pub const YELLOW: Color = rgb(0xF7, 0xD9, 0x6F);
pub const GOLD: Color = rgb(0xF0, 0xAC, 0x5F);
pub const ORANGE: Color = rgb(0xFF, 0x86, 0x2F);
pub const PURPLE: Color = rgb(0x9A, 0x72, 0xAC);
pub const RED: Color = rgb(0xFC, 0x62, 0x55);
pub const RED_E: Color = rgb(0xCF, 0x50, 0x44);
pub const TEAL: Color = rgb(0x5C, 0xD0, 0xB3);

pub const fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}
