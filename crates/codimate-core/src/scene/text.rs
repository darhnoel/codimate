//! The `Text` Node: a label with animated position, font size, and fill.

use super::{Node, TimeCurve};
use crate::value::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
}

/// A Node (Layer 2): a text label with position, font size, and fill.
/// Timeless — no duration.
///
/// ```
/// use codimate_core::{text, tween, Color};
///
/// let t = text()
///     .x(tween(0.0, 100.0))
///     .y(50.0)
///     .text("hello")
///     .font_size(24.0)
///     .fill(Color::RED);
///
/// let at_mid = t.resolve(0.5);
/// assert_eq!(at_mid.x, 50.0);
/// assert_eq!(at_mid.text, "hello");
/// ```
#[derive(Clone)]
pub struct Text {
    x: Animated<f32>,
    y: Animated<f32>,
    text: Animated<String>,
    font_size: Animated<f32>,
    fill: Animated<Color>,
    align: TextAlign,
}

/// A `Text` resolved at a specific `t` — all plain values.
#[derive(Clone, Debug, PartialEq)]
pub struct ConcreteText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub font_size: f32,
    pub fill: Color,
    pub align: TextAlign,
}

impl Text {
    pub fn new() -> Self {
        Text {
            x: 0.0.into_animated(),
            y: 0.0.into_animated(),
            text: String::new().into_animated(),
            font_size: 16.0.into_animated(),
            fill: Color::WHITE.into_animated(),
            align: TextAlign::Left,
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

    pub fn text(mut self, text: impl IntoAnimated<String>) -> Self {
        self.text = text.into_animated();
        self
    }

    pub fn font_size(mut self, size: impl IntoAnimated<f32>) -> Self {
        self.font_size = size.into_animated();
        self
    }

    pub fn fill(mut self, c: impl IntoAnimated<Color>) -> Self {
        self.fill = c.into_animated();
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub(crate) fn with_opacity(self, multiplier: Animated<f32>) -> Self {
        let fill = self.fill;
        Text {
            fill: Animated::new(move |t| {
                let mut color = fill.resolve(t);
                color.a *= multiplier.resolve(t);
                color
            }),
            ..self
        }
    }

    pub(crate) fn ease(self, curve: TimeCurve) -> Self {
        let x_curve = curve.clone();
        let y_curve = curve.clone();
        let text_curve = curve.clone();
        let font_size_curve = curve.clone();
        Text {
            x: Animated::new(move |t| self.x.resolve(x_curve(t))),
            y: Animated::new(move |t| self.y.resolve(y_curve(t))),
            text: Animated::new(move |t| self.text.resolve(text_curve(t))),
            font_size: Animated::new(move |t| self.font_size.resolve(font_size_curve(t))),
            fill: Animated::new(move |t| self.fill.resolve(curve(t))),
            align: self.align,
        }
    }

    pub(crate) fn try_lerp_to(&self, to: &Self) -> Self {
        let from_text = self.text.clone();
        let to_text = to.text.clone();
        Text {
            x: tween(self.x.clone(), to.x.clone()),
            y: tween(self.y.clone(), to.y.clone()),
            text: Animated::new(move |t| {
                if t < 0.5 {
                    from_text.resolve(t)
                } else {
                    to_text.resolve(t)
                }
            }),
            font_size: tween(self.font_size.clone(), to.font_size.clone()),
            fill: tween(self.fill.clone(), to.fill.clone()),
            align: to.align,
        }
    }

    pub fn resolve(&self, t: f32) -> ConcreteText {
        ConcreteText {
            x: self.x.resolve(t),
            y: self.y.resolve(t),
            text: self.text.resolve(t),
            font_size: self.font_size.resolve(t),
            fill: self.fill.resolve(t),
            align: self.align,
        }
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for Text {
    type Concrete = ConcreteText;

    fn resolve(&self, t: f32) -> Self::Concrete {
        Text::resolve(self, t)
    }
}

/// Lowercase free constructor so scenes read like English: `text().font_size(..)`.
pub fn text() -> Text {
    Text::new()
}
