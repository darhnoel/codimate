//! Reconciliation — two pictures in, movement out.
//!
//! Given consecutive Scenes whose shapes carry stable names, pair them by name
//! and turn the differences into `Animated<T>`. A shape present in both moved;
//! one that appears enters; one that disappears leaves. That is the whole idea
//! Codimate rests on (ADR 0008), and it lives here rather than in the bindings
//! so that it can be tested, reused by another frontend, and read without a
//! Python toolchain in the way.
//!
//! The input is a flat `Shape` — a data structure, not an API — so nothing
//! here knows that Python exists.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};

use codimate_animation::Playable;
use codimate_core::{
    scene::AnchorKind, scene::Transformable, tween, Animated, Color, ConcreteScene, Geometry,
    IntoAnimated, Path, Pixels, Primitive, Scene, Segment, Style, TextAlign, Vec2,
};

/// Why a Scene could not be built.
///
/// Every one of these is an authoring mistake — an unknown shape kind, a
/// motion path that does not exist, LaTeX that will not typeset. The frontend
/// decides how to present it; this crate only says what went wrong.
#[derive(Debug, Clone)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

// ============================================================
// Payload — the boundary is a data structure, not an API
// ============================================================

/// One shape from Python. A flat union: a text shape carries unused `w`/`h`.
///
/// Deliberate (ADR 0008) — it makes the diff a uniform field-by-field
/// comparison instead of per-kind special cases. Revisit around 8-10 kinds.
#[derive(Clone, Debug, Default)]
pub struct Shape {
    pub item: String,
    pub kind: String,
    pub x: f32,
    pub y: f32,
    /// For a line, the far end. Unused by every other kind.
    pub x2: f32,
    pub y2: f32,
    /// For a line, the stroke width.
    pub w: f32,
    pub h: f32,
    /// A circle's radius; on a rect, its corner radius; on a formula, how
    /// much of it is revealed (0..1). One flat field, three readings — the
    /// payload is a union, not a class hierarchy.
    pub r: f32,
    pub color: String,
    /// The outline, when `edge_w` is positive. Fill and outline are separate,
    /// so a shape can have both — which is why a bordered box no longer has to
    /// be two stacked rectangles.
    pub edge: String,
    pub edge_w: f32,
    /// A polygon's corners, flat: `[x0, y0, x1, y1, ...]`. The one payload
    /// field that is not a single number — see ADR 0010.
    pub points: Vec<f32>,
    pub text: String,
    pub size: f32,
    pub layer: i32,
    pub opacity: f32,
    /// The rest of `Transform`, which the surface used to leave unreachable.
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotate: f32,
    /// What the shape turns and grows around: center, top, bottom, left, right.
    pub pivot: String,
}

/// Every `kind` Python may send. An unknown kind is a Python `ValueError`,
/// never a silently missing shape.
pub const KINDS: [&str; 9] = [
    "rect", "circle", "text", "line", "formula", "polygon", "curve", "svg", "image",
];

/// A rectangle with rounded corners, in local space, centred on the anchor.
///
/// Built as a `Path` rather than a new `Geometry` arm, which is what the
/// refactor plan says free shapes should be. A radius of 0 still produces the
/// same eight segments, so a rect can animate from square to round corners —
/// two paths only tween if they have matching structure.
fn round_rect_path(s: &Shape) -> Path {
    let (hw, hh) = (s.w / 2.0, s.h / 2.0);
    let r = s.r.max(0.0).min(hw.min(hh));
    let p = |x: f32, y: f32| Vec2::new(x, y);

    // Each corner is one quad through the true corner — visually
    // indistinguishable from an arc at these radii, and one segment instead of
    // the two a cubic approximation would need.
    let corner = |segments: &mut Vec<Segment>, from: Vec2, ctrl: Vec2, to: Vec2| {
        segments.push(Segment::Quad(from, ctrl, to));
    };

    let mut segments = vec![Segment::MoveTo(p(-hw + r, -hh))];
    segments.push(Segment::Line(p(-hw + r, -hh), p(hw - r, -hh)));
    corner(&mut segments, p(hw - r, -hh), p(hw, -hh), p(hw, -hh + r));
    segments.push(Segment::Line(p(hw, -hh + r), p(hw, hh - r)));
    corner(&mut segments, p(hw, hh - r), p(hw, hh), p(hw - r, hh));
    segments.push(Segment::Line(p(hw - r, hh), p(-hw + r, hh)));
    corner(&mut segments, p(-hw + r, hh), p(-hw, hh), p(-hw, hh - r));
    segments.push(Segment::Line(p(-hw, hh - r), p(-hw, -hh + r)));
    corner(&mut segments, p(-hw, -hh + r), p(-hw, -hh), p(-hw + r, -hh));

    Path {
        segments,
        closed: true,
    }
}

/// What a shape turns and grows around.
fn pivot_of(name: &str) -> Result<AnchorKind> {
    Ok(match name {
        "center" | "" => AnchorKind::Center,
        "top" => AnchorKind::Top,
        "bottom" => AnchorKind::Bottom,
        "left" => AnchorKind::Left,
        "right" => AnchorKind::Right,
        other => {
            return Err(Error(format!(
                "unknown pivot {other:?} — use center, top, bottom, left or right"
            )))
        }
    })
}

/// A polygon in local space, relative to its anchor.
///
/// `w` carries whether it is closed — one more reading of a float rather than a
/// boolean in the payload, consistent with `r` and `size`.
fn polygon_path(s: &Shape) -> Path {
    let corners: Vec<Vec2> = s
        .points
        .chunks_exact(2)
        .map(|p| Vec2::new(p[0] - s.x, p[1] - s.y))
        .collect();

    let mut segments = Vec::new();
    if let Some(&first) = corners.first() {
        segments.push(Segment::MoveTo(first));
        for pair in corners.windows(2) {
            segments.push(Segment::Line(pair[0], pair[1]));
        }
        if s.w > 0.0 {
            if let Some(&last) = corners.last() {
                segments.push(Segment::Line(last, first));
            }
        }
    }

    Path {
        segments,
        closed: s.w > 0.0,
    }
}

/// A smooth curve in local space, passing through every one of its points.
///
/// Control points are derived rather than authored (ADR 0012): the author has
/// samples — an easing curve, a streamline, a traced outline — and wants a
/// line through them, not a Bezier to solve for.
///
/// Uniform Catmull-Rom, converted to cubics:
///
/// ```text
/// c1 = p[i]   + (p[i+1] - p[i-1]) / 6
/// c2 = p[i+1] - (p[i+2] - p[i])   / 6
/// ```
///
/// The ends reflect their neighbour so an open curve starts and finishes
/// exactly where it was told to.
fn curve_path(s: &Shape) -> Path {
    let pts: Vec<Vec2> = s
        .points
        .chunks_exact(2)
        .map(|p| Vec2::new(p[0] - s.x, p[1] - s.y))
        .collect();

    let closed = s.w > 0.0;
    let n = pts.len();
    if n < 2 {
        return Path {
            segments: Vec::new(),
            closed,
        };
    }

    // The neighbour used for a tangent: wrapped when closed, reflected when
    // open, so the first and last points keep their exact position either way.
    let at = |i: isize| -> Vec2 {
        if closed {
            pts[i.rem_euclid(n as isize) as usize]
        } else if i < 0 {
            let p0 = pts[0];
            Vec2::new(2.0 * p0.x - pts[1].x, 2.0 * p0.y - pts[1].y)
        } else if i as usize >= n {
            let last = pts[n - 1];
            Vec2::new(2.0 * last.x - pts[n - 2].x, 2.0 * last.y - pts[n - 2].y)
        } else {
            pts[i as usize]
        }
    };

    let sixth = |a: Vec2, b: Vec2| Vec2::new((b.x - a.x) / 6.0, (b.y - a.y) / 6.0);

    let mut segments = vec![Segment::MoveTo(pts[0])];
    let spans = if closed { n } else { n - 1 };
    for i in 0..spans as isize {
        let (p0, p1) = (at(i), at(i + 1));
        let d1 = sixth(at(i - 1), p1);
        let d2 = sixth(p0, at(i + 2));
        segments.push(Segment::Cubic(
            p0,
            Vec2::new(p0.x + d1.x, p0.y + d1.y),
            Vec2::new(p1.x - d2.x, p1.y - d2.y),
            p1,
        ));
    }

    Path { segments, closed }
}

/// A line in local space: from the anchor to the far end.
fn line_path(s: &Shape) -> Path {
    Path {
        segments: vec![Segment::Line(
            Vec2::new(0.0, 0.0),
            Vec2::new(s.x2 - s.x, s.y2 - s.y),
        )],
        closed: false,
    }
}

