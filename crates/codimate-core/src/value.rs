//! Layer 1 — Value.
//!
//! How a single value changes over `t`. Everything here is **timeless** (no
//! duration) and **pure**. The key type is [`Animated<T>`]; the geometry leaf
//! types (`Vec2`, `Color`, `Style`, `Segment`, `Path`) are the values that get
//! animated.

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

/// A single curve segment in a Path.
///
/// Each variant owns all its points — `from`, `to`, and control points — so
/// every segment is self-describing and inspectable without traversal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    Line(Vec2, Vec2),
    Quad(Vec2, Vec2, Vec2),
    Cubic(Vec2, Vec2, Vec2, Vec2),
}

impl Segment {
    pub fn to_cubic(self) -> (Vec2, Vec2, Vec2, Vec2) {
        match self {
            Segment::Line(a, b) => {
                let c1 = Vec2::lerp(a, b, 1.0 / 3.0);
                let c2 = Vec2::lerp(a, b, 2.0 / 3.0);
                (a, c1, c2, b)
            }
            Segment::Quad(a, ctrl, b) => {
                let c1 = Vec2::lerp(a, ctrl, 2.0 / 3.0);
                let c2 = Vec2::lerp(ctrl, b, 2.0 / 3.0);
                (a, c1, c2, b)
            }
            Segment::Cubic(a, c1, c2, b) => (a, c1, c2, b),
        }
    }

    pub fn from_cubic(from: Vec2, c1: Vec2, c2: Vec2, to: Vec2) -> Self {
        Segment::Cubic(from, c1, c2, to)
    }

    /// All defining points for this segment (start, controls, end).
    pub fn points(&self) -> Vec<Vec2> {
        match self {
            Segment::Line(a, b) => vec![*a, *b],
            Segment::Quad(a, ctrl, b) => vec![*a, *ctrl, *b],
            Segment::Cubic(a, c1, c2, b) => vec![*a, *c1, *c2, *b],
        }
    }
}

/// A shape defined by curve segments — the canonical geometry primitive.
///
/// Every shape (circle, rect, polygon) can be expressed as a `Path`. Because
/// `Path` implements `Lerp`, `tween(path_a, path_b)` produces shape morphing
/// for free — the core benefit from ADR 0002.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub segments: Vec<Segment>,
    pub closed: bool,
}

/// Evaluate a cubic Bézier at parameter `t ∈ [0, 1]`.
fn cubic_point(a: Vec2, c1: Vec2, c2: Vec2, b: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    Vec2::new(
        a.x * mt3 + 3.0 * c1.x * mt2 * t + 3.0 * c2.x * mt * t2 + b.x * t3,
        a.y * mt3 + 3.0 * c1.y * mt2 * t + 3.0 * c2.y * mt * t2 + b.y * t3,
    )
}

impl Path {
    /// Axis-aligned bounding box. Returns `None` for an empty path.
    pub fn bounding_box(&self) -> Option<(f32, f32, f32, f32)> {
        let points: Vec<Vec2> = self.segments.iter().flat_map(|s| s.points()).collect();
        let first = *points.first()?;
        let (mut xmin, mut xmax) = (first.x, first.x);
        let (mut ymin, mut ymax) = (first.y, first.y);
        for p in &points {
            if p.x < xmin {
                xmin = p.x;
            }
            if p.x > xmax {
                xmax = p.x;
            }
            if p.y < ymin {
                ymin = p.y;
            }
            if p.y > ymax {
                ymax = p.y;
            }
        }
        Some((xmin, ymin, xmax, ymax))
    }

    /// Arc-length parameterization: the point `t` fraction (0.0–1.0) along
    /// the path, measured by length rather than control-point parameter.
    ///
    /// Uses sub-division (20 steps per segment) for numerical arc-length
    /// approximation.  Returns `(0, 0)` for an empty path.
    ///
    /// ```
    /// use codimate_core::{Path, Segment, Vec2};
    ///
    /// let path = Path {
    ///     segments: vec![Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0))],
    ///     closed: false,
    /// };
    /// let mid = path.point_at(0.5);
    /// assert!((mid.x - 50.0).abs() < 0.1);
    /// assert!((mid.y - 0.0).abs() < 0.1);
    /// ```
    pub fn point_at(&self, t: f32) -> Vec2 {
        let t = t.clamp(0.0, 1.0);
        if self.segments.is_empty() {
            return Vec2::new(0.0, 0.0);
        }
        const STEPS: usize = 20;
        // Flatten each segment into linear sub-segments with accumulated lengths
        let mut cumul: Vec<f32> = Vec::new();
        let mut points: Vec<(Vec2, Vec2)> = Vec::new();
        let mut total = 0.0;
        for seg in &self.segments {
            let (a, c1, c2, b) = seg.to_cubic();
            for i in 0..STEPS {
                let u1 = i as f32 / STEPS as f32;
                let u2 = (i + 1) as f32 / STEPS as f32;
                let p1 = cubic_point(a, c1, c2, b, u1);
                let p2 = cubic_point(a, c1, c2, b, u2);
                points.push((p1, p2));
                let dx = p2.x - p1.x;
                let dy = p2.y - p1.y;
                total += (dx * dx + dy * dy).sqrt();
                cumul.push(total);
            }
        }
        if total <= 0.0 {
            return self.segments[0].to_cubic().0; // first point
        }
        let target = t * total;
        for (i, &c) in cumul.iter().enumerate() {
            if c >= target {
                let prev = if i > 0 { cumul[i - 1] } else { 0.0 };
                let frac = (target - prev) / (c - prev);
                let (p1, p2) = &points[i];
                return Vec2::lerp(*p1, *p2, frac);
            }
        }
        // Clamped to t=1: last point of last segment
        let last = self.segments.last().unwrap();
        let (_, _, _, b) = last.to_cubic();
        b
    }
}

