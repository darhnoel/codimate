//! codimate-layout — Slot-based View layout helpers.
//!
//! A `Slot` is a View-only layout position with a top-left point and size.
//! It is not concept state, not a Scene Node, and not a runtime layout pass.
//! Use Slots in View code before reaching for raw coordinates.

use crate::Slot;
use codimate_core::{text, Color, IntoAnimated, Text};

// ---------------------------------------------------------------------------
// Text helpers
// ---------------------------------------------------------------------------

/// A `Text` node centered within the given slot, with `TextAlign::Center`.
pub fn centered_text(
    slot: &Slot,
    content: impl Into<String>,
    font_size: f32,
    fill: impl IntoAnimated<Color>,
) -> Text {
    let content = content.into();
    let center = slot.center();
    let cx = center.clone().map(|v| v.x);
    let cy = center.map(move |v| v.y + font_size * 0.34);
    text()
        .x(cx)
        .y(cy)
        .text(content)
        .font_size(font_size)
        .fill(fill)
        .align(codimate_core::TextAlign::Center)
}

/// A `Text` node whose top-left is positioned at the slot's top-left.
pub fn text_at(
    slot: &Slot,
    content: impl Into<String>,
    font_size: f32,
    fill: impl IntoAnimated<Color>,
) -> Text {
    let content = content.into();
    let origin = slot.origin.clone();
    let x = origin.clone().map(|v| v.x);
    let y = origin.map(|v| v.y);
    text()
        .x(x)
        .y(y)
        .text(content)
        .font_size(font_size)
        .fill(fill)
}
