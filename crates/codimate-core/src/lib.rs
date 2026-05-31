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
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
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
            fill: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
            .into_animated(),
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

impl Rect {
    /// Defaults: `x = y = width = height = 0.0`, `fill = opaque white`.
    pub fn new() -> Self {
        Rect {
            x: 0.0.into_animated(),
            y: 0.0.into_animated(),
            width: 0.0.into_animated(),
            height: 0.0.into_animated(),
            fill: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
            .into_animated(),
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

/// Lowercase free constructor so scenes read like English: `rect().width(..)`.
pub fn rect() -> Rect {
    Rect::new()
}

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
}

/// A `PathNode` resolved at a specific `t` — concrete geometry and color.
#[derive(Clone, Debug, PartialEq)]
pub struct ConcretePath {
    pub path: Path,
    pub fill: Color,
}

impl PathNode {
    pub fn new() -> Self {
        PathNode {
            path: Path {
                segments: Vec::new(),
                closed: false,
            }
            .into_animated(),
            fill: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
            .into_animated(),
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

    pub fn resolve(&self, t: f32) -> ConcretePath {
        ConcretePath {
            path: self.path.resolve(t),
            fill: self.fill.resolve(t),
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
}

/// A `Text` resolved at a specific `t` — all plain values.
#[derive(Clone, Debug, PartialEq)]
pub struct ConcreteText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub font_size: f32,
    pub fill: Color,
}

impl Text {
    pub fn new() -> Self {
        Text {
            x: 0.0.into_animated(),
            y: 0.0.into_animated(),
            text: String::new().into_animated(),
            font_size: 16.0.into_animated(),
            fill: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
            .into_animated(),
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

    pub fn resolve(&self, t: f32) -> ConcreteText {
        ConcreteText {
            x: self.x.resolve(t),
            y: self.y.resolve(t),
            text: self.text.resolve(t),
            font_size: self.font_size.resolve(t),
            fill: self.fill.resolve(t),
        }
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

/// Lowercase free constructor so scenes read like English: `text().font_size(..)`.
pub fn text() -> Text {
    Text::new()
}

/// Shared Layer 2 interface: pure Node data resolves into concrete data at `t`.
pub trait Node {
    type Concrete;

    fn resolve(&self, t: f32) -> Self::Concrete;
}

impl Node for Circle {
    type Concrete = ConcreteCircle;

    fn resolve(&self, t: f32) -> Self::Concrete {
        Circle::resolve(self, t)
    }
}

impl Node for Rect {
    type Concrete = ConcreteRect;

    fn resolve(&self, t: f32) -> Self::Concrete {
        Rect::resolve(self, t)
    }
}

impl Node for Text {
    type Concrete = ConcreteText;

    fn resolve(&self, t: f32) -> Self::Concrete {
        Text::resolve(self, t)
    }
}

/// A supported Node inside a Scene.
#[derive(Clone)]
pub enum SceneNode {
    Circle(Circle),
    Rect(Rect),
    Path(PathNode),
    Text(Text),
}

/// A resolved Scene child — concrete data only, no rendering backend.
#[derive(Clone, Debug, PartialEq)]
pub enum ConcreteNode {
    Circle(ConcreteCircle),
    Rect(ConcreteRect),
    Path(ConcretePath),
    Text(ConcreteText),
}

impl From<Circle> for SceneNode {
    fn from(circle: Circle) -> Self {
        SceneNode::Circle(circle)
    }
}

impl From<Rect> for SceneNode {
    fn from(rect: Rect) -> Self {
        SceneNode::Rect(rect)
    }
}

impl From<PathNode> for SceneNode {
    fn from(path: PathNode) -> Self {
        SceneNode::Path(path)
    }
}

impl From<Text> for SceneNode {
    fn from(text: Text) -> Self {
        SceneNode::Text(text)
    }
}

impl Node for SceneNode {
    type Concrete = ConcreteNode;

    fn resolve(&self, t: f32) -> Self::Concrete {
        match self {
            SceneNode::Circle(circle) => ConcreteNode::Circle(circle.resolve(t)),
            SceneNode::Rect(rect) => ConcreteNode::Rect(rect.resolve(t)),
            SceneNode::Path(path) => ConcreteNode::Path(path.resolve(t)),
            SceneNode::Text(text) => ConcreteNode::Text(text.resolve(t)),
        }
    }
}

/// A Scene (Layer 2): a tree of Nodes resolved by the same normalized `t`.
///
/// ```
/// use codimate_core::{circle, rect, scene, Color};
///
/// let s = scene()
///     .node(circle().x(10.0).radius(5.0).fill(Color::RED))
///     .node(rect().width(100.0).height(40.0).fill(Color::RED));
///
/// let concrete = s.resolve(0.5);
/// assert_eq!(concrete.children.len(), 2);
/// ```
#[derive(Clone)]
pub struct Scene {
    children: Vec<SceneNode>,
}

/// A Scene resolved at a specific `t` — all children are concrete data.
#[derive(Clone, Debug, PartialEq)]
pub struct ConcreteScene {
    pub children: Vec<ConcreteNode>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            children: Vec::new(),
        }
    }

    pub fn node(mut self, node: impl Into<SceneNode>) -> Self {
        self.children.push(node.into());
        self
    }

    /// `f(t) → ConcreteScene` — resolves every child Node at the same `t`.
    pub fn resolve(&self, t: f32) -> ConcreteScene {
        ConcreteScene {
            children: self.children.iter().map(|node| node.resolve(t)).collect(),
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for Scene {
    type Concrete = ConcreteScene;

    fn resolve(&self, t: f32) -> Self::Concrete {
        Scene::resolve(self, t)
    }
}

/// Lowercase free constructor so scene roots read like English.
pub fn scene() -> Scene {
    Scene::new()
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