impl Shape {
    fn geometry(&self, other: &Shape) -> Geometry {
        match self.kind.as_str() {
            "circle" => Geometry::circle(tween(self.r, other.r)),

            // Python's `x` is the centre, so text is always centre-aligned;
            // the renderer does the width measurement (raster.rs).
            "text" => Geometry::Text {
                text: self.text.clone().into_animated(),
                font_size: tween(self.size, other.size),
                align: TextAlign::Center,
            },
            "line" => Geometry::path(tween(line_path(self), line_path(other))),

            // Two polygons only tween if they have the same number of corners;
            // interpolating a triangle into a pentagon has no answer worth
            // inventing. Otherwise the later shape stands for the whole
            // segment, exactly as text does when its content changes.
            "polygon" => {
                if self.points.len() == other.points.len() {
                    Geometry::path(tween(polygon_path(self), polygon_path(other)))
                } else {
                    Geometry::path(polygon_path(other).into_animated())
                }
            }

            // Same rule as a polygon: matching sample counts interpolate, and
            // otherwise the later shape stands for the whole segment.
            "curve" => {
                if self.points.len() == other.points.len() {
                    Geometry::path(tween(curve_path(self), curve_path(other)))
                } else {
                    Geometry::path(curve_path(other).into_animated())
                }
            }

            // A formula is many glyph outlines and an imported SVG is many
            // paths, so neither can be one Geometry. `primitives()` expands
            // them; these arms are never reached.
            // `primitives()` handles these three and returns before asking
            // for a Geometry: a formula and an SVG are many, and an image can
            // fail to decode, which this signature cannot report.
            "formula" | "svg" | "image" => Geometry::rect(0.0.into_animated(), 0.0.into_animated()),

            // Square corners stay a real Rect — the common case keeps the
            // cheaper primitive and the renderer's own rectangle path.
            _ if self.r > 0.0 || other.r > 0.0 => {
                Geometry::path(tween(round_rect_path(self), round_rect_path(other)))
            }

            _ => Geometry::rect(tween(self.w, other.w), tween(self.h, other.h)),
        }
    }

    fn style(&self) -> Result<Style> {
        // An imported SVG may say nothing about colour, which means "as
        // authored" (ADR 0014). Its per-path styles are built during
        // expansion; this shared one is only the fallback for a path the file
        // gave no paint at all, so white is as good an answer as any.
        let fill = if self.kind == "svg" && self.color.is_empty() {
            Color::WHITE
        } else {
            parse_color(&self.color)?
        };

        // A line is drawn, not filled — `w` is its stroke width, and it has no
        // separate edge. An open curve reads the same way: it encloses nothing,
        // so filling it would paint the region between the curve and the chord
        // that closes it, which is never what was meant. Its width comes from
        // `edge_w`, because `w` is already carrying the closed flag.
        if self.kind == "line" {
            return Ok(Style::new().fill(Color::TRANSPARENT).stroke(self.w, fill));
        }
        if self.kind == "curve" && self.w <= 0.0 {
            return Ok(Style::new()
                .fill(Color::TRANSPARENT)
                .stroke(self.edge_w.max(1.0), fill));
        }

        let style = Style::new().fill(fill);
        if self.edge_w <= 0.0 {
            return Ok(style);
        }
        Ok(style.stroke(self.edge_w, parse_color(&self.edge)?))
    }

    /// Python's `y` is always the **centre** of the shape. Rust text is
    /// baseline-positioned, so convert here — an author never meets a baseline.
    fn anchor(&self) -> Vec2 {
        // A formula is centred on its own bounding box when the glyphs are
        // built, so unlike text it needs no baseline correction.
        if self.kind == "text" {
            // ponytail: 0.35 * size approximates cap-height/2; good enough for
            // labels. Swap for a real font metric when text sizing needs it.
            Vec2::new(self.x, self.y + self.size * 0.35)
        } else {
            Vec2::new(self.x, self.y)
        }
    }
}

// ============================================================
// Colors
// ============================================================

fn parse_color(name: &str) -> Result<Color> {
    let c = |r: f32, g: f32, b: f32| Color { r, g, b, a: 1.0 };
    Ok(match name {
        // No fill at all, for a shape that is only an outline. Meaningless
        // before `edge_w` existed; now it is how you draw a ring.
        "none" | "transparent" => Color::TRANSPARENT,
        "white" => Color::WHITE,
        "black" => Color::BLACK,
        "red" => Color::RED,
        "cyan" => Color::CYAN,
        "orange" => c(1.0, 0.60, 0.20),
        "blue" => c(0.35, 0.60, 0.90),
        "green" => c(0.35, 0.75, 0.45),
        "grey" | "gray" => c(0.55, 0.55, 0.55),
        "yellow" => c(0.95, 0.85, 0.30),
        hex if hex.starts_with('#') && hex.len() == 7 => {
            let v = |i: usize| {
                u8::from_str_radix(&hex[i..i + 2], 16)
                    .map(|n| n as f32 / 255.0)
                    .map_err(|_| Error(format!("bad hex color: {hex}")))
            };
            Color {
                r: v(1)?,
                g: v(3)?,
                b: v(5)?,
                a: 1.0,
            }
        }
        other => {
            return Err(Error(format!(
                "unknown color {other:?} — use a name or #rrggbb"
            )))
        }
    })
}

// ============================================================
// Motion rules
// ============================================================

pub type Rule = (String, String, HashMap<String, f32>);

/// Greedy wildcard match — `*` and `?`. Avoids a glob dependency for 15 lines.
fn glob_match(pattern: &str, s: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = s.chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    let (mut star, mut mark) = (usize::MAX, 0usize);

    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = pi;
            pi += 1;
            mark = ti;
        } else if star != usize::MAX {
            pi = star + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

/// Build the `Animated<Vec2>` for one shape's journey, per the matching rule.
pub fn motion_path(a: Vec2, b: Vec2, rules: &[Rule], item: &str) -> Result<Animated<Vec2>> {
    // A shape that did not move does not travel — whatever the rule says.
    // Without this, every item matching `bar_*` performs the arc in place,
    // including the ones the event never touched.
    if (a.x - b.x).abs() < 1e-4 && (a.y - b.y).abs() < 1e-4 {
        return Ok(Animated::new(move |_| a));
    }

    let rule = rules.iter().find(|(pat, _, _)| glob_match(pat, item));

    let (kind, opts) = match rule {
        Some((_, kind, opts)) => (kind.as_str(), opts.clone()),
        None => ("straight", HashMap::new()),
    };

    Ok(match kind {
        "straight" => Animated::new(move |t| lerp(a, b, ease_in_out(t))),

        // No easing at all. A shape that is mid-journey at every event — a
        // turning wheel, an orbit, a conveyor — must not accelerate and stop
        // inside each segment, or it judders once per event.
        "linear" => Animated::new(move |t| lerp(a, b, t)),

        // A parabola: horizontal drift at a constant rate, vertical distance
        // going as t squared. What a thing dropped from rest does, and what
        // every hop of a falling ball should do — it must arrive fast, not
        // settle gently the way an eased path does.
        "fall" => Animated::new(move |t| {
            Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t * t)
        }),

        "lift_carry_drop" => {
            let clearance = opts.get("clearance").copied().unwrap_or(40.0);
            let top = a.y.min(b.y) - clearance;
            let (ta, tb) = (Vec2::new(a.x, top), Vec2::new(b.x, top));
            Animated::new(move |t| {
                if t < 1.0 / 3.0 {
                    lerp(a, ta, ease_in_out(t * 3.0))
                } else if t < 2.0 / 3.0 {
                    lerp(ta, tb, ease_in_out((t - 1.0 / 3.0) * 3.0))
                } else {
                    let l = (t - 2.0 / 3.0) * 3.0;
                    lerp(tb, b, l * l) // ease_in — it falls
                }
            })
        }

        other => {
            return Err(Error(format!(
                "unknown motion path {other:?} — use \"straight\", \"linear\", \"fall\" or \"lift_carry_drop\""
            )))
        }
    })
}

// ============================================================
// Formulas — LaTeX, via codimate-math
// ============================================================

/// Typst's default text size in SVG pixels (11pt, at 4/3 px per pt).
///
/// `codimate_math::formula` typesets at Typst's default size, so this is what
/// turns an author's `size=` — which must mean the same thing it means for
/// `text` — into a scale factor. Measured against it: a cap-height `H` comes
/// back 10.02px, or 0.68 em, which is the usual cap-height ratio.
const FORMULA_EM: f32 = 11.0 * 4.0 / 3.0;

/// How many glyphs are fading at once during a `reveal`.
///
/// At 1.0 each glyph gets exactly its own slice of the time and none of its
/// neighbour's, which marches rather than flows — with thirty glyphs over two
/// seconds that is 66ms each, and it reads as popping. Overlapping them gives
/// each glyph four times as long and keeps several in flight, which is what
/// Manim calls a lag ratio.
const REVEAL_OVERLAP: f32 = 4.0;

/// The last part of a glyph's reveal, during which the drawn outline fades out
/// and the solid glyph fades in. Manim calls the whole move DrawBorderThenFill;
/// this is the "then".
const FILL_TAKEOVER: f32 = 0.35;

