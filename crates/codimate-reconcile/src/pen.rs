//! Drawing on: an outline traced to a polyline and cut to a fraction of its length.

use crate::{Path, Segment, Vec2};

/// Typst's default text size in SVG pixels (11pt, at 4/3 px per pt).
///
/// `codimate_math::formula` typesets at Typst's default size, so this is what
/// turns an author's `size=` — which must mean the same thing it means for
/// `text` — into a scale factor. Measured against it: a cap-height `H` comes
/// back 10.02px, or 0.68 em, which is the usual cap-height ratio.
pub(crate) const FORMULA_EM: f32 = 11.0 * 4.0 / 3.0;

/// How many glyphs are fading at once during a `reveal`.
///
/// At 1.0 each glyph gets exactly its own slice of the time and none of its
/// neighbour's, which marches rather than flows — with thirty glyphs over two
/// seconds that is 66ms each, and it reads as popping. Overlapping them gives
/// each glyph four times as long and keeps several in flight, which is what
/// Manim calls a lag ratio.
pub(crate) const REVEAL_OVERLAP: f32 = 4.0;

/// The last part of a glyph's reveal, during which the drawn outline fades out
/// and the solid glyph fades in. Manim calls the whole move DrawBorderThenFill;
/// this is the "then".
pub(crate) const FILL_TAKEOVER: f32 = 0.35;

/// A glyph outline flattened to a polyline, with the running length at every
/// point, so it can be cut to a fraction of its total length cheaply.
///
/// Curves are sampled rather than measured analytically. At glyph scale the
/// error is far below a pixel, and the alternative is a closed-form arc length
/// for cubics, which does not have one.
pub struct Traced {
    /// `(point, starts a new contour, length along the outline so far)`.
    points: Vec<(Vec2, bool, f32)>,
    pub(crate) length: f32,
}

/// How finely a curve is sampled when flattening. Eight is invisible at glyph
/// size and keeps a whole formula's outline a few thousand points.
pub(crate) const CURVE_SAMPLES: usize = 8;

pub(crate) fn flatten(path: &Path) -> Traced {
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

pub(crate) fn cubic_at(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, u: f32) -> Vec2 {
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
pub(crate) fn trim(traced: &Traced, fraction: f32) -> Path {
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
pub(crate) fn map_path(path: &Path, f: impl Fn(Vec2) -> Vec2) -> Path {
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
