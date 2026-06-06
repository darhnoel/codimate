//! The `PathNode`: a Bézier shape with animated geometry, fill, and stroke.

use super::{AnchorKind, TimeCurve};
use crate::{path::*, value::*};

/// A Node (Layer 2): a path shape whose geometry and fill are animated.
///
/// ```
/// use codimate_core::{path_node, circle_path, tween, Color};
///
/// let p = path_node()
///     .path(tween(circle_path(0.0, 0.0, 20.0), circle_path(100.0, 50.0, 40.0)))
///     .fill(Color::RED);
/// let resolved = p.resolve(0.5);
/// assert_eq!(resolved.path.segments.len(), 4);
/// ```
#[derive(Clone)]
pub struct PathNode {
    path: Animated<Path>,
    fill: Animated<Color>,
    stroke_width: Animated<f32>,
    stroke_color: Animated<Color>,
}

/// A `PathNode` resolved at a specific `t` — concrete geometry and colors.
#[derive(Clone, Debug, PartialEq)]
pub struct ConcretePath {
    pub path: Path,
    pub fill: Color,
    pub stroke_width: f32,
    pub stroke_color: Color,
}

impl ConcretePath {
    pub fn anchor(&self, kind: AnchorKind) -> Vec2 {
        let Some((xmin, ymin, xmax, ymax)) = self.path.bounding_box() else {
            return Vec2::new(0.0, 0.0);
        };
        let cx = (xmin + xmax) / 2.0;
        let cy = (ymin + ymax) / 2.0;
        match kind {
            AnchorKind::Center => Vec2::new(cx, cy),
            AnchorKind::Top => Vec2::new(cx, ymin),
            AnchorKind::Bottom => Vec2::new(cx, ymax),
            AnchorKind::Left => Vec2::new(xmin, cy),
            AnchorKind::Right => Vec2::new(xmax, cy),
        }
    }
}

impl PathNode {
    pub fn new() -> Self {
        PathNode {
            path: Path {
                segments: Vec::new(),
                closed: false,
            }
            .into_animated(),
            fill: Color::WHITE.into_animated(),
            stroke_width: 0.0.into_animated(),
            stroke_color: Color::WHITE.into_animated(),
        }
    }

    pub fn path(mut self, path: impl IntoAnimated<Path>) -> Self {
        self.path = path.into_animated();
        self
    }

    pub fn fill(mut self, c: impl IntoAnimated<Color>) -> Self {
        self.fill = c.into_animated();
        self
    }

    pub fn stroke(
        mut self,
        width: impl IntoAnimated<f32>,
        color: impl IntoAnimated<Color>,
    ) -> Self {
        self.stroke_width = width.into_animated();
        self.stroke_color = color.into_animated();
        self
    }

    pub(crate) fn with_opacity(self, multiplier: Animated<f32>) -> Self {
        let fill = self.fill;
        let stroke_color = self.stroke_color;
        let stroke_multiplier = multiplier.clone();
        PathNode {
            fill: Animated::new(move |t| {
                let mut color = fill.resolve(t);
                color.a *= multiplier.resolve(t);
                color
            }),
            stroke_color: Animated::new(move |t| {
                let mut color = stroke_color.resolve(t);
                color.a *= stroke_multiplier.resolve(t);
                color
            }),
            ..self
        }
    }

    pub(crate) fn ease(self, curve: TimeCurve) -> Self {
        let path_curve = curve.clone();
        let fill_curve = curve.clone();
        let stroke_width_curve = curve.clone();
        PathNode {
            path: Animated::new(move |t| self.path.resolve(path_curve(t))),
            fill: Animated::new(move |t| self.fill.resolve(fill_curve(t))),
            stroke_width: Animated::new(move |t| self.stroke_width.resolve(stroke_width_curve(t))),
            stroke_color: Animated::new(move |t| self.stroke_color.resolve(curve(t))),
        }
    }

    pub(crate) fn try_lerp_to(&self, to: &Self) -> Self {
        PathNode {
            path: tween(self.path.clone(), to.path.clone()),
            fill: tween(self.fill.clone(), to.fill.clone()),
            stroke_width: tween(self.stroke_width.clone(), to.stroke_width.clone()),
            stroke_color: tween(self.stroke_color.clone(), to.stroke_color.clone()),
        }
    }

