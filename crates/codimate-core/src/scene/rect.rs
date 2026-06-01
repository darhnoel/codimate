//! The `Rect` Node: animated position, size, and fill.

use super::{AnchorKind, Node};
use crate::value::*;

/// A Node (Layer 2): a rectangle with animated layout and fill properties.
/// Timeless — no duration. Nodes do not render themselves (Invariant 3).
///
/// ```
/// use codimate_core::{rect, tween, Color};
///
/// let r = rect()
///     .x(tween(0.0, 100.0))
///     .y(50.0)
///     .width(120.0)
///     .height(40.0)
///     .fill(Color::RED);
///
/// let at_mid = r.resolve(0.5);
/// assert_eq!(at_mid.x, 50.0);
/// assert_eq!(at_mid.y, 50.0);
/// ```
#[derive(Clone)]
pub struct Rect {
    x: Animated<f32>,
    y: Animated<f32>,
    width: Animated<f32>,
    height: Animated<f32>,
    fill: Animated<Color>,
}

/// A `Rect` resolved at a specific `t` — all plain values, no Skia.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConcreteRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub fill: Color,
}

impl ConcreteRect {
    pub fn anchor(&self, kind: AnchorKind) -> Vec2 {
        match kind {
            AnchorKind::Center => Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0),
            AnchorKind::Top => Vec2::new(self.x + self.width / 2.0, self.y),
            AnchorKind::Bottom => Vec2::new(self.x + self.width / 2.0, self.y + self.height),
            AnchorKind::Left => Vec2::new(self.x, self.y + self.height / 2.0),
            AnchorKind::Right => Vec2::new(self.x + self.width, self.y + self.height / 2.0),
        }
    }
}

impl Rect {
    /// Defaults: `x = y = width = height = 0.0`, `fill = opaque white`.
    pub fn new() -> Self {
        Rect {
            x: 0.0.into_animated(),
            y: 0.0.into_animated(),
            width: 0.0.into_animated(),
            height: 0.0.into_animated(),
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

    pub fn width(mut self, width: impl IntoAnimated<f32>) -> Self {
        self.width = width.into_animated();
        self
    }

    pub fn height(mut self, height: impl IntoAnimated<f32>) -> Self {
        self.height = height.into_animated();
        self
    }

    pub fn fill(mut self, c: impl IntoAnimated<Color>) -> Self {
        self.fill = c.into_animated();
        self
    }

    /// A named anchor point derived from the shape's animated geometry.
    ///
    /// ```
    /// use codimate_core::{rect, tween, AnchorKind};
    /// let r = rect().x(tween(0.0, 100.0)).y(50.0).width(80.0).height(40.0);
    /// let right = r.anchor(AnchorKind::Right);
    /// assert_eq!(right.resolve(0.0), codimate_core::Vec2::new(80.0, 70.0));
    /// assert_eq!(right.resolve(1.0), codimate_core::Vec2::new(180.0, 70.0));
    /// ```
    pub fn anchor(&self, kind: AnchorKind) -> Animated<Vec2> {
        let x = self.x.clone();
        let y = self.y.clone();
        let w = self.width.clone();
        let h = self.height.clone();
        match kind {
            AnchorKind::Center => Animated::new(move |t| {
                Vec2::new(
                    x.resolve(t) + w.resolve(t) / 2.0,
                    y.resolve(t) + h.resolve(t) / 2.0,
                )
            }),
            AnchorKind::Top => {
                Animated::new(move |t| Vec2::new(x.resolve(t) + w.resolve(t) / 2.0, y.resolve(t)))
            }
            AnchorKind::Bottom => Animated::new(move |t| {
                Vec2::new(
                    x.resolve(t) + w.resolve(t) / 2.0,
                    y.resolve(t) + h.resolve(t),
                )
            }),
            AnchorKind::Left => {
                Animated::new(move |t| Vec2::new(x.resolve(t), y.resolve(t) + h.resolve(t) / 2.0))
            }
            AnchorKind::Right => Animated::new(move |t| {
                Vec2::new(
                    x.resolve(t) + w.resolve(t),
                    y.resolve(t) + h.resolve(t) / 2.0,
                )
            }),
        }
    }

    /// An evenly-divided anchor slot ("port `index` of `count`") along an edge,
    /// so several fan-in Connections don't pile onto one point. Ports sit at the
    /// centres of `count` equal slots; `port(_, 0, 1)` is the edge midpoint
    /// (i.e. the plain [`anchor`](Self::anchor)). `Center` ignores the port.
    ///
    /// ```
    /// use codimate_core::{rect, AnchorKind, Vec2};
    /// // The three Q/K/V arrows into an attention box: 3 ports on the bottom edge.
    /// let attn = rect().x(0.0).y(0.0).width(120.0).height(40.0);
    /// let k = attn.anchor_port(AnchorKind::Bottom, 1, 3); // the middle of three
    /// assert_eq!(k.resolve(0.0), Vec2::new(60.0, 40.0));
    /// ```
    pub fn anchor_port(&self, kind: AnchorKind, index: usize, count: usize) -> Animated<Vec2> {
        let x = self.x.clone();
        let y = self.y.clone();
        let w = self.width.clone();
        let h = self.height.clone();
        let f = (index as f32 + 0.5) / count.max(1) as f32;
        match kind {
            AnchorKind::Center => Animated::new(move |t| {
                Vec2::new(
                    x.resolve(t) + w.resolve(t) / 2.0,
                    y.resolve(t) + h.resolve(t) / 2.0,
                )
            }),
            AnchorKind::Top => {
                Animated::new(move |t| Vec2::new(x.resolve(t) + f * w.resolve(t), y.resolve(t)))
            }
            AnchorKind::Bottom => Animated::new(move |t| {
                Vec2::new(x.resolve(t) + f * w.resolve(t), y.resolve(t) + h.resolve(t))
            }),
            AnchorKind::Left => {
                Animated::new(move |t| Vec2::new(x.resolve(t), y.resolve(t) + f * h.resolve(t)))
            }
            AnchorKind::Right => Animated::new(move |t| {
                Vec2::new(x.resolve(t) + w.resolve(t), y.resolve(t) + f * h.resolve(t))
            }),
        }
    }

    /// `f(t) → Concrete` — resolves every `Animated` field at the same `t`.
    pub fn resolve(&self, t: f32) -> ConcreteRect {
        ConcreteRect {
            x: self.x.resolve(t),
            y: self.y.resolve(t),
            width: self.width.resolve(t),
            height: self.height.resolve(t),
            fill: self.fill.resolve(t),
        }
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for Rect {
    type Concrete = ConcreteRect;

    fn resolve(&self, t: f32) -> Self::Concrete {
        Rect::resolve(self, t)
    }
}

/// Lowercase free constructor so scenes read like English: `rect().width(..)`.
pub fn rect() -> Rect {
    Rect::new()
}
