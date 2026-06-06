//! The `Connection` Node: a stroked line between two anchors, optional arrowhead.

use super::{ConcretePath, TimeCurve};
use crate::{path::*, value::*};

/// A Node (Layer 2) that links two shape Anchors with a stroked line,
/// optionally ending in an arrowhead at the target (end) anchor.
///
/// Construction is clone-based: keep the shapes in variables, clone them into
/// the scene, and pass their anchors to the connection.
///
/// ```
/// use codimate_core::{circle, connection, AnchorKind, Color};
///
/// let a = circle().x(50.0).y(100.0).radius(20.0);
/// let b = circle().x(200.0).y(100.0).radius(20.0);
///
/// let conn = connection(
///     a.anchor(AnchorKind::Right),
///     b.anchor(AnchorKind::Left),
/// )
/// .stroke(2.0, Color::RED)
/// .arrow(10.0);
///
/// let resolved = conn.resolve(0.0);
/// // resolved.start → Vec2(70, 100), resolved.end → Vec2(180, 100)
/// ```
#[derive(Clone)]
pub struct Connection {
    start: Animated<Vec2>,
    end: Animated<Vec2>,
    waypoints: Vec<Animated<Vec2>>,
    stroke_width: Animated<f32>,
    stroke_color: Animated<Color>,
    arrow_size: Animated<f32>,
    reveal_progress: Animated<f32>,
}

impl Connection {
    pub fn new(start: impl IntoAnimated<Vec2>, end: impl IntoAnimated<Vec2>) -> Self {
        Connection {
            start: start.into_animated(),
            end: end.into_animated(),
            waypoints: Vec::new(),
            stroke_width: 1.0.into_animated(),
            stroke_color: Color::WHITE.into_animated(),
            arrow_size: 0.0.into_animated(),
            reveal_progress: 1.0.into_animated(),
        }
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
        let stroke_color = self.stroke_color;
        Connection {
            stroke_color: Animated::new(move |t| {
                let mut color = stroke_color.resolve(t);
                color.a *= multiplier.resolve(t);
                color
            }),
            ..self
        }
    }

    pub(crate) fn ease(self, curve: TimeCurve) -> Self {
        let start_curve = curve.clone();
        let end_curve = curve.clone();
        let stroke_width_curve = curve.clone();
        let stroke_color_curve = curve.clone();
        let arrow_size_curve = curve.clone();
        Connection {
            start: Animated::new(move |t| self.start.resolve(start_curve(t))),
            end: Animated::new(move |t| self.end.resolve(end_curve(t))),
            waypoints: self
                .waypoints
                .into_iter()
                .map(|waypoint| {
                    let curve = curve.clone();
                    Animated::new(move |t| waypoint.resolve(curve(t)))
                })
                .collect(),
            stroke_width: Animated::new(move |t| self.stroke_width.resolve(stroke_width_curve(t))),
            stroke_color: Animated::new(move |t| self.stroke_color.resolve(stroke_color_curve(t))),
            arrow_size: Animated::new(move |t| self.arrow_size.resolve(arrow_size_curve(t))),
            reveal_progress: Animated::new(move |t| self.reveal_progress.resolve(curve(t))),
        }
    }

    pub(crate) fn try_lerp_to(&self, to: &Self) -> Result<Self, super::SceneTransformError> {
        if self.waypoints.len() != to.waypoints.len() {
            return Err(super::SceneTransformError::WaypointCountMismatch {
                from: self.waypoints.len(),
                to: to.waypoints.len(),
            });
        }

        Ok(Connection {
            start: tween(self.start.clone(), to.start.clone()),
            end: tween(self.end.clone(), to.end.clone()),
            waypoints: self
                .waypoints
                .iter()
                .zip(&to.waypoints)
                .map(|(from, to)| tween(from.clone(), to.clone()))
                .collect(),
            stroke_width: tween(self.stroke_width.clone(), to.stroke_width.clone()),
            stroke_color: tween(self.stroke_color.clone(), to.stroke_color.clone()),
            arrow_size: tween(self.arrow_size.clone(), to.arrow_size.clone()),
            reveal_progress: tween(self.reveal_progress.clone(), to.reveal_progress.clone()),
        })
    }