/// A glyph outline flattened to a polyline, with the running length at every
/// point, so it can be cut to a fraction of its total length cheaply.
///
/// Curves are sampled rather than measured analytically. At glyph scale the
/// error is far below a pixel, and the alternative is a closed-form arc length
/// for cubics, which does not have one.
pub struct Traced {
    /// `(point, starts a new contour, length along the outline so far)`.
    points: Vec<(Vec2, bool, f32)>,
    length: f32,
}

/// How finely a curve is sampled when flattening. Eight is invisible at glyph
/// size and keeps a whole formula's outline a few thousand points.
const CURVE_SAMPLES: usize = 8;

fn flatten(path: &Path) -> Traced {
    let mut points: Vec<(Vec2, bool)> = Vec::new();
    let mut contour_start: Option<Vec2> = None;

    // A segment carries its own start point, but the previous segment usually
    // ended there already — only emit it when it would actually break.
    let open_at = |points: &mut Vec<(Vec2, bool)>, a: Vec2| {
        let joins = points
            .last()
            .is_some_and(|(p, _)| (p.x - a.x).abs() < 1e-6 && (p.y - a.y).abs() < 1e-6);
        if !joins {
            points.push((a, true));
        }
    };

    for seg in &path.segments {
        match *seg {
            Segment::MoveTo(p) => {
                points.push((p, true));
                contour_start = Some(p);
            }
            Segment::Line(a, b) => {
                open_at(&mut points, a);
                if contour_start.is_none() {
                    contour_start = Some(a);
                }
                points.push((b, false));
            }
            Segment::Quad(a, ..) | Segment::Cubic(a, ..) => {
                open_at(&mut points, a);
                if contour_start.is_none() {
                    contour_start = Some(a);
                }
                let (p0, p1, p2, p3) = seg.to_cubic();
                for step in 1..=CURVE_SAMPLES {
                    let u = step as f32 / CURVE_SAMPLES as f32;
                    points.push((cubic_at(p0, p1, p2, p3, u), false));
                }
            }
            Segment::Close => {
                if let Some(start) = contour_start {
                    points.push((start, false));
                }
            }
        }
    }

    let mut length = 0.0;
    let measured = points
        .into_iter()
        .scan(None::<Vec2>, |previous, (point, starts)| {
            if let (Some(prev), false) = (*previous, starts) {
                length += ((point.x - prev.x).powi(2) + (point.y - prev.y).powi(2)).sqrt();
            }
            *previous = Some(point);
            Some((point, starts, length))
        })
        .collect();

    Traced {
        points: measured,
        length,
    }
}

fn cubic_at(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, u: f32) -> Vec2 {
    let v = 1.0 - u;
    let (a, b, c, d) = (v * v * v, 3.0 * v * v * u, 3.0 * v * u * u, u * u * u);
    Vec2::new(
        a * p0.x + b * p1.x + c * p2.x + d * p3.x,
        a * p0.y + b * p1.y + c * p2.y + d * p3.y,
    )
}

/// The first `fraction` of an outline, as a path — what the pen has drawn so
/// far. The cut lands mid-segment, so the line grows smoothly rather than one
/// sample at a time.
fn trim(traced: &Traced, fraction: f32) -> Path {
    let target = traced.length * fraction.clamp(0.0, 1.0);
    let mut segments = Vec::new();
    let mut previous: Option<(Vec2, f32)> = None;

    for &(point, starts, at) in &traced.points {
        if starts {
            segments.push(Segment::MoveTo(point));
            previous = Some((point, at));
            continue;
        }
        let Some((from, from_at)) = previous else {
            continue;
        };
        if at <= target {
            segments.push(Segment::Line(from, point));
            previous = Some((point, at));
        } else {
            let span = at - from_at;
            let part = if span > 1e-6 {
                (target - from_at) / span
            } else {
                0.0
            };
            if part > 0.0 {
                segments.push(Segment::Line(
                    from,
                    Vec2::new(
                        from.x + (point.x - from.x) * part,
                        from.y + (point.y - from.y) * part,
                    ),
                ));
            }
            break;
        }
    }

    Path {
        segments,
        closed: false,
    }
}

/// Apply `f` to every point of a path.
fn map_path(path: &Path, f: impl Fn(Vec2) -> Vec2) -> Path {
    Path {
        segments: path
            .segments
            .iter()
            .map(|s| match *s {
                Segment::MoveTo(a) => Segment::MoveTo(f(a)),
                Segment::Line(a, b) => Segment::Line(f(a), f(b)),
                Segment::Quad(a, b, c) => Segment::Quad(f(a), f(b), f(c)),
                Segment::Cubic(a, b, c, d) => Segment::Cubic(f(a), f(b), f(c), f(d)),
                Segment::Close => Segment::Close,
            })
            .collect(),
        closed: path.closed,
    }
}

/// Typeset one LaTeX string into glyph outlines, centred, in em units.
///
/// Typeset **once per process**, not once per segment. `codimate-math` already
/// caches the Typst SVG on disk by content hash, but re-parsing that SVG into
/// paths for every segment of an explanation is still waste — an explanation
/// has tens of segments and the answer never changes.
///
/// Cached in em units with the fill left off, so one entry serves every size
/// and colour the same formula is ever drawn at.
pub fn formula_glyphs(latex: &str) -> Result<Arc<Vec<Path>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Vec<Path>>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(hit) = cache.lock().unwrap().get(latex) {
        return Ok(hit.clone());
    }

    let block = codimate_math::formula(latex, Color::WHITE).map_err(|e| {
        Error(match e {
            codimate_math::FormulaError::TypstSpawn(_) => format!(
                "could not run `typst`, which Codimate uses to typeset {latex:?}. \
                 Install it (macOS: `brew install typst`) — like ffmpeg it is an \
                 external tool, not a Python dependency."
            ),
            codimate_math::FormulaError::Mitex(m) => {
                format!("could not read the LaTeX {latex:?}: {m}")
            }
            codimate_math::FormulaError::TypstCompile(m) => {
                format!("could not typeset {latex:?}:\n{m}")
            }
            codimate_math::FormulaError::Svg(m) => {
                format!("could not read the typeset {latex:?}: {m}")
            }
            // Only an imported file can raise these; typst's own output is
            // never read from disk and never contains text elements.
            codimate_math::FormulaError::SvgRead(m) | codimate_math::FormulaError::SvgText(m) => m,
        })
    })?;

    let paths: Vec<Path> = block.glyphs.iter().map(|g| g.resolve(0.0).path).collect();

    // Centre on the block's own bounding box. `GlyphBlock` reports width and
    // height but keeps the glyphs at their absolute page coordinates, so the
    // origin has to be recovered here.
    let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
    let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
    for path in &paths {
        if let Some((x0, y0, x1, y1)) = path.bounding_box() {
            min_x = min_x.min(x0);
            min_y = min_y.min(y0);
            max_x = max_x.max(x1);
            max_y = max_y.max(y1);
        }
    }
    if min_x > max_x {
        return Err(Error(format!("{latex:?} typeset to nothing visible")));
    }
    let (cx, cy) = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);

    let mut centred: Vec<Path> = paths
        .iter()
        .map(|p| {
            map_path(p, |v| {
                Vec2::new((v.x - cx) / FORMULA_EM, (v.y - cy) / FORMULA_EM)
            })
        })
        .collect();

    // Left to right, so revealing part of a formula is a wipe in reading
    // order. Typst emits glyphs in layout order, which is close but not a
    // promise — and for a fraction it puts the whole numerator before the
    // denominator, which would reveal the equation in two passes.
    centred.sort_by(|a, b| {
        let key = |p: &Path| p.bounding_box().map(|(x0, ..)| x0).unwrap_or(0.0);
        key(a)
            .partial_cmp(&key(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let shared = Arc::new(centred);
    cache
        .lock()
        .unwrap()
        .insert(latex.to_string(), shared.clone());
    Ok(shared)
}

/// Decode, premultiply and cache a picture — once per path, for the life of
/// the process.
///
/// Cached for the reason the formula glyphs and SVG artwork are: a 1,200-frame
/// render must not decode the same megabyte 1,200 times. As with those, it
/// also means editing the file mid-session and re-rendering shows the old
/// picture.
fn image_pixels(file: &str) -> Result<Arc<Pixels>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Pixels>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(hit) = cache.lock().unwrap().get(file) {
        return Ok(hit.clone());
    }

    let bytes = std::fs::read(file)
        .map_err(|e| Error(format!("could not read the image {file:?}: {e}")))?;

    // By content, not by extension: a `.png` that is really a JPEG is a
    // mistake worth surviving, and the magic numbers are unambiguous.
    let decoded = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        decode_png(&bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        decode_jpeg(&bytes)
    } else {
        return Err(Error(format!(
            "{file:?} is not a PNG or a JPEG — those are the two Codimate reads"
        )));
    }
    .map_err(|m| Error(format!("could not read the image {file:?}: {m}")))?;

    let shared = Arc::new(decoded);
    cache
        .lock()
        .unwrap()
        .insert(file.to_string(), shared.clone());
    Ok(shared)
}

/// Straight RGBA to premultiplied, which is what tiny-skia blits.
fn premultiplied(width: u32, height: u32, rgba: Vec<u8>) -> Pixels {
    let mut rgba = rgba;
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a == 255 {
            continue;
        }
        for c in 0..3 {
            px[c] = ((px[c] as u32 * a + 127) / 255) as u8;
        }
    }
    Pixels {
        width,
        height,
        rgba,
    }
}

fn decode_png(bytes: &[u8]) -> std::result::Result<Pixels, String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    // Normalise greyscale, palette and 16-bit down to 8-bit colour, and expand
    // transparency into a real alpha channel, so the rest of this sees one
    // layout instead of six.
    decoder.set_transformations(
        png::Transformations::normalize_to_color8() | png::Transformations::ALPHA,
    );
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buffer = vec![0; reader.output_buffer_size().unwrap_or(0)];
    let info = reader.next_frame(&mut buffer).map_err(|e| e.to_string())?;
    buffer.truncate(info.buffer_size());

    let rgba = match info.color_type {
        png::ColorType::Rgba => buffer,
        png::ColorType::GrayscaleAlpha => buffer
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        other => return Err(format!("unsupported PNG colour type {other:?}")),
    };
    Ok(premultiplied(info.width, info.height, rgba))
}

