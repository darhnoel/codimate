//! The `Pulse`: an overlay dot that travels along a Connection by arc length.

use super::{ConcreteCircle, Connection};
use crate::value::*;

/// A marker (dot) that travels along a Connection's path as progress goes 0→1.
///
/// The Connection (line + arrowhead) stays fully drawn; the Pulse is an *overlay*
/// — it does not reveal the line, it rides on top. Position is computed via
/// arc-length parameterization so the dot moves at uniform screen speed.
///
/// ```
/// use codimate_core::{circle, connection, pulse_on, tween, AnchorKind, Color};
///
/// let a = circle().x(50.0).y(100.0).radius(20.0);
/// let b = circle().x(200.0).y(100.0).radius(20.0);
///
/// let conn = connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left));
/// let pulse = pulse_on(conn, tween(0.0, 1.0)).radius(5.0).fill(Color::CYAN);
///
/// let dot = pulse.resolve(0.0);
/// assert_eq!(dot.radius, 5.0);
/// ```
#[derive(Clone)]
pub struct Pulse {
    source: Connection,
    progress: Animated<f32>,
    radius: Animated<f32>,
    fill: Animated<Color>,
}

impl Pulse {
    pub fn new(source: Connection, progress: impl IntoAnimated<f32>) -> Self {
        Pulse {
            source,
            progress: progress.into_animated(),
            radius: 4.0.into_animated(),
            fill: Color::CYAN.into_animated(),
        }
    }

    pub fn radius(mut self, r: impl IntoAnimated<f32>) -> Self {
        self.radius = r.into_animated();
        self
    }

    pub fn fill(mut self, c: impl IntoAnimated<Color>) -> Self {
        self.fill = c.into_animated();
        self
    }

    /// Resolve the underlying Connection, find the point at `progress` along
    /// its main line (excluding arrowhead geometry) by arc length, and return
    /// a concrete circle at that position.
    pub fn resolve(&self, t: f32) -> ConcreteCircle {
        let line = self.source.resolve_line(t);
        let progress = self.progress.resolve(t);
        let radius = self.radius.resolve(t);
        let fill = self.fill.resolve(t);
        let pos = line.path.point_at(progress);
        ConcreteCircle {
            x: pos.x,
            y: pos.y,
            radius,
            fill,
        }
    }
}

/// Lowercase free constructor.
pub fn pulse_on(source: Connection, progress: impl IntoAnimated<f32>) -> Pulse {
    Pulse::new(source, progress)
}