/// Cubic-Bézier approximation of a circle centred at `(cx, cy)` with radius `r`.
/// Uses the standard 4-cubic-segment approximation (k = 0.55228).
pub fn circle_path(cx: f32, cy: f32, r: f32) -> Path {
    let k = r * 0.552_284_9;
    Path {
        segments: vec![
            Segment::Cubic(
                Vec2::new(cx + r, cy),
                Vec2::new(cx + r, cy + k),
                Vec2::new(cx + k, cy + r),
                Vec2::new(cx, cy + r),
            ),
            Segment::Cubic(
                Vec2::new(cx, cy + r),
                Vec2::new(cx - k, cy + r),
                Vec2::new(cx - r, cy + k),
                Vec2::new(cx - r, cy),
            ),
            Segment::Cubic(
                Vec2::new(cx - r, cy),
                Vec2::new(cx - r, cy - k),
                Vec2::new(cx - k, cy - r),
                Vec2::new(cx, cy - r),
            ),
            Segment::Cubic(
                Vec2::new(cx, cy - r),
                Vec2::new(cx + k, cy - r),
                Vec2::new(cx + r, cy - k),
                Vec2::new(cx + r, cy),
            ),
        ],
        closed: true,
    }
}

/// Path for an axis-aligned rectangle at `(x, y)` with given `width` and `height`.
pub fn rect_path(x: f32, y: f32, w: f32, h: f32) -> Path {
    Path {
        segments: vec![
            Segment::Line(Vec2::new(x, y), Vec2::new(x + w, y)),
            Segment::Line(Vec2::new(x + w, y), Vec2::new(x + w, y + h)),
            Segment::Line(Vec2::new(x + w, y + h), Vec2::new(x, y + h)),
            Segment::Line(Vec2::new(x, y + h), Vec2::new(x, y)),
        ],
        closed: true,
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

impl From<Path> for Animated<Path> {
    fn from(v: Path) -> Self {
        Animated(Arc::new(move |_| v.clone()))
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

impl Lerp for Segment {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        let (a0, a1, a2, a3) = a.to_cubic();
        let (b0, b1, b2, b3) = b.to_cubic();
        Segment::from_cubic(
            Vec2::lerp(a0, b0, t),
            Vec2::lerp(a1, b1, t),
            Vec2::lerp(a2, b2, t),
            Vec2::lerp(a3, b3, t),
        )
    }
}

impl Lerp for Path {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        let max_len = a.segments.len().max(b.segments.len());
        let a_end = a
            .segments
            .last()
            .map(|s| s.to_cubic().3)
            .unwrap_or(Vec2::new(0.0, 0.0));
        let b_end = b
            .segments
            .last()
            .map(|s| s.to_cubic().3)
            .unwrap_or(Vec2::new(0.0, 0.0));

        let segments = (0..max_len)
            .map(|i| {
                let a_cubic = a
                    .segments
                    .get(i)
                    .map(|s| s.to_cubic())
                    .unwrap_or((a_end, a_end, a_end, a_end));
                let b_cubic = b
                    .segments
                    .get(i)
                    .map(|s| s.to_cubic())
                    .unwrap_or((b_end, b_end, b_end, b_end));
                Segment::from_cubic(
                    Vec2::lerp(a_cubic.0, b_cubic.0, t),
                    Vec2::lerp(a_cubic.1, b_cubic.1, t),
                    Vec2::lerp(a_cubic.2, b_cubic.2, t),
                    Vec2::lerp(a_cubic.3, b_cubic.3, t),
                )
            })
            .collect();

        Path {
            segments,
            closed: a.closed && b.closed,
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
