//! The `Circle` Node: animated centre, radius, and fill.

use super::{AnchorKind, Node};
use crate::value::*;

/// A Node (Layer 2): pure data whose every property is an `Animated<T>`.
/// Timeless — no duration. Nodes do not render themselves (Invariant 3).
///
/// ```
/// use codimate_core::{circle, tween, Color};
///
/// let c = circle()
///     .x(tween(0.0, 100.0))   // sweeps left -> right
///     .y(50.0)                // holds, independent of x
///     .radius(20.0)
///     .fill(Color::RED);
///
/// let at_mid = c.resolve(0.5);
/// assert_eq!(at_mid.x, 50.0);
/// assert_eq!(at_mid.y, 50.0);
/// ```
#[derive(Clone)]
pub struct Circle {
    x: Animated<f32>,
    y: Animated<f32>,
    radius: Animated<f32>,
    fill: Animated<Color>,
}

/// A `Circle` resolved at a specific `t` — all plain values, no Skia.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConcreteCircle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub fill: Color,
}

impl ConcreteCircle {
    pub fn anchor(&self, kind: AnchorKind) -> Vec2 {
        match kind {
            AnchorKind::Center => Vec2::new(self.x, self.y),
            AnchorKind::Top => Vec2::new(self.x, self.y - self.radius),
            AnchorKind::Bottom => Vec2::new(self.x, self.y + self.radius),
            AnchorKind::Left => Vec2::new(self.x - self.radius, self.y),
            AnchorKind::Right => Vec2::new(self.x + self.radius, self.y),
        }
    }
}

impl Circle {
    /// Defaults: `x = y = radius = 0.0`, `fill = opaque white`.
    pub fn new() -> Self {
        Circle {
            x: 0.0.into_animated(),
            y: 0.0.into_animated(),
            radius: 0.0.into_animated(),
            fill: Color::WHITE.into_animated(),
        }
    }

    pub fn x(mut self, x: impl IntoAnimated<f32>) -> Self {
        self.x = x.into_animated();
        self
    }

    pub fn y(mut self, y: impl IntoAnimated<f32>) -> Self {
        self.y = y.into_animated();
        self
    }

    pub fn radius(mut self, r: impl IntoAnimated<f32>) -> Self {
        self.radius = r.into_animated();
        self
    }

    pub fn fill(mut self, c: impl IntoAnimated<Color>) -> Self {
        self.fill = c.into_animated();
        self
    }

    /// A named anchor point derived from the shape's animated geometry.
    ///
    /// Returns an `Animated<Vec2>` that tracks the anchor across time.
    ///
    /// ```
    /// use codimate_core::{circle, tween, AnchorKind};
    /// let c = circle().x(tween(0.0, 100.0)).y(200.0).radius(50.0);
    /// let top = c.anchor(AnchorKind::Top);
    /// assert_eq!(top.resolve(0.0), codimate_core::Vec2::new(0.0, 150.0));
    /// assert_eq!(top.resolve(1.0), codimate_core::Vec2::new(100.0, 150.0));
    /// ```
    pub fn anchor(&self, kind: AnchorKind) -> Animated<Vec2> {
        let x = self.x.clone();
        let y = self.y.clone();
        let r = self.radius.clone();
        match kind {
            AnchorKind::Center => Animated::new(move |t| Vec2::new(x.resolve(t), y.resolve(t))),
            AnchorKind::Top => {
                Animated::new(move |t| Vec2::new(x.resolve(t), y.resolve(t) - r.resolve(t)))
            }
            AnchorKind::Bottom => {
                Animated::new(move |t| Vec2::new(x.resolve(t), y.resolve(t) + r.resolve(t)))
            }
            AnchorKind::Left => {
                Animated::new(move |t| Vec2::new(x.resolve(t) - r.resolve(t), y.resolve(t)))
            }
            AnchorKind::Right => {
                Animated::new(move |t| Vec2::new(x.resolve(t) + r.resolve(t), y.resolve(t)))
            }
        }
    }

    /// `f(t) → Concrete` — resolves every `Animated` field at the same `t`.
    pub fn resolve(&self, t: f32) -> ConcreteCircle {
        ConcreteCircle {
            x: self.x.resolve(t),
            y: self.y.resolve(t),
            radius: self.radius.resolve(t),
            fill: self.fill.resolve(t),
        }
    }
}

impl Default for Circle {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for Circle {
    type Concrete = ConcreteCircle;

    fn resolve(&self, t: f32) -> Self::Concrete {
        Circle::resolve(self, t)
    }
}

/// Lowercase free constructor so scenes read like English: `circle().radius(..)`.
pub fn circle() -> Circle {
    Circle::new()
}
