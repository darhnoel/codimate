//! Layer 1 — Geometry (Segment, Path, and shape constructors).
//!
//! Every shape (circle, rect, polygon, ellipse) can be expressed as a `Path`.
//! Because `Path` implements `Lerp`, `tween(path_a, path_b)` produces shape
//! morphing for free — the core benefit from ADR 0002.

use crate::value::{Animated, Lerp, Vec2};

/// A single curve segment in a Path.
///
/// Each variant owns all its points — `from`, `to`, and control points — so
/// every segment is self-describing and inspectable without traversal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    MoveTo(Vec2),
    Line(Vec2, Vec2),
    Quad(Vec2, Vec2, Vec2),
    Cubic(Vec2, Vec2, Vec2, Vec2),
    Close,
}

impl Segment {
    pub fn to_cubic(self) -> (Vec2, Vec2, Vec2, Vec2) {
        match self {
            Segment::MoveTo(p) => (p, p, p, p),
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
            Segment::Close => (
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.0),
            ),
        }
    }

    pub fn from_cubic(from: Vec2, c1: Vec2, c2: Vec2, to: Vec2) -> Self {
        Segment::Cubic(from, c1, c2, to)
    }

    /// All defining points for this segment (start, controls, end).
    pub fn points(&self) -> Vec<Vec2> {
        match self {
            Segment::MoveTo(p) => vec![*p],
            Segment::Line(a, b) => vec![*a, *b],
            Segment::Quad(a, ctrl, b) => vec![*a, *ctrl, *b],
            Segment::Cubic(a, c1, c2, b) => vec![*a, *c1, *c2, *b],
            Segment::Close => vec![],
        }
    }

    /// Offset every point in this segment by `(dx, dy)`.
    pub fn translate(self, dx: f32, dy: f32) -> Self {
        let t = |v: Vec2| Vec2::new(v.x + dx, v.y + dy);
        match self {
            Segment::MoveTo(p) => Segment::MoveTo(t(p)),
            Segment::Line(a, b) => Segment::Line(t(a), t(b)),
            Segment::Quad(a, c, b) => Segment::Quad(t(a), t(c), t(b)),
            Segment::Cubic(a, c1, c2, b) => Segment::Cubic(t(a), t(c1), t(c2), t(b)),
            Segment::Close => Segment::Close,
        }
    }
}

/// A shape defined by curve segments — the canonical geometry primitive.
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
    /// Offset every point in the path's segments by `(dx, dy)`.
    pub fn translate(self, dx: f32, dy: f32) -> Self {
        Path {
            segments: self
                .segments
                .into_iter()
                .map(|s| s.translate(dx, dy))
                .collect(),
            closed: self.closed,
        }
    }

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
            return self.segments[0].to_cubic().0;
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
        let last = self.segments.last().unwrap();
        let (_, _, _, b) = last.to_cubic();
        b
    }
}

// --- `From` impl so `Path` can be used as `impl IntoAnimated<Path>` ---

impl From<Path> for Animated<Path> {
    fn from(v: Path) -> Self {
        Animated::new(move |_| v.clone())
    }
}

// --- `Lerp` impls for Segment and Path enable tweening/morphing ---

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

// --- Shape constructors ---

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

/// Closed polygon from a sequence of vertices.
pub fn polygon_path(vertices: &[Vec2]) -> Path {
    let mut segments: Vec<Segment> = Vec::with_capacity(vertices.len());
    if vertices.len() < 2 {
        return Path {
            segments,
            closed: true,
        };
    }
    for i in 1..vertices.len() {
        segments.push(Segment::Line(vertices[i - 1], vertices[i]));
    }
    segments.push(Segment::Line(vertices[vertices.len() - 1], vertices[0]));
    Path {
        segments,
        closed: true,
    }
}

/// Regular polygon inscribed in a circle at `(cx, cy)` with radius `r` and `n` sides.
/// The first vertex starts at the top (12 o'clock).
pub fn regular_polygon_path(cx: f32, cy: f32, r: f32, n: u32) -> Path {
    let n = n.max(3);
    let mut vertices = Vec::with_capacity(n as usize);
    for i in 0..n {
        let angle = std::f32::consts::TAU * i as f32 / n as f32 - std::f32::consts::FRAC_PI_2;
        vertices.push(Vec2::new(cx + r * angle.cos(), cy + r * angle.sin()));
    }
    polygon_path(&vertices)
}

/// Equilateral triangle inscribed in a circle at `(cx, cy)` with circumradius `r`.
pub fn triangle_path(cx: f32, cy: f32, r: f32) -> Path {
    regular_polygon_path(cx, cy, r, 3)
}

/// Ellipse at `(cx, cy)` with semi-axes `rx` and `ry`, approximated by four cubic Béziers.
pub fn ellipse_path(cx: f32, cy: f32, rx: f32, ry: f32) -> Path {
    let kx = rx * 0.552_284_9;
    let ky = ry * 0.552_284_9;
    Path {
        segments: vec![
            Segment::Cubic(
                Vec2::new(cx + rx, cy),
                Vec2::new(cx + rx, cy + ky),
                Vec2::new(cx + kx, cy + ry),
                Vec2::new(cx, cy + ry),
            ),
            Segment::Cubic(
                Vec2::new(cx, cy + ry),
                Vec2::new(cx - kx, cy + ry),
                Vec2::new(cx - rx, cy + ky),
                Vec2::new(cx - rx, cy),
            ),
            Segment::Cubic(
                Vec2::new(cx - rx, cy),
                Vec2::new(cx - rx, cy - ky),
                Vec2::new(cx - kx, cy - ry),
                Vec2::new(cx, cy - ry),
            ),
            Segment::Cubic(
                Vec2::new(cx, cy - ry),
                Vec2::new(cx + kx, cy - ry),
                Vec2::new(cx + rx, cy - ky),
                Vec2::new(cx + rx, cy),
            ),
        ],
        closed: true,
    }
}
