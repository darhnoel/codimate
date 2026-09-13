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
    scene::AnchorKind, scene::Transformable, tween, Animated, Color, ConcreteScene, Geometry, IntoAnimated, Path,
    Primitive, Scene, Segment, Style, TextAlign, Vec2,
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
pub const KINDS: [&str; 6] = ["rect", "circle", "text", "line", "formula", "polygon"];

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

            // A formula is many glyph outlines, so it cannot be one Geometry.
            // `primitives()` expands it; this arm is never reached.
            "formula" => Geometry::rect(0.0.into_animated(), 0.0.into_animated()),

            // Square corners stay a real Rect — the common case keeps the
            // cheaper primitive and the renderer's own rectangle path.
            _ if self.r > 0.0 || other.r > 0.0 => {
                Geometry::path(tween(round_rect_path(self), round_rect_path(other)))
            }

            _ => Geometry::rect(tween(self.w, other.w), tween(self.h, other.h)),
        }
    }

    fn style(&self) -> Result<Style> {
        let fill = parse_color(&self.color)?;

        // A line is drawn, not filled — `w` is its stroke width, and it has no
        // separate edge.
        if self.kind == "line" {
            return Ok(Style::new().fill(Color::TRANSPARENT).stroke(self.w, fill));
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
        return Err(Error(format!(
            "{latex:?} typeset to nothing visible"
        )));
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
        key(a).partial_cmp(&key(b)).unwrap_or(std::cmp::Ordering::Equal)
    });

    let shared = Arc::new(centred);
    cache
        .lock()
        .unwrap()
        .insert(latex.to_string(), shared.clone());
    Ok(shared)
}

// ============================================================
// The diff — motion is implied by identity
// ============================================================