    pub(crate) fn reveal(self, progress: Animated<f32>) -> Self {
        Connection {
            reveal_progress: progress,
            ..self
        }
    }

    pub fn arrow(mut self, size: impl IntoAnimated<f32>) -> Self {
        self.arrow_size = size.into_animated();
        self
    }

    /// Add intermediate waypoints the line passes through before reaching the
    /// end anchor. Each waypoint is a bend in the polyline.
    pub fn via(mut self, points: impl IntoIterator<Item = impl IntoAnimated<Vec2>>) -> Self {
        self.waypoints = points.into_iter().map(|p| p.into_animated()).collect();
        self
    }

    /// Build the main line segments (start → waypoints → back) where `back`
    /// is the arrow-base point (or `end` when arrow is absent). Also returns
    /// the unit direction from the last pre-arrow vertex toward `end`.
    fn resolve_main(&self, t: f32) -> (Vec<Segment>, Vec2, f32, f32) {
        let start = self.start.resolve(t);
        let end = self.end.resolve(t);
        let as_ = self.arrow_size.resolve(t);

        let mut prev = start;
        let mut segs = Vec::new();
        for wp in &self.waypoints {
            let p = wp.resolve(t);
            segs.push(Segment::Line(prev, p));
            prev = p;
        }

        // Direction from last vertex toward end
        let dx = end.x - prev.x;
        let dy = end.y - prev.y;
        let len = (dx * dx + dy * dy).sqrt();

        let (back, ux, uy) = if as_ > 0.0 && len > 0.0 {
            let ux = dx / len;
            let uy = dy / len;
            (Vec2::new(end.x - ux * as_, end.y - uy * as_), ux, uy)
        } else {
            (end, 0.0, 0.0)
        };

        segs.push(Segment::Line(prev, back));
        (segs, back, ux, uy)
    }

    /// Resolve both anchors and build a path at this moment in time.
    /// Produces a ConcretePath that the renderer draws as a stroked line
    /// (plus optional arrowhead).
    pub fn resolve(&self, t: f32) -> ConcretePath {
        let (mut segments, back, ux, uy) = self.resolve_main(t);
        let sw = self.stroke_width.resolve(t);
        let sc = self.stroke_color.resolve(t);
        let as_ = self.arrow_size.resolve(t);

        if as_ > 0.0 && (ux != 0.0 || uy != 0.0) {
            let end = self.end.resolve(t);
            let nx = -uy;
            let ny = ux;
            let spread = as_ * 0.4;
            let w1 = Vec2::new(back.x + nx * spread, back.y + ny * spread);
            let w2 = Vec2::new(back.x - nx * spread, back.y - ny * spread);
            segments.push(Segment::Line(back, w1));
            segments.push(Segment::Line(w1, end));
            segments.push(Segment::Line(end, w2));
            segments.push(Segment::Line(w2, back));
        }

        prefix_concrete_path(
            ConcretePath {
                path: Path {
                    segments,
                    closed: false,
                },
                fill: Color::TRANSPARENT,
                stroke_width: sw,
                stroke_color: sc,
            },
            self.reveal_progress.resolve(t),
        )
    }

    /// Resolve just the main line, excluding arrowhead geometry.
    /// Used by Pulse for clean arc-length traversal.
    pub fn resolve_line(&self, t: f32) -> ConcretePath {
        let (segments, _, _, _) = self.resolve_main(t);
        let sw = self.stroke_width.resolve(t);
        let sc = self.stroke_color.resolve(t);
        prefix_concrete_path(
            ConcretePath {
                path: Path {
                    segments,
                    closed: false,
                },
                fill: Color::TRANSPARENT,
                stroke_width: sw,
                stroke_color: sc,
            },
            self.reveal_progress.resolve(t),
        )
    }
}

fn prefix_concrete_path(mut path: ConcretePath, progress: f32) -> ConcretePath {
    path.path = path.path.prefix(progress);
    path
}

/// Lowercase free constructor so scenes read like English.
pub fn connection(start: impl IntoAnimated<Vec2>, end: impl IntoAnimated<Vec2>) -> Connection {
    Connection::new(start, end)
}