fn decode_jpeg(bytes: &[u8]) -> std::result::Result<Pixels, String> {
    let mut decoder = jpeg_decoder::Decoder::new(std::io::Cursor::new(bytes));
    let pixels = decoder.decode().map_err(|e| e.to_string())?;
    let info = decoder.info().ok_or("no JPEG header")?;

    // JPEG has no alpha, so every pixel is opaque and premultiplying is a
    // no-op — the conversion is only about layout.
    let rgba: Vec<u8> = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => pixels
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        jpeg_decoder::PixelFormat::L8 => pixels.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        other => return Err(format!("unsupported JPEG pixel format {other:?}")),
    };
    Ok(Pixels {
        width: info.width as u32,
        height: info.height as u32,
        rgba,
    })
}

/// An imported SVG, ready to draw.
///
/// Paths are centred on the artwork's own bounding box and scaled so its
/// longest side is 1.0, which makes `size` mean the same thing whatever units
/// the file was drawn in — a 24-unit icon and a 1000-unit diagram both arrive
/// the same size (ADR 0014).
struct SvgArt {
    paths: Vec<(Path, Option<Color>)>,
    /// Labels, in the same normalised space as the paths. Not glyph outlines:
    /// these become `Geometry::Text` and are shaped at draw time by the call
    /// that shapes every other label in the frame (ADR 0014).
    texts: Vec<SvgLabel>,
    /// Normalised extent. The longer side is 1.0.
    size: (f32, f32),
}

struct SvgLabel {
    content: String,
    /// The baseline anchor, normalised. SVG places text on its baseline and so
    /// does the Engine, so this needs no conversion.
    at: Vec2,
    size: f32,
    fill: Option<Color>,
    align: TextAlign,
    /// True when the file said `text-anchor="end"`. The Engine has no right
    /// alignment, so the label is drawn left-aligned from a point shifted back
    /// by its own measured width.
    from_right: bool,
}

/// How much to scale normalised artwork so it fits inside `box`.
///
/// Aspect is always preserved: the drawing fits inside the box the author
/// named, whichever way round it is. A zero or missing box falls back to the
/// artwork's own normalised size, which keeps a forgotten `size=` visible
/// rather than invisible.
fn fit_scale(art: (f32, f32), fit: (f32, f32)) -> f32 {
    let (w, h) = fit;
    if w <= 0.0 && h <= 0.0 {
        return 1.0;
    }
    let by_w = if w > 0.0 { w / art.0 } else { f32::MAX };
    let by_h = if h > 0.0 { h / art.1 } else { f32::MAX };
    by_w.min(by_h)
}

/// The size a picture is actually drawn at, fitted inside the box the author
/// named with its aspect kept — the same rule `scene.svg` follows, so the two
/// imports size the same way.
fn fitted_box(pixels: &Pixels, fit: (f32, f32)) -> (f32, f32) {
    let natural = (pixels.width.max(1) as f32, pixels.height.max(1) as f32);
    if fit.0 <= 0.0 && fit.1 <= 0.0 {
        return natural;
    }
    let by_w = if fit.0 > 0.0 {
        fit.0 / natural.0
    } else {
        f32::MAX
    };
    let by_h = if fit.1 > 0.0 {
        fit.1 / natural.1
    } else {
        f32::MAX
    };
    let scale = by_w.min(by_h);
    (natural.0 * scale, natural.1 * scale)
}