    pub(crate) fn reveal(self, progress: Animated<f32>) -> Self {
        let path = self.path;
        PathNode {
            path: Animated::new(move |t| path.resolve(t).prefix(progress.resolve(t))),
            ..self
        }
    }

    /// Apply a coordinated visual style.
    ///
    /// Builder order is last-wins, so `.style(s).fill(c)` overrides just the
    /// fill while keeping the stroke from `s`.
    ///
    /// ```
    /// use codimate_core::{path_node, rect_path, tween, Color, Style};
    ///
    /// let rest = Style::new()
    ///     .fill(Color::WHITE)
    ///     .stroke(1.0, Color::BLACK);
    /// let active = Style::new()
    ///     .fill(Color::RED)
    ///     .stroke(4.0, Color::CYAN);
    ///
    /// let node = path_node()
    ///     .path(rect_path(0.0, 0.0, 100.0, 50.0))
    ///     .style(tween(rest, active));
    /// assert_eq!(node.resolve(0.5).stroke_width, 2.5);
    /// ```
    pub fn style(mut self, style: impl IntoAnimated<Style>) -> Self {
        let style = style.into_animated();
        let fill = style.clone();
        let stroke_width = style.clone();
        self.fill = Animated::new(move |t| fill.resolve(t).fill);
        self.stroke_width = Animated::new(move |t| stroke_width.resolve(t).stroke_width);
        self.stroke_color = Animated::new(move |t| style.resolve(t).stroke_color);
        self
    }

    /// A named anchor point derived from the shape's animated bounding box.
    ///
    /// For an empty path (no segments), the anchor falls back to `(0.0, 0.0)`.
    ///
    /// ```
    /// use codimate_core::{path_node, rect_path, tween, AnchorKind};
    /// let p = path_node()
    ///     .path(tween(rect_path(0.0, 0.0, 100.0, 50.0), rect_path(10.0, 20.0, 80.0, 40.0)));
    /// let bottom = p.anchor(AnchorKind::Bottom);
    /// assert_eq!(bottom.resolve(0.0), codimate_core::Vec2::new(50.0, 50.0));
    /// ```
    pub fn anchor(&self, kind: AnchorKind) -> Animated<Vec2> {
        let path_anim = self.path.clone();
        match kind {
            AnchorKind::Center => Animated::new(move |t| {
                let path = path_anim.resolve(t);
                let (xmin, ymin, xmax, ymax) = path.bounding_box().unwrap_or((0.0, 0.0, 0.0, 0.0));
                Vec2::new((xmin + xmax) / 2.0, (ymin + ymax) / 2.0)
            }),
            AnchorKind::Top => Animated::new(move |t| {
                let path = path_anim.resolve(t);
                let (xmin, ymin, xmax, _ymax) = path.bounding_box().unwrap_or((0.0, 0.0, 0.0, 0.0));
                Vec2::new((xmin + xmax) / 2.0, ymin)
            }),
            AnchorKind::Bottom => Animated::new(move |t| {
                let path = path_anim.resolve(t);
                let (xmin, _ymin, xmax, ymax) = path.bounding_box().unwrap_or((0.0, 0.0, 0.0, 0.0));
                Vec2::new((xmin + xmax) / 2.0, ymax)
            }),
            AnchorKind::Left => Animated::new(move |t| {
                let path = path_anim.resolve(t);
                let (xmin, ymin, _xmax, ymax) = path.bounding_box().unwrap_or((0.0, 0.0, 0.0, 0.0));
                Vec2::new(xmin, (ymin + ymax) / 2.0)
            }),
            AnchorKind::Right => Animated::new(move |t| {
                let path = path_anim.resolve(t);
                let (_xmin, ymin, xmax, ymax) = path.bounding_box().unwrap_or((0.0, 0.0, 0.0, 0.0));
                Vec2::new(xmax, (ymin + ymax) / 2.0)
            }),
        }
    }

    pub fn resolve(&self, t: f32) -> ConcretePath {
        ConcretePath {
            path: self.path.resolve(t),
            fill: self.fill.resolve(t),
            stroke_width: self.stroke_width.resolve(t),
            stroke_color: self.stroke_color.resolve(t),
        }
    }
}

impl Default for PathNode {
    fn default() -> Self {
        Self::new()
    }
}

pub fn path_node() -> PathNode {
    PathNode::new()
}
