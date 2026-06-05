//! Layer 1 — Value.
//!
//! How a single value changes over `t`. Everything here is **timeless** (no
//! duration) and **pure**. The key type is [`Animated<T>`]; the leaf types
//! (`Vec2`, `Color`, `Style`) are the values that get animated.
//!
//! Geometry types (`Segment`, `Path`, shape constructors) live in
//! [`super::path`](crate::path).

use std::sync::Arc;

/// A 2D vector — a leaf value (position or size).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
}

/// An RGBA color — a leaf value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
}

/// A coordinated visual style — plain leaf values, timeless and lerpable.
///
/// ```
/// use codimate_core::{tween, Color, Style};
///
/// let rest = Style::new()
///     .fill(Color::WHITE)
///     .stroke(1.0, Color::BLACK);
/// let active = Style::new()
///     .fill(Color::RED)
///     .stroke(4.0, Color::CYAN);
///
/// let style = tween(rest, active).resolve(0.5);
/// assert_eq!(style.stroke_width, 2.5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub fill: Color,
    pub stroke_width: f32,
    pub stroke_color: Color,
}

impl Style {
    /// Defaults match `PathNode`: white fill, no visible stroke.
    pub fn new() -> Self {
        Style {
            fill: Color::WHITE,
            stroke_width: 0.0,
            stroke_color: Color::WHITE,
        }
    }

    pub fn fill(mut self, fill: Color) -> Self {
        self.fill = fill;
        self
    }

    pub fn stroke(mut self, width: f32, color: Color) -> Self {
        self.stroke_width = width;
        self.stroke_color = color;
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}



/// Layer 1 — a value that resolves at time `t ∈ [0.0, 1.0]`.
///
/// Timeless (no `duration`) and pure (`resolve` depends only on `t`).
/// A plain leaf value is trivially `Animated` — a constant function of `t`.
///
/// ```
/// use codimate_core::{Animated, IntoAnimated};
///
/// // A constant: ignores t, same value at every moment.
/// let fixed = 50.0_f32.into_animated();
/// assert_eq!(fixed.resolve(0.0), 50.0);
/// assert_eq!(fixed.resolve(1.0), 50.0);
///
/// // Custom motion via the escape hatch — the closure must be pure.
/// let grow = Animated::new(|t| 50.0 + t * 50.0);
/// assert_eq!(grow.resolve(0.5), 75.0);
/// ```
#[derive(Clone)]
pub struct Animated<T>(Arc<dyn Fn(f32) -> T>);

impl<T> Animated<T> {
    /// Escape hatch for custom motion. The closure MUST be pure (Invariant 1).
    pub fn new(f: impl Fn(f32) -> T + 'static) -> Self {
        Animated(Arc::new(f))
    }

    /// `f(t)` for a single value — the Layer 1 analogue of `f(t) → Scene`.
    pub fn resolve(&self, t: f32) -> T {
        (self.0)(t)
    }

    /// Remap this value's time through an easing curve: `resolve(t)` becomes
    /// `self.resolve(curve(t))`. Pure and timeless.
    ///
    /// The curve may push `t` outside `[0,1]` (overshoot, e.g. [`back`]); the
    /// eased value still *receives* `t ∈ [0,1]` from its Animation context, so
    /// this is not an Invariant 2 violation.
    ///
    /// ```
    /// use codimate_core::{tween, ease_in};
    ///
    /// let r = tween(0.0, 100.0).ease(ease_in);
    /// assert_eq!(r.resolve(0.5), 25.0);   // eased: a quarter of the way, not half
    /// ```
    pub fn ease(self, curve: impl Fn(f32) -> f32 + 'static) -> Animated<T>
    where
        T: 'static,
    {
        Animated::new(move |t| self.resolve(curve(t)))
    }

    /// Transform the wrapped value by `f` at resolve time.
    pub fn map<U>(self, f: impl Fn(T) -> U + 'static) -> Animated<U>
    where
        T: 'static,
        U: 'static,
    {
        Animated::new(move |t| f(self.resolve(t)))
    }
}

// A plain leaf value is trivially Animated: a constant function of `t`.
impl From<f32> for Animated<f32> {
    fn from(v: f32) -> Self {
        Animated(Arc::new(move |_| v))
    }
}

impl From<Color> for Animated<Color> {
    fn from(v: Color) -> Self {
        Animated(Arc::new(move |_| v))
    }
}

impl From<Vec2> for Animated<Vec2> {
    fn from(v: Vec2) -> Self {
        Animated(Arc::new(move |_| v))
    }
}

impl From<Style> for Animated<Style> {
    fn from(v: Style) -> Self {
        Animated(Arc::new(move |_| v))
    }
}

impl From<String> for Animated<String> {
    fn from(s: String) -> Self {
        Animated(Arc::new(move |_| s.clone()))
    }
}

impl From<&'static str> for Animated<String> {
    fn from(s: &'static str) -> Self {
        Animated(Arc::new(move |_| s.to_string()))
    }
}

/// The conversion every public API accepts (Invariant 7): a plain leaf value
/// or an already-`Animated` value can be passed without ceremony.
pub trait IntoAnimated<T> {
    fn into_animated(self) -> Animated<T>;
}

impl<T, U: Into<Animated<T>>> IntoAnimated<T> for U {
    fn into_animated(self) -> Animated<T> {
        self.into()
    }
}

/// A leaf value that knows how to linearly interpolate between two of itself.
///
/// Symmetric: neither endpoint is privileged. Not clamped — `t` outside
/// `[0,1]` extrapolates, which leaves room for future easing to overshoot.
/// (Invariant 2 keeps the normal input range to `[0,1]`.)
pub trait Lerp {
    /// The value `t` of the way from `a` to `b`.
    fn lerp(a: Self, b: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        a + (b - a) * t
    }
}

impl Lerp for Vec2 {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        Vec2 {
            x: f32::lerp(a.x, b.x, t),
            y: f32::lerp(a.y, b.y, t),
        }
    }
}

impl Lerp for Color {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        Color {
            r: f32::lerp(a.r, b.r, t),
            g: f32::lerp(a.g, b.g, t),
            b: f32::lerp(a.b, b.b, t),
            a: f32::lerp(a.a, b.a, t),
        }
    }
}

impl Lerp for Style {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        Style {
            fill: Color::lerp(a.fill, b.fill, t),
            stroke_width: f32::lerp(a.stroke_width, b.stroke_width, t),
            stroke_color: Color::lerp(a.stroke_color, b.stroke_color, t),
        }
    }
}

/// Layer 1 builder: an `Animated<T>` that travels from `from` (at `t = 0.0`)
/// to `to` (at `t = 1.0`) by interpolation.
///
/// **Timeless** — no duration; how long the travel takes is decided in Layer 3.
/// Endpoints are `impl IntoAnimated<T>` (Invariant 7), so either may be a plain
/// leaf value or an already-`Animated` value.
///
/// ```
/// use codimate_core::tween;
///
/// let radius = tween(50.0, 100.0);     // grows 50 -> 100 across t
/// assert_eq!(radius.resolve(0.0), 50.0);
/// assert_eq!(radius.resolve(0.5), 75.0);
/// assert_eq!(radius.resolve(1.0), 100.0);
/// ```
pub fn tween<T>(from: impl IntoAnimated<T>, to: impl IntoAnimated<T>) -> Animated<T>
where
    T: Lerp + 'static,
{
    let from = from.into_animated();
    let to = to.into_animated();
    Animated::new(move |t| T::lerp(from.resolve(t), to.resolve(t), t))
}