/// One Item becomes one Primitive — except a formula, which becomes one per
/// glyph. They share a position, style and opacity, so they move as a unit.
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
        let glyphs = formula_glyphs(&before.text)?;

        // `r` is the revealed fraction. Scaled to a glyph count, it becomes a
        // moving edge: glyphs behind it are solid, the one under it is
        // part-faded, the rest are not there yet. So an equation can assemble
        // itself term by term instead of appearing all at once.
        // The edge travels past the last glyph by the overlap, so the tail
        // finishes fading instead of snapping on at the end.
        let span = glyphs.len() as f32 + REVEAL_OVERLAP;
        let (edge_from, edge_to) = (before.r * span, after.r * span);
        let (fade_from, fade_to) = (before.opacity, after.opacity);

        // How far this glyph has got, 0 to 1. The edge is eased; a linear
        // sweep starts and stops abruptly, which is most of what makes a
        // reveal look mechanical. Plain opacity stays linear, matching `tween`.
        let progress = move |index: f32, t: f32| {
            let edge = edge_from + (edge_to - edge_from) * ease_in_out(t);
            ((edge - index) / REVEAL_OVERLAP).clamp(0.0, 1.0)
        };
        let fade = move |t: f32| fade_from + (fade_to - fade_from) * t;

        // `w` is the pen. Zero means no pen: glyphs simply fade in, which is
        // cheaper and right for a formula that is just arriving.
        let pen = before.w.max(after.w);
        let ink = parse_color(&before.color)?;

        let mut out = Vec::with_capacity(glyphs.len() * if pen > 0.0 { 2 } else { 1 });
        for (index, glyph) in glyphs.iter().enumerate() {
            let scaled = map_path(glyph, |v| Vec2::new(v.x * size, v.y * size));
            let index = index as f32;

            if pen <= 0.0 {
                out.push(
                    Primitive::new(Geometry::path(scaled.into_animated()))
                        .pos(path.clone())
                        .style(style.clone())
                        .opacity(Animated::new(move |t| fade(t) * progress(index, t))),
                );
                continue;
            }

            // The solid glyph, arriving only at the end of its own trace.
            out.push(
                Primitive::new(Geometry::path(scaled.clone().into_animated()))
                    .pos(path.clone())
                    .style(style.clone())
                    .opacity(Animated::new(move |t| {
                        let u = progress(index, t);
                        fade(t) * ((u - (1.0 - FILL_TAKEOVER)) / FILL_TAKEOVER).clamp(0.0, 1.0)
                    })),
            );

            // The pen, drawing that glyph's outline and then lifting.
            let traced = flatten(&scaled);
            out.push(
                Primitive::new(Geometry::path(Animated::new(move |t| {
                    trim(&traced, progress(index, t))
                })))
                .pos(path.clone())
                .style(
                    Style::new()
                        .fill(Color::TRANSPARENT)
                        .stroke(pen, ink)
                        .into_animated(),
                )
                .opacity(Animated::new(move |t| {
                    let u = progress(index, t);
                    let lifting = ((1.0 - u) / FILL_TAKEOVER).clamp(0.0, 1.0);
                    fade(t) * lifting
                })),
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
        "polygon" => {
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
    scenes: &[Vec<Shape>],
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

    // Frame every Scene before diffing any of them, so the diff only ever sees
    // screen coordinates and knows nothing about cameras.
    let framed: Vec<Vec<Shape>> = scenes
        .iter()
        .enumerate()
        .map(|(i, scene)| framed(scene.clone(), cameras.get(i).and_then(|c| c.as_ref()), viewport))
        .collect::<Result<_>>()?;

    let mut segments = Vec::new();
    let mut cursor = 0.0f32;

    for (i, duration) in durations.iter().enumerate() {
        if *duration <= 0.0 {
            continue;
        }
        segments.push((cursor, *duration, build_segment(&framed[i], &framed[i + 1], rules)?));
        cursor += duration;
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
            x: 0.0, y: 0.0, x2: 0.0, y2: 0.0,
            w: 100.0, h: 60.0, r: 0.0,
            color: "white".into(), text: String::new(),
            size: 0.0, layer: 0, opacity: 1.0,
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
                assert!(v.x.abs() <= 50.001 && v.y.abs() <= 30.001, "{v:?} escaped the box");
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
                    Segment::Line(a, b) => {
                        Some(((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt())
                    }
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

        assert!((first_x - last_x).abs() < 0.01, "sideways drift must not change");
        assert!(last_y > first_y * 8.0, "downward speed must build: {first_y} -> {last_y}");
        assert!((path.resolve(1.0).y - 400.0).abs() < 0.01, "and still arrive");
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

    fn box_at(item: &str, x: f32, y: f32, w: f32, h: f32) -> Shape {
        Shape {
            item: item.into(),
            kind: "rect".into(),
            x, y, w, h,
            color: "white".into(),
            opacity: 1.0,
            ..Default::default()
        }
    }

    /// The camera is aimed by name, so it has to find the thing and frame it —
    /// centre on it, and zoom until it fills the frame.
    #[test]
    fn focus_frames_the_shape_it_names() {
        let shapes = vec![box_at("a", 100.0, 100.0, 40.0, 40.0),
                          box_at("b", 900.0, 600.0, 40.0, 40.0)];
        let focus = Focus { names: vec!["b".into()], pad: 0.0, min_size: 0.0, fixed: vec![] };

        let (cx, cy, zoom) = aim(&shapes, &focus, (1280.0, 720.0)).unwrap();
        assert_eq!((cx, cy), (900.0, 600.0), "centred on what it was told to look at");
        assert!(zoom > 1.0, "zoomed in, not out: {zoom}");

        // and the framed scene puts that shape in the middle of the screen
        let framed = framed(shapes, Some(&focus), (1280.0, 720.0)).unwrap();
        let b = framed.iter().find(|s| s.item == "b").unwrap();
        assert!((b.x - 640.0).abs() < 0.01 && (b.y - 360.0).abs() < 0.01, "{b:?}");
    }

    /// A camera pointing at whitespace is the failure aiming-by-name exists to
    /// prevent, so a name that is not in the Scene is an error, not a wide shot.
    #[test]
    fn focus_on_a_name_that_is_not_there_is_an_error() {
        let shapes = vec![box_at("a", 100.0, 100.0, 40.0, 40.0)];
        let focus = Focus { names: vec!["typo".into()], pad: 0.0, min_size: 0.0, fixed: vec![] };
        assert!(aim(&shapes, &focus, (1280.0, 720.0)).is_err());
    }

    /// Focusing a three-pixel dot would otherwise magnify it two hundred times.
    #[test]
    fn a_tiny_shape_does_not_fill_the_screen() {
        let shapes = vec![box_at("dot", 640.0, 360.0, 3.0, 3.0)];
        let loose = Focus { names: vec!["dot".into()], pad: 0.0, min_size: 240.0, fixed: vec![] };
        let (_, _, zoom) = aim(&shapes, &loose, (1280.0, 720.0)).unwrap();
        assert!(zoom <= 720.0 / 240.0 + 0.01, "clamped by min_size, got {zoom}");
    }

    /// The overlay is the reason `focus` is usable at all: without it a caption
    /// is pushed off the frame the first time the camera moves.
    #[test]
    fn the_overlay_stays_where_it_was_put() {
        let shapes = vec![box_at("thing", 200.0, 200.0, 40.0, 40.0),
                          box_at("_overlay/title", 640.0, 52.0, 300.0, 30.0)];
        let focus = Focus {
            names: vec!["thing".into()],
            pad: 0.0,
            min_size: 0.0,
            fixed: vec!["_overlay".into()],
        };

        let framed = framed(shapes, Some(&focus), (1280.0, 720.0)).unwrap();
        let title = framed.iter().find(|s| s.item.starts_with("_overlay")).unwrap();
        assert_eq!((title.x, title.y), (640.0, 52.0), "the camera moved the title");
        assert_eq!((title.w, title.h), (300.0, 30.0), "the camera resized the title");
    }
}