/// Read, parse and normalise an SVG file — once per path, for the life of the
/// process.
///
/// Cached for the same reason formula glyphs are: a 1,200-frame render must
/// not re-parse the same file 1,200 times. It also means editing the file
/// mid-session and re-rendering shows the old drawing.
fn svg_art(file: &str) -> Result<Arc<SvgArt>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<SvgArt>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(hit) = cache.lock().unwrap().get(file) {
        return Ok(hit.clone());
    }

    let source = std::fs::read_to_string(file)
        .map_err(|e| Error(format!("could not read the SVG {file:?}: {e}")))?;
    let imported = codimate_math::import_svg(&source).map_err(|e| {
        Error(match e {
            codimate_math::FormulaError::SvgText(m) => format!("{file:?}: {m}"),
            other => format!("could not read the SVG {file:?}: {other:?}"),
        })
    })?;

    let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
    let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
    for item in &imported {
        if let Some((x0, y0, x1, y1)) = item.path.bounding_box() {
            min_x = min_x.min(x0);
            min_y = min_y.min(y0);
            max_x = max_x.max(x1);
            max_y = max_y.max(y1);
        }
    }
    if min_x > max_x {
        return Err(Error(format!("{file:?} has nothing visible in it")));
    }

    let (cx, cy) = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
    let longest = (max_x - min_x).max(max_y - min_y).max(1e-6);

    let labels = codimate_math::import_svg_text(&source).map_err(|e| match e {
        codimate_math::FormulaError::SvgText(m) => Error(format!("{file:?}: {m}")),
        other => Error(format!("could not read the SVG {file:?}: {other:?}")),
    })?;

    let texts: Vec<SvgLabel> = labels
        .into_iter()
        .map(|l| SvgLabel {
            content: l.content,
            at: Vec2::new((l.x - cx) / longest, (l.y - cy) / longest),
            size: l.size / longest,
            fill: l.fill,
            align: if l.anchor == "middle" {
                TextAlign::Center
            } else {
                TextAlign::Left
            },
            from_right: l.anchor == "end",
        })
        .collect();

    let mut paths: Vec<(Path, Option<Color>)> = imported
        .iter()
        .map(|item| {
            (
                map_path(&item.path, |v| {
                    Vec2::new((v.x - cx) / longest, (v.y - cy) / longest)
                }),
                item.fill,
            )
        })
        .collect();

    // Left to right, so the pen draws across the picture. Document order was
    // the alternative and is better for artwork a person built stroke by
    // stroke, but most files come out of a tool where the saved order is
    // arbitrary — a Mermaid diagram emits arrows before boxes (ADR 0014).
    paths.sort_by(|a, b| {
        let key = |p: &Path| p.bounding_box().map(|(x0, ..)| x0).unwrap_or(0.0);
        key(&a.0)
            .partial_cmp(&key(&b.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let art = Arc::new(SvgArt {
        paths,
        texts,
        size: ((max_x - min_x) / longest, (max_y - min_y) / longest),
    });
    cache.lock().unwrap().insert(file.to_string(), art.clone());
    Ok(art)
}

// ============================================================
// The diff — motion is implied by identity
// ============================================================

/// How much of a many-outlined Item is showing, and whether a pen is drawing
/// it.
///
/// Shared by `formula` and `svg`, which are the same animation over different
/// artwork: a list of outlines, revealed left to right, optionally traced.
struct Reveal {
    /// The reveal edge at t=0 and t=1, in outline counts.
    edge: (f32, f32),
    /// Opacity at t=0 and t=1.
    fade: (f32, f32),
    /// Pen width. Zero means the outlines simply fade in.
    pen: f32,
}

impl Reveal {
    /// `pen` is passed rather than read off the Shape because the two kinds
    /// keep it in different fields: a formula in `w`, an imported SVG in
    /// `size`, because `w` there is already the fit box.
    fn of(before: &Shape, after: &Shape, count: usize, pen: f32) -> Self {
        // `r` is the revealed fraction. Scaled to an outline count it becomes a
        // moving edge: outlines behind it are solid, the one under it is
        // part-faded, the rest are not there yet — so a drawing assembles
        // itself instead of appearing all at once. The edge travels past the
        // last outline by the overlap so the tail finishes fading rather than
        // snapping on.
        let span = count as f32 + REVEAL_OVERLAP;
        Reveal {
            edge: (before.r * span, after.r * span),
            fade: (before.opacity, after.opacity),
            pen,
        }
    }
}

/// Draw outlines as one Item, swept by the reveal edge and optionally traced
/// by a pen.
fn revealed(
    outlines: &[(Path, Animated<Style>)],
    show: Reveal,
    pen_ink: Color,
    path: Animated<Vec2>,
) -> Vec<Primitive> {
    let (edge_from, edge_to) = show.edge;
    let (fade_from, fade_to) = show.fade;
    let pen = show.pen;

    // How far this outline has got, 0 to 1. The edge is eased; a linear sweep
    // starts and stops abruptly, which is most of what makes a reveal look
    // mechanical. Plain opacity stays linear, matching `tween`.
    let progress = move |index: f32, t: f32| {
        let edge = edge_from + (edge_to - edge_from) * ease_in_out(t);
        ((edge - index) / REVEAL_OVERLAP).clamp(0.0, 1.0)
    };
    let fade = move |t: f32| fade_from + (fade_to - fade_from) * t;

    let mut out = Vec::with_capacity(outlines.len() * if pen > 0.0 { 2 } else { 1 });
    for (index, (outline, look)) in outlines.iter().enumerate() {
        let index = index as f32;

        if pen <= 0.0 {
            out.push(
                Primitive::new(Geometry::path(outline.clone().into_animated()))
                    .pos(path.clone())
                    .style(look.clone())
                    .opacity(Animated::new(move |t| fade(t) * progress(index, t))),
            );
            continue;
        }

        // The solid outline, arriving only at the end of its own trace.
        out.push(
            Primitive::new(Geometry::path(outline.clone().into_animated()))
                .pos(path.clone())
                .style(look.clone())
                .opacity(Animated::new(move |t| {
                    let u = progress(index, t);
                    fade(t) * ((u - (1.0 - FILL_TAKEOVER)) / FILL_TAKEOVER).clamp(0.0, 1.0)
                })),
        );

        // The pen, drawing that outline and then lifting.
        let traced = flatten(outline);
        out.push(
            Primitive::new(Geometry::path(Animated::new(move |t| {
                trim(&traced, progress(index, t))
            })))
            .pos(path.clone())
            .style(
                Style::new()
                    .fill(Color::TRANSPARENT)
                    .stroke(pen, pen_ink)
                    .into_animated(),
            )
            .opacity(Animated::new(move |t| {
                let u = progress(index, t);
                let lifting = ((1.0 - u) / FILL_TAKEOVER).clamp(0.0, 1.0);
                fade(t) * lifting
            })),
        );
    }
    out
}

/// One Item becomes one Primitive — except a formula or an imported SVG, which
/// become one per outline. They share a position and opacity, so they move as
/// a unit.
fn primitives(before: &Shape, after: &Shape, rules: &[Rule]) -> Result<Vec<Primitive>> {
    if before.kind != after.kind {
        return Err(Error(format!(
            "item {:?} changed kind from {:?} to {:?} — an Item keeps one shape kind",
            after.item, before.kind, after.kind
        )));
    }

    let path = motion_path(before.anchor(), after.anchor(), rules, &after.item)?;
    let style = tween(before.style()?, after.style()?);
    let opacity = tween(before.opacity, after.opacity);

    // The rest of the Transform. Tweened like everything else, so a shape can
    // grow or turn between two moments without the author saying how.
    let scale = tween(
        Vec2::new(before.scale_x, before.scale_y),
        Vec2::new(after.scale_x, after.scale_y),
    );
    let spin = tween(before.rotate, after.rotate);
    let pivot = pivot_of(&after.pivot)?;

    if before.kind == "formula" {
        // `before`'s LaTeX, matching how text takes its value from the start of
        // the segment. An author who wants a formula read before it changes
        // emits a move and then a hold, exactly as they already do for text.
        let size = before.size;
        let ink = parse_color(&before.color)?;
        let outlines: Vec<(Path, Animated<Style>)> = formula_glyphs(&before.text)?
            .iter()
            .map(|g| {
                (
                    map_path(g, |v| Vec2::new(v.x * size, v.y * size)),
                    style.clone(),
                )
            })
            .collect();
        return Ok(revealed(
            &outlines,
            Reveal::of(before, after, outlines.len(), before.w.max(after.w)),
            ink,
            path,
        ));
    }

    // One Geometry, not an expansion — an image is a single thing. It is here
    // rather than in `geometry()` only because decoding can fail, and that
    // signature has no way to say so.
    if before.kind == "image" {
        let pixels = image_pixels(&before.text)?;
        let from = fitted_box(&pixels, (before.w, before.h));
        let to = fitted_box(&pixels, (after.w, after.h));
        return Ok(vec![Primitive::new(Geometry::Image {
            pixels,
            width: tween(from.0, to.0),
            height: tween(from.1, to.1),
        })
        .pos(path)
        .scale_xy(scale)
        .rotate(spin)
        .pivot(pivot)
        .style(style)
        .opacity(opacity)]);
    }

    if before.kind == "svg" {
        // Same shape as a formula: one Item, many outlines, revealed left to
        // right. The difference is that the artwork brought its own colours
        // (ADR 0014), so each outline carries a Style instead of sharing one.
        let art = svg_art(&before.text)?;
        let scale = fit_scale(art.size, (before.w, before.h));
        let authored = before.color.is_empty();
        let ink = if authored {
            Color::WHITE
        } else {
            parse_color(&before.color)?
        };

        let outlines: Vec<(Path, Animated<Style>)> = art
            .paths
            .iter()
            .map(|(p, own)| {
                let geometry = map_path(p, |v| Vec2::new(v.x * scale, v.y * scale));
                // An empty `color` means "as authored"; anything else is an
                // override the author asked for, and flattens the drawing.
                let look = match (authored, own) {
                    (true, Some(c)) => Style::new().fill(*c).into_animated(),
                    _ => style.clone(),
                };
                (geometry, look)
            })
            .collect();
        let pen = before.size.max(after.size);
        let mut out = revealed(
            &outlines,
            Reveal::of(before, after, outlines.len(), pen),
            ink,
            path.clone(),
        );

        // Labels are translated, not traced: each becomes a `Geometry::Text`
        // and is shaped at draw time by the same call that shapes every other
        // label in the frame. They cannot be pen-drawn, which is exactly true
        // of `scene.text` too, so it is consistent rather than surprising.
        let fade = tween(before.opacity, after.opacity);
        for label in &art.texts {
            let size = label.size * scale;
            // The Engine aligns left or centre; `text-anchor="end"` becomes a
            // left-aligned label starting one measured width earlier.
            let shift = if label.from_right {
                -codimate_render::measure_text(&label.content, size).0
            } else {
                0.0
            };
            let offset = Vec2::new(label.at.x * scale + shift, label.at.y * scale);
            let anchored = path.clone();
            let colour = match (authored, label.fill) {
                (true, Some(c)) => Style::new().fill(c).into_animated(),
                _ => style.clone(),
            };
            out.push(
                Primitive::new(Geometry::Text {
                    text: label.content.clone().into_animated(),
                    font_size: size.into_animated(),
                    align: label.align,
                })
                .pos(Animated::new(move |t| {
                    let base = anchored.resolve(t);
                    Vec2::new(base.x + offset.x, base.y + offset.y)
                }))
                .style(colour)
                .opacity(fade.clone()),
            );
        }
        return Ok(out);
    }

    Ok(vec![Primitive::new(before.geometry(after))
        .pos(path)
        .scale_xy(scale)
        .rotate(spin)
        .pivot(pivot)
        .style(style)
        .opacity(opacity)])
}

/// One segment's Scene: every Item that exists in `before` or `after`, tweened.
pub fn build_segment(before: &[Shape], after: &[Shape], rules: &[Rule]) -> Result<Scene> {
    let by_item = |s: &[Shape]| -> HashMap<String, Shape> {
        s.iter().map(|sh| (sh.item.clone(), sh.clone())).collect()
    };
    let (b, a) = (by_item(before), by_item(after));

    // Draw order: layer, then Item — stable and independent of dict ordering.
    let mut items: Vec<&Shape> = after.iter().chain(before.iter()).collect();
    items.sort_by(|x, y| (x.layer, &x.item).cmp(&(y.layer, &y.item)));
    items.dedup_by(|x, y| x.item == y.item);

    let mut scene = Scene::new();

    for shape in items {
        let prims = match (b.get(&shape.item), a.get(&shape.item)) {
            // Present in both — the ordinary case.
            (Some(from), Some(to)) => primitives(from, to, rules)?,

            // Entering — fade in where it lands, no travel.
            (None, Some(to)) => {
                let mut faded = to.clone();
                faded.opacity = 0.0;
                primitives(&faded, to, rules)?
            }

            // Exiting — fade out where it was.
            (Some(from), None) => {
                let mut faded = from.clone();
                faded.opacity = 0.0;
                primitives(from, &faded, rules)?
            }

            (None, None) => unreachable!("item came from one of the two lists"),
        };
        for prim in prims {
            scene = scene.add(prim);
        }
    }

    Ok(scene)
}

// ============================================================
// Playable
// ============================================================

pub struct Explanation {
    /// `(start_seconds, duration, scene)` — scenes hold their own local `t`.
    pub segments: Vec<(f32, f32, Scene)>,
    pub total: f32,
}

impl Playable for Explanation {
    fn name(&self) -> &str {
        "explanation"
    }

    fn duration(&self) -> f32 {
        self.total
    }

    fn resolve(&self, t: f32) -> ConcreteScene {
        let seconds = t.clamp(0.0, 1.0) * self.total;

        for (start, duration, scene) in &self.segments {
            if seconds < start + duration {
                return scene.resolve(((seconds - start) / duration).clamp(0.0, 1.0));
            }
        }

        match self.segments.last() {
            Some((_, _, scene)) => scene.resolve(1.0),
            None => ConcreteScene { children: vec![] },
        }
    }
}

// ============================================================
// ============================================================
// The camera
// ============================================================

/// What to look at. Aimed by name, never by coordinate (ADR 0009).
#[derive(Clone, Debug, Default)]
pub struct Focus {
    /// Items to frame. Empty means the whole canvas.
    pub names: Vec<String>,
    /// Breathing room around them, in canvas units.
    pub pad: f32,
    /// Never frame tighter than this, so a small shape is not magnified into
    /// abstraction. A statement about the picture, not about the arithmetic.
    pub min_size: f32,
    /// Shapes that ignore the camera, by name — the overlay.
    pub fixed: Vec<String>,
}

/// How far a shape reaches from its anchor, as `(left, top, right, bottom)`.
fn bounds(shape: &Shape) -> (f32, f32, f32, f32) {
    let (w, h) = match shape.kind.as_str() {
        "circle" => (shape.r * 2.0, shape.r * 2.0),
        "text" => codimate_render::measure_text(&shape.text, shape.size),
        "formula" => formula_size(&shape.text, shape.size).unwrap_or((0.0, 0.0)),
        // The fitted box, not the box asked for: a wide drawing in a square
        // `size` does not fill the square, and framing the square would leave
        // the camera looking at empty air above and below.
        "image" => image_pixels(&shape.text)
            .map(|p| fitted_box(&p, (shape.w, shape.h)))
            .unwrap_or((0.0, 0.0)),
        "svg" => svg_art(&shape.text)
            .map(|art| {
                let s = fit_scale(art.size, (shape.w, shape.h));
                (art.size.0 * s, art.size.1 * s)
            })
            .unwrap_or((0.0, 0.0)),
        // A Catmull-Rom curve can bulge a little past its samples between
        // them, but never far, and framing by the samples is what an author
        // means by "frame this curve".
        "polygon" | "curve" => {
            let xs: Vec<f32> = shape.points.iter().step_by(2).copied().collect();
            let ys: Vec<f32> = shape.points.iter().skip(1).step_by(2).copied().collect();
            if xs.is_empty() {
                return (shape.x, shape.y, shape.x, shape.y);
            }
            return (
                xs.iter().cloned().fold(f32::MAX, f32::min),
                ys.iter().cloned().fold(f32::MAX, f32::min),
                xs.iter().cloned().fold(f32::MIN, f32::max),
                ys.iter().cloned().fold(f32::MIN, f32::max),
            );
        }
        "line" => {
            let (x0, x1) = (shape.x.min(shape.x2), shape.x.max(shape.x2));
            let (y0, y1) = (shape.y.min(shape.y2), shape.y.max(shape.y2));
            return (x0, y0, x1, y1);
        }
        _ => (shape.w, shape.h),
    };
    (
        shape.x - w / 2.0,
        shape.y - h / 2.0,
        shape.x + w / 2.0,
        shape.y + h / 2.0,
    )
}

/// Where the camera sits and how far in, for one Scene.
///
/// Returns the centre to look at and the zoom that frames it. Zoom is capped
/// by `min_size`, because `focus` on a three-pixel dot is a two-hundred-times
/// magnification of nothing.
fn aim(shapes: &[Shape], focus: &Focus, viewport: (f32, f32)) -> Result<(f32, f32, f32)> {
    if focus.names.is_empty() {
        return Ok((viewport.0 / 2.0, viewport.1 / 2.0, 1.0));
    }

    let (mut x0, mut y0) = (f32::MAX, f32::MAX);
    let (mut x1, mut y1) = (f32::MIN, f32::MIN);
    let mut found = 0;
    for shape in shapes.iter().filter(|s| focus.names.contains(&s.item)) {
        let (a, b, c, d) = bounds(shape);
        x0 = x0.min(a);
        y0 = y0.min(b);
        x1 = x1.max(c);
        y1 = y1.max(d);
        found += 1;
    }

    if found == 0 {
        // A camera aimed at nothing is the failure this design exists to
        // prevent, so it is an error rather than a silent wide shot.
        return Err(Error(format!(
            "focus({}) — no shape by that name in this scene",
            focus.names.join(", ")
        )));
    }

    let want_w = (x1 - x0 + focus.pad * 2.0).max(focus.min_size);
    let want_h = (y1 - y0 + focus.pad * 2.0).max(focus.min_size);
    // Fit the whole box: the tighter of the two axes decides.
    let zoom = (viewport.0 / want_w).min(viewport.1 / want_h).max(1.0);

    Ok(((x0 + x1) / 2.0, (y0 + y1) / 2.0, zoom))
}

/// Move a Scene's shapes into the camera's view.
///
/// Applied to the flat payload rather than to built primitives, which is only
/// possible because the payload is a flat union: one pass over a handful of
/// floats covers every kind, sizes and positions alike.
///
/// Each Scene is framed with its own camera *before* the diff sees it, so a
/// camera that moves between two moments is simply two sets of coordinates
/// that differ — and the existing tween animates it. Camera movement needs no
/// machinery of its own.
fn framed(shapes: Vec<Shape>, focus: Option<&Focus>, viewport: (f32, f32)) -> Result<Vec<Shape>> {
    let Some(focus) = focus else {
        return Ok(shapes);
    };
    let (cx, cy, zoom) = aim(&shapes, focus, viewport)?;
    let (sx, sy) = (viewport.0 / 2.0, viewport.1 / 2.0);

    Ok(shapes
        .into_iter()
        .map(|mut s| {
            if focus.fixed.iter().any(|f| s.item.starts_with(f.as_str())) {
                return s; // the overlay: pinned to the screen
            }
            s.x = (s.x - cx) * zoom + sx;
            s.y = (s.y - cy) * zoom + sy;
            s.x2 = (s.x2 - cx) * zoom + sx;
            s.y2 = (s.y2 - cy) * zoom + sy;
            for (i, value) in s.points.iter_mut().enumerate() {
                *value = if i % 2 == 0 {
                    (*value - cx) * zoom + sx
                } else {
                    (*value - cy) * zoom + sy
                };
            }
            s.w *= zoom;
            s.h *= zoom;
            s.r *= zoom;
            s.size *= zoom;
            s
        })
        .collect())
}

// ============================================================
// The entry point
// ============================================================

/// Build a playable explanation from the Scenes of every Trace Event.
///
/// `scenes` is one longer than `durations`: the opening Scene, then one per
/// event. Zero-length segments are dropped rather than producing an empty time
/// interval.
///
/// This is the whole surface a frontend needs. Everything else in this crate
/// is how it is done.
pub fn explanation(
    scenes: Vec<Vec<Shape>>,
    cameras: &[Option<Focus>],
    rules: &[Rule],
    durations: &[f32],
    viewport: (f32, f32),
) -> Result<Explanation> {
    if scenes.len() != durations.len() + 1 {
        return Err(Error(format!(
            "got {} scenes for {} durations — expected {}",
            scenes.len(),
            durations.len(),
            durations.len() + 1
        )));
    }

    for shape in scenes.iter().flatten() {
        if !KINDS.contains(&shape.kind.as_str()) {
            return Err(Error(format!(
                "unknown kind {:?} on item {:?} — Codimate draws {}",
                shape.kind,
                shape.item,
                KINDS.join(", ")
            )));
        }
    }

    // Scenes are framed as the diff walks them, not all at once. The diff
    // never looks further than one step back, so two framed Scenes are alive
    // at a time rather than one per event — on a Scene of a few thousand
    // shapes that is the difference between holding the whole payload again
    // and holding a pair of them.
    //
    // `scenes` is consumed for the same reason: framing used to clone every
    // Scene while the caller's copy was still alive, so the payload existed
    // twice before a single frame was drawn.
    let mut scenes = scenes.into_iter();
    let mut prev = framed(
        scenes.next().expect("scene count checked above"),
        cameras.first().and_then(|c| c.as_ref()),
        viewport,
    )?;

    let mut segments = Vec::new();
    let mut cursor = 0.0f32;

    for (i, duration) in durations.iter().enumerate() {
        // Framed even when the segment is dropped: a zero-length event still
        // advances which Scene the next segment diffs against.
        let next = framed(
            scenes.next().expect("scene count checked above"),
            cameras.get(i + 1).and_then(|c| c.as_ref()),
            viewport,
        )?;
        if *duration > 0.0 {
            segments.push((cursor, *duration, build_segment(&prev, &next, rules)?));
            cursor += duration;
        }
        prev = next;
    }

    if segments.is_empty() {
        return Err(Error(
            "nothing to render — no events with a positive duration".into(),
        ));
    }

    Ok(Explanation {
        segments,
        total: cursor,
    })
}

/// How wide and tall a typeset formula is at `size`.
///
/// Lives here rather than in a frontend because it reads the same cached glyph
/// outlines the renderer draws, and because framing a formula — the thing a
/// camera would need — is scene geometry, not marshalling.
pub fn formula_size(latex: &str, size: f32) -> Result<(f32, f32)> {
    let glyphs = formula_glyphs(latex)?;
    let (mut x0, mut y0) = (f32::MAX, f32::MAX);
    let (mut x1, mut y1) = (f32::MIN, f32::MIN);
    for path in glyphs.iter() {
        if let Some((a, b, c, d)) = path.bounding_box() {
            x0 = x0.min(a);
            y0 = y0.min(b);
            x1 = x1.max(c);
            y1 = y1.max(d);
        }
    }
    if x0 > x1 {
        return Ok((0.0, 0.0));
    }
    // The cache holds glyphs centred and in em units, so scaling by `size` is
    // the same arithmetic the renderer does.
    Ok(((x1 - x0) * size, (y1 - y0) * size))
}

/// The easing applied between two moments.
pub fn ease(t: f32) -> f32 {
    ease_in_out(t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    /// Two paths only tween if they have the same structure, so a rect must
    /// produce the same segments whether or not its corners are rounded —
    /// otherwise animating `radius` from 0 would pop instead of ease.
    #[test]
    fn rounding_the_corners_keeps_the_path_structure() {
        let mut square = Shape {
            item: "b".into(),
            kind: "rect".into(),
            x: 0.0,
            y: 0.0,
            x2: 0.0,
            y2: 0.0,
            w: 100.0,
            h: 60.0,
            r: 0.0,
            color: "white".into(),
            text: String::new(),
            size: 0.0,
            layer: 0,
            opacity: 1.0,
            ..Default::default()
        };
        let flat = round_rect_path(&square).segments.len();
        square.r = 12.0;
        let round = round_rect_path(&square).segments.len();
        assert_eq!(flat, round, "square and rounded rects must tween");

        // A radius past half the short side is a pill, not a broken path.
        square.r = 9_999.0;
        let pill = round_rect_path(&square);
        assert_eq!(pill.segments.len(), flat);
        for seg in &pill.segments {
            for v in [seg_start(seg)] {
                assert!(
                    v.x.abs() <= 50.001 && v.y.abs() <= 30.001,
                    "{v:?} escaped the box"
                );
            }
        }
    }

    /// The pen has to draw a steady fraction of the outline, not a steady
    /// fraction of the *segments* — a glyph's segments vary wildly in length,
    /// so counting them would make the pen lurch.
    #[test]
    fn the_pen_draws_by_length_not_by_segment() {
        // A 10x10 square: four sides, perimeter 40.
        let square = Path {
            segments: vec![
                Segment::MoveTo(Vec2::new(0.0, 0.0)),
                Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)),
                Segment::Line(Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)),
                Segment::Line(Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)),
                Segment::Line(Vec2::new(0.0, 10.0), Vec2::new(0.0, 0.0)),
            ],
            closed: true,
        };
        let traced = flatten(&square);
        assert!((traced.length - 40.0).abs() < 1e-3, "{}", traced.length);

        let drawn = |f: f32| {
            trim(&traced, f)
                .segments
                .iter()
                .filter_map(|s| match *s {
                    Segment::Line(a, b) => Some(((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()),
                    _ => None,
                })
                .sum::<f32>()
        };

        assert!(drawn(0.0) < 1e-3, "nothing is drawn at the start");
        assert!((drawn(0.25) - 10.0).abs() < 1e-3, "{}", drawn(0.25));
        // Half way is mid-side, not on a corner — the cut lands inside a
        // segment, which is what stops the line growing a side at a time.
        assert!((drawn(0.5) - 20.0).abs() < 1e-3, "{}", drawn(0.5));
        assert!((drawn(1.0) - 40.0).abs() < 1e-3, "{}", drawn(1.0));
    }

    fn seg_start(s: &Segment) -> Vec2 {
        match *s {
            Segment::MoveTo(a) | Segment::Line(a, _) | Segment::Quad(a, _, _) => a,
            Segment::Cubic(a, _, _, _) => a,
            Segment::Close => Vec2::new(0.0, 0.0),
        }
    }

    use super::*;

    #[test]
    fn glob_matches_the_patterns_authors_actually_write() {
        assert!(glob_match("*", "bar_3"));
        assert!(glob_match("bar_*", "bar_3"));
        assert!(!glob_match("bar_*", "label_3"));
        assert!(glob_match("*_3", "bar_3"));
        assert!(glob_match("bar_?", "bar_3"));
        assert!(!glob_match("bar_?", "bar_33"));
        assert!(glob_match("bar_3", "bar_3"));
    }

    #[test]
    fn lift_carry_drop_actually_leaves_the_ground() {
        let a = Vec2::new(100.0, 455.0);
        let b = Vec2::new(230.0, 455.0);
        let rules = vec![(
            "bar_*".to_string(),
            "lift_carry_drop".to_string(),
            HashMap::from([("clearance".to_string(), 90.0)]),
        )];

        let path = motion_path(a, b, &rules, "bar_3").unwrap();
        let mid = path.resolve(0.5);

        assert!(
            mid.y <= 455.0 - 89.0,
            "carry phase should sit a full clearance above the ends, got y={}",
            mid.y
        );
        assert!((path.resolve(0.0).y - 455.0).abs() < 0.01);
        assert!((path.resolve(1.0).y - 455.0).abs() < 0.01);

        // A bar the event never touched must stay put, even though `bar_*`
        // matches it — otherwise every bar arcs in place on every swap.
        let still = motion_path(a, a, &rules, "bar_4").unwrap();
        assert_eq!(still.resolve(0.5), a);
        assert_eq!(still.resolve(0.25), a);
    }

    #[test]
    fn linear_holds_a_constant_speed() {
        let (a, b) = (Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0));
        let rules = vec![("*".to_string(), "linear".to_string(), HashMap::new())];
        let path = motion_path(a, b, &rules, "wheel").unwrap();

        // Equal steps in t must give equal steps in distance. Easing would
        // make the middle step several times the first — which judders once
        // per event for anything mid-journey, like a turning wheel.
        let steps: Vec<f32> = (0..10)
            .map(|i| path.resolve((i + 1) as f32 / 10.0).x - path.resolve(i as f32 / 10.0).x)
            .collect();
        let lo = steps.iter().cloned().fold(f32::MAX, f32::min);
        let hi = steps.iter().cloned().fold(0.0, f32::max);
        assert!(hi / lo < 1.01, "linear must not accelerate: {steps:?}");

        // The default path does ease, and should keep doing so.
        let eased = motion_path(a, b, &[], "wheel").unwrap();
        let mid = eased.resolve(0.55).x - eased.resolve(0.45).x;
        let edge = eased.resolve(0.1).x - eased.resolve(0.0).x;
        assert!(mid > edge * 1.5, "straight should still ease in and out");
    }

    #[test]
    fn fall_accelerates_downwards_but_drifts_evenly() {
        let (a, b) = (Vec2::new(0.0, 0.0), Vec2::new(100.0, 400.0));
        let rules = vec![("*".to_string(), "fall".to_string(), HashMap::new())];
        let path = motion_path(a, b, &rules, "ball").unwrap();

        let step = |lo: f32, hi: f32| {
            let (p, q) = (path.resolve(lo), path.resolve(hi));
            (q.x - p.x, q.y - p.y)
        };
        let (first_x, first_y) = step(0.0, 0.1);
        let (last_x, last_y) = step(0.9, 1.0);

        assert!(
            (first_x - last_x).abs() < 0.01,
            "sideways drift must not change"
        );
        assert!(
            last_y > first_y * 8.0,
            "downward speed must build: {first_y} -> {last_y}"
        );
        assert!(
            (path.resolve(1.0).y - 400.0).abs() < 0.01,
            "and still arrive"
        );
    }

    #[test]
    fn a_swap_makes_both_bars_travel() {
        let bar = |item: &str, x: f32| Shape {
            item: item.into(),
            kind: "rect".into(),
            x,
            y: 300.0,
            x2: 0.0,
            y2: 0.0,
            w: 60.0,
            h: 100.0,
            r: 0.0,
            color: "blue".into(),
            text: String::new(),
            size: 16.0,
            layer: 0,
            opacity: 1.0,
            ..Default::default()
        };

        let before = vec![bar("bar_3", 100.0), bar("bar_1", 180.0)];
        let after = vec![bar("bar_3", 180.0), bar("bar_1", 100.0)];

        // The rule must survive all the way into the resolved Scene.
        let rules = vec![(
            "bar_*".to_string(),
            "lift_carry_drop".to_string(),
            HashMap::from([("clearance".to_string(), 90.0)]),
        )];
        let lifted = build_segment(&before, &after, &rules).unwrap();
        let carry = lifted.resolve(0.5);
        for child in &carry.children {
            if let codimate_core::ConcreteNode::Primitive(p) = child {
                assert!(
                    p.transform.pos.y <= 300.0 - 89.0,
                    "bar should be lifted during carry, got y={}",
                    p.transform.pos.y
                );
            }
        }

        let scene = build_segment(&before, &after, &[]).unwrap();

        // Midway, both bars sit between their endpoints — they crossed.
        let mid = scene.resolve(0.5);
        assert_eq!(mid.children.len(), 2);

        let start = scene.resolve(0.0);
        let end = scene.resolve(1.0);
        assert_ne!(start, end, "a swap must change the picture");
        assert_ne!(start, mid, "a swap must not snap at t=0");
    }

    fn curve_of(points: &[(f32, f32)], closed: bool) -> Shape {
        let flat: Vec<f32> = points.iter().flat_map(|(x, y)| [*x, *y]).collect();
        let xs: Vec<f32> = points.iter().map(|p| p.0).collect();
        let ys: Vec<f32> = points.iter().map(|p| p.1).collect();
        Shape {
            item: "c".into(),
            kind: "curve".into(),
            x: (xs.iter().cloned().fold(f32::MAX, f32::min)
                + xs.iter().cloned().fold(f32::MIN, f32::max))
                / 2.0,
            y: (ys.iter().cloned().fold(f32::MAX, f32::min)
                + ys.iter().cloned().fold(f32::MIN, f32::max))
                / 2.0,
            points: flat,
            w: if closed { 1.0 } else { 0.0 },
            color: "white".into(),
            opacity: 1.0,
            ..Default::default()
        }
    }

    /// The whole promise of `curve` is that the line goes through the samples
    /// you gave it (ADR 0012). Control points are derived, so the only thing
    /// worth asserting is that the derivation did not move the samples.
    #[test]
    fn a_curve_passes_through_every_point_it_was_given() {
        let points = [(0.0, 0.0), (100.0, -50.0), (200.0, 0.0), (300.0, 80.0)];
        let shape = curve_of(&points, false);
        let path = curve_path(&shape);

        let visited: Vec<Vec2> = path
            .segments
            .iter()
            .filter_map(|s| match *s {
                Segment::Cubic(from, _, _, to) => Some([from, to]),
                _ => None,
            })
            .flatten()
            .collect();

        for (px, py) in points {
            let want = Vec2::new(px - shape.x, py - shape.y);
            assert!(
                visited
                    .iter()
                    .any(|v| (v.x - want.x).abs() < 0.001 && (v.y - want.y).abs() < 0.001),
                "the curve never reaches {want:?} — visited {visited:?}"
            );
        }
        assert_eq!(
            path.segments.len(),
            points.len(),
            "one MoveTo, then a span per gap"
        );
    }

    /// Open and closed differ by one span: the closed one comes back.
    #[test]
    fn a_closed_curve_joins_its_ends() {
        let points = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
        let open = curve_path(&curve_of(&points, false));
        let closed = curve_path(&curve_of(&points, true));

        assert_eq!(closed.segments.len(), open.segments.len() + 1);
        assert!(closed.closed && !open.closed);

        // The last span of a closed curve returns to where the first began.
        let (Some(Segment::MoveTo(start)), Some(Segment::Cubic(_, _, _, end))) =
            (closed.segments.first(), closed.segments.last())
        else {
            panic!("a closed curve should start with a MoveTo and end with a Cubic");
        };
        assert!((start.x - end.x).abs() < 0.001 && (start.y - end.y).abs() < 0.001);
    }

    /// Two points is a straight line, so `curve` is never wrong to reach for.
    #[test]
    fn a_curve_through_two_points_is_straight() {
        let path = curve_path(&curve_of(&[(0.0, 0.0), (100.0, 100.0)], false));
        let Some(Segment::Cubic(from, c1, c2, to)) = path.segments.get(1) else {
            panic!("expected one cubic span");
        };
        // Every control point sits on the chord, so the cubic is a line.
        for c in [c1, c2] {
            let t = (c.x - from.x) / (to.x - from.x);
            let on_chord = from.y + t * (to.y - from.y);
            assert!((c.y - on_chord).abs() < 0.001, "{c:?} bulges off the chord");
        }
    }

    fn box_at(item: &str, x: f32, y: f32, w: f32, h: f32) -> Shape {
        Shape {
            item: item.into(),
            kind: "rect".into(),
            x,
            y,
            w,
            h,
            color: "white".into(),
            opacity: 1.0,
            ..Default::default()
        }
    }

    /// The camera is aimed by name, so it has to find the thing and frame it —
    /// centre on it, and zoom until it fills the frame.
    #[test]
    fn focus_frames_the_shape_it_names() {
        let shapes = vec![
            box_at("a", 100.0, 100.0, 40.0, 40.0),
            box_at("b", 900.0, 600.0, 40.0, 40.0),
        ];
        let focus = Focus {
            names: vec!["b".into()],
            pad: 0.0,
            min_size: 0.0,
            fixed: vec![],
        };

        let (cx, cy, zoom) = aim(&shapes, &focus, (1280.0, 720.0)).unwrap();
        assert_eq!(
            (cx, cy),
            (900.0, 600.0),
            "centred on what it was told to look at"
        );
        assert!(zoom > 1.0, "zoomed in, not out: {zoom}");

        // and the framed scene puts that shape in the middle of the screen
        let framed = framed(shapes, Some(&focus), (1280.0, 720.0)).unwrap();
        let b = framed.iter().find(|s| s.item == "b").unwrap();
        assert!(
            (b.x - 640.0).abs() < 0.01 && (b.y - 360.0).abs() < 0.01,
            "{b:?}"
        );
    }

    /// A camera pointing at whitespace is the failure aiming-by-name exists to
    /// prevent, so a name that is not in the Scene is an error, not a wide shot.
    #[test]
    fn focus_on_a_name_that_is_not_there_is_an_error() {
        let shapes = vec![box_at("a", 100.0, 100.0, 40.0, 40.0)];
        let focus = Focus {
            names: vec!["typo".into()],
            pad: 0.0,
            min_size: 0.0,
            fixed: vec![],
        };
        assert!(aim(&shapes, &focus, (1280.0, 720.0)).is_err());
    }

    /// Focusing a three-pixel dot would otherwise magnify it two hundred times.
    #[test]
    fn a_tiny_shape_does_not_fill_the_screen() {
        let shapes = vec![box_at("dot", 640.0, 360.0, 3.0, 3.0)];
        let loose = Focus {
            names: vec!["dot".into()],
            pad: 0.0,
            min_size: 240.0,
            fixed: vec![],
        };
        let (_, _, zoom) = aim(&shapes, &loose, (1280.0, 720.0)).unwrap();
        assert!(
            zoom <= 720.0 / 240.0 + 0.01,
            "clamped by min_size, got {zoom}"
        );
    }

    /// The overlay is the reason `focus` is usable at all: without it a caption
    /// is pushed off the frame the first time the camera moves.
    #[test]
    fn the_overlay_stays_where_it_was_put() {
        let shapes = vec![
            box_at("thing", 200.0, 200.0, 40.0, 40.0),
            box_at("_overlay/title", 640.0, 52.0, 300.0, 30.0),
        ];
        let focus = Focus {
            names: vec!["thing".into()],
            pad: 0.0,
            min_size: 0.0,
            fixed: vec!["_overlay".into()],
        };

        let framed = framed(shapes, Some(&focus), (1280.0, 720.0)).unwrap();
        let title = framed
            .iter()
            .find(|s| s.item.starts_with("_overlay"))
            .unwrap();
        assert_eq!(
            (title.x, title.y),
            (640.0, 52.0),
            "the camera moved the title"
        );
        assert_eq!(
            (title.w, title.h),
            (300.0, 30.0),
            "the camera resized the title"
        );
    }
}
