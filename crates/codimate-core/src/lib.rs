//! codimate-core — Layer 1 (Value) and Layer 2 (Scene).
//!
//! Invariant 6: this crate has zero non-pure dependencies (std only).

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
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
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

impl Circle {
    /// Defaults: `x = y = radius = 0.0`, `fill = opaque white`.
    pub fn new() -> Self {
        Circle {
            x: 0.0.into_animated(),
            y: 0.0.into_animated(),
            radius: 0.0.into_animated(),
            fill: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }.into_animated(),
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

/// Lowercase free constructor so scenes read like English: `circle().radius(..)`.
pub fn circle() -> Circle {
    Circle::new()
}

// --- Built-in easing curves (`f32 → f32`). All satisfy curve(0)=0, curve(1)=1. ---

/// Starts slow, accelerates. `t * t`.
pub fn ease_in(t: f32) -> f32 {
    t * t
}

/// Starts fast, decelerates. `1 - (1 - t)^2`.
pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Slow at both ends — quadratic in/out.
pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        let u = 1.0 - t;
        1.0 - 2.0 * u * u
    }
}

/// Overshoots past the target near the end, then settles. Relies on `tween`'s
/// deliberate extrapolation (no clamping) for the overshoot.
pub fn back(t: f32) -> f32 {
    const C1: f32 = 1.701_58;
    const C3: f32 = C1 + 1.0;
    let u = t - 1.0;
    1.0 + C3 * u * u * u + C1 * u * u
}
