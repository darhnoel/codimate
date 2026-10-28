//! codimate-layout — Slot-based View layout helpers.
//!
//! A `Slot` is a View-only layout position with a top-left point and size.
//! It is not concept state, not a Scene Node, and not a runtime layout pass.
//! Use Slots in View code before reaching for raw coordinates.

use crate::Slot;
use codimate_core::{
    Animated, Color, IntoAnimated, Path, PathNode, SceneNode, Segment, Style, Vec2,
};

// ---------------------------------------------------------------------------
// Box builder — utility for rounded-rect PathNodes from Slots
// ---------------------------------------------------------------------------

/// Build a styled rounded-rect PathNode from a slot or animated center.
///
/// Created by [`box_in`] or [`box_at`].
pub struct BoxBuilder {
    center: Animated<Vec2>,
    size: Vec2,
    radius: Animated<f32>,
    fill: Animated<Color>,
    stroke_width: Animated<f32>,
    stroke_color: Animated<Color>,
}

/// Construct a `BoxBuilder` from a Slot (uses the slot's full bounds).
#[must_use]
pub fn box_in(slot: &Slot) -> BoxBuilder {
    box_at(slot.center(), Vec2::new(slot.w, slot.h))
}

/// Construct a `BoxBuilder` from an animated center and fixed size.
pub fn box_at(center: impl IntoAnimated<Vec2>, size: Vec2) -> BoxBuilder {
    BoxBuilder {
        center: center.into_animated(),
        size,
        radius: 0.0.into_animated(),
        fill: Color::WHITE.into_animated(),
        stroke_width: 0.0.into_animated(),
        stroke_color: Color::WHITE.into_animated(),
    }
}

impl BoxBuilder {
    /// Sets the corner radius: a number, or an animated value.
    #[must_use]
    pub fn radius(mut self, r: impl IntoAnimated<f32>) -> Self {
        self.radius = r.into_animated();
        self
    }

    /// Sets the whole style at once, instead of the fill and stroke separately.
    #[must_use]
    pub fn style(mut self, s: impl IntoAnimated<Style>) -> Self {
        let s = s.into_animated();
        let fill = s.clone();
        let sw = s.clone();
        self.fill = Animated::new(move |t| fill.resolve(t).fill);
        self.stroke_width = Animated::new(move |t| sw.resolve(t).stroke_width);
        self.stroke_color = Animated::new(move |t| s.resolve(t).stroke_color);
        self
    }

    /// Sets the fill colour: a colour, or an animated colour.
    #[must_use]
    pub fn fill(mut self, c: impl IntoAnimated<Color>) -> Self {
        self.fill = c.into_animated();
        self
    }

    /// Sets the stroke width and colour. Either can be a plain value or animated.
    #[must_use]
    pub fn stroke(
        mut self,
        width: impl IntoAnimated<f32>,
        color: impl IntoAnimated<Color>,
    ) -> Self {
        self.stroke_width = width.into_animated();
        self.stroke_color = color.into_animated();
        self
    }

    /// Build the final `PathNode`.
    #[must_use]
    pub fn into_node(self) -> PathNode {
        let center = self.center;
        let w = self.size.x;
        let h = self.size.y;
        let radius = self.radius;
        let path = Animated::new(move |t| {
            let c = center.resolve(t);
            let r = radius.resolve(t);
            rounded_rect_path(c.x - w / 2.0, c.y - h / 2.0, w, h, r)
        });
        PathNode::new()
            .path(path)
            .fill(self.fill)
            .stroke(self.stroke_width, self.stroke_color)
    }
}

impl From<BoxBuilder> for SceneNode {
    fn from(b: BoxBuilder) -> Self {
        SceneNode::Path(b.into_node())
    }
}

/// Build a rounded-rect path. Always produces 8 segments (4 lines + 4 corner cubics),
/// even at `r = 0` where corner cubics become degenerate.
fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let r = r.max(0.0);
    let k = r * 0.552_284_9;
    Path {
        segments: vec![
            Segment::Line(Vec2::new(x + r, y), Vec2::new(x + w - r, y)),
            Segment::Cubic(
                Vec2::new(x + w - r, y),
                Vec2::new(x + w - r + k, y),
                Vec2::new(x + w, y + r - k),
                Vec2::new(x + w, y + r),
            ),
            Segment::Line(Vec2::new(x + w, y + r), Vec2::new(x + w, y + h - r)),
            Segment::Cubic(
                Vec2::new(x + w, y + h - r),
                Vec2::new(x + w, y + h - r + k),
                Vec2::new(x + w - r + k, y + h),
                Vec2::new(x + w - r, y + h),
            ),
            Segment::Line(Vec2::new(x + w - r, y + h), Vec2::new(x + r, y + h)),
            Segment::Cubic(
                Vec2::new(x + r, y + h),
                Vec2::new(x + r - k, y + h),
                Vec2::new(x, y + h - r + k),
                Vec2::new(x, y + h - r),
            ),
            Segment::Line(Vec2::new(x, y + h - r), Vec2::new(x, y + r)),
            Segment::Cubic(
                Vec2::new(x, y + r),
                Vec2::new(x, y + r - k),
                Vec2::new(x + r - k, y),
                Vec2::new(x + r, y),
            ),
        ],
        closed: true,
    }
}
