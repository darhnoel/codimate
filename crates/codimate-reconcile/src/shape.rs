//! A shape as the frontend sends it, its outline, and its colour.

use crate::{
    tween, AnchorKind, Animated, Color, Error, Geometry, IntoAnimated, Path, Result, Segment,
    Style, TextAlign, Vec2,
};

/// One shape from Python. A flat union: a text shape carries unused `w`/`h`.
///
/// Deliberate (ADR 0008) — it makes the diff a uniform field-by-field
/// comparison instead of per-kind special cases. Revisit around 8-10 kinds.
#[derive(Clone, Debug, Default, PartialEq)]
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
    /// A text's font: a `.ttf`/`.otf` path or an installed family name.
    /// Empty is the built-in font.
    pub font: String,
    /// Per-Scene joint angles and expression for a character Item.
    pub pose: String,
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
pub const KINDS: [&str; 11] = [
    "rect",
    "circle",
    "text",
    "line",
    "formula",
    "polygon",
    "curve",
    "svg",
    "image",
    "arc",
    "character",
];

/// A rectangle with rounded corners, in local space, centred on the anchor.
///
/// Built as a `Path` rather than a new `Geometry` arm, which is what the
/// refactor plan says free shapes should be. A radius of 0 still produces the
/// same eight segments, so a rect can animate from square to round corners —
/// two paths only tween if they have matching structure.
pub(crate) fn round_rect_path(s: &Shape) -> Path {
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
pub(crate) fn pivot_of(name: &str) -> Result<AnchorKind> {
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
pub(crate) fn polygon_path(s: &Shape) -> Path {
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
pub(crate) fn curve_path(s: &Shape) -> Path {
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

/// How many cubics an arc is built from, whatever it sweeps.
///
/// Fixed rather than "one per 90 degrees", because two arcs only tween if they
/// have the same path structure — the rule `polygon` follows for corners. A
/// fixed count is what lets a pie fill or an angle mark grow smoothly, which is
/// the whole reason this kind exists rather than a `curve` through points on a
/// circle. Eight keeps every span under 45 degrees even at a full turn, where
/// the cubic approximation is good to a few thousandths of a radius.
pub(crate) const ARC_SPANS: usize = 8;

/// Whether an arc is an open curve or a pie slice closed back to its centre.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outline {
    /// An open curve.
    Arc,
    /// A pie slice, closed back to the centre.
    Sector,
}

/// An arc or a sector in local space.
///
/// `w`/`h` are the bounding box, so an ellipse costs nothing extra; `x2`/`y2`
/// are the start and end angle in degrees; `r` above zero closes it back to the
/// centre, making a pie slice rather than an open curve.
///
/// Angles run clockwise from twelve o'clock, matching `cm.ngon`, whose first
/// corner also points straight up.
/// The same arc, from plain numbers — so the tween can interpolate the
/// *angles* and build a true arc at every instant.
///
/// Interpolating the path's control points instead looks right for an open
/// arc and is wrong for a sector: halfway between a collapsed slice and a
/// 300-degree one is not a 150-degree slice, it is a self-crossing shape that
/// fills as a sliver and a half-disc. Caught by rendering one.
pub(crate) fn arc_at(rx: f32, ry: f32, from_deg: f32, to_deg: f32, outline: Outline) -> Path {
    let sector = outline == Outline::Sector;
    // Screen y grows downward, so subtracting a quarter turn puts zero at the
    // top and makes a rising angle read as clockwise.
    let at = |deg: f32| {
        let a = deg.to_radians() - std::f32::consts::FRAC_PI_2;
        (Vec2::new(rx * a.cos(), ry * a.sin()), a)
    };

    let (start, theta0) = at(from_deg);
    let (_, theta1) = at(to_deg);
    let step = (theta1 - theta0) / ARC_SPANS as f32;
    // The classic cubic fit to a circular span: exact at both ends and at the
    // midpoint, which is why the error stays small for spans under a quarter
    // turn.
    let k = 4.0 / 3.0 * (step / 4.0).tan();

    let mut segments = Vec::with_capacity(ARC_SPANS + 3);
    if sector {
        segments.push(Segment::MoveTo(Vec2::new(0.0, 0.0)));
        segments.push(Segment::Line(Vec2::new(0.0, 0.0), start));
    } else {
        segments.push(Segment::MoveTo(start));
    }

    let mut from = start;
    let mut a = theta0;
    for _ in 0..ARC_SPANS {
        let b = a + step;
        let to = Vec2::new(rx * b.cos(), ry * b.sin());
        // The tangent at each end, scaled by the span — the control points sit
        // along it, which is what keeps the join smooth.
        let c1 = Vec2::new(from.x - k * rx * a.sin(), from.y + k * ry * a.cos());
        let c2 = Vec2::new(to.x + k * rx * b.sin(), to.y - k * ry * b.cos());
        segments.push(Segment::Cubic(from, c1, c2, to));
        from = to;
        a = b;
    }

    if sector {
        segments.push(Segment::Line(from, Vec2::new(0.0, 0.0)));
    }

    Path {
        segments,
        closed: sector,
    }
}

/// A line in local space: from the anchor to the far end.
pub(crate) fn line_path(s: &Shape) -> Path {
    Path {
        segments: vec![Segment::Line(
            Vec2::new(0.0, 0.0),
            Vec2::new(s.x2 - s.x, s.y2 - s.y),
        )],
        closed: false,
    }
}

impl Shape {
    pub(crate) fn geometry(&self, other: &Shape) -> Geometry {
        match self.kind.as_str() {
            "circle" => Geometry::circle(tween(self.r, other.r)),

            // Python's `x` is the centre, so text is always centre-aligned;
            // the renderer does the width measurement (raster.rs).
            "text" => Geometry::Text {
                text: self.text.clone().into_animated(),
                font_size: tween(self.size, other.size),
                align: TextAlign::Center,
                font: self.font.clone(),
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

            // The angles tween, and the arc is rebuilt at every instant — so
            // a dial sweeps and a pie fills through shapes that are all
            // genuinely arcs. This is the point of the kind: a `curve` through
            // points on a circle changes its point count when the sweep
            // changes, and then it snaps instead of sweeping.
            "arc" => {
                let (rx0, ry0) = (self.w / 2.0, self.h / 2.0);
                let (rx1, ry1) = (other.w / 2.0, other.h / 2.0);
                let (a0, b0) = (self.x2, self.y2);
                let (a1, b1) = (other.x2, other.y2);
                let outline = if other.r > 0.0 {
                    Outline::Sector
                } else {
                    Outline::Arc
                };
                Geometry::path(Animated::new(move |t| {
                    let mix = |x: f32, y: f32| x + (y - x) * t;
                    arc_at(
                        mix(rx0, rx1),
                        mix(ry0, ry1),
                        mix(a0, a1),
                        mix(b0, b1),
                        outline,
                    )
                }))
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
            "formula" | "svg" | "image" | "character" => {
                Geometry::rect(0.0.into_animated(), 0.0.into_animated())
            }

            // Square corners stay a real Rect — the common case keeps the
            // cheaper primitive and the renderer's own rectangle path.
            _ if self.r > 0.0 || other.r > 0.0 => {
                Geometry::path(tween(round_rect_path(self), round_rect_path(other)))
            }

            _ => Geometry::rect(tween(self.w, other.w), tween(self.h, other.h)),
        }
    }

    pub(crate) fn style(&self) -> Result<Style> {
        // An imported SVG may say nothing about colour, which means "as
        // authored" (ADR 0014). Its per-path styles are built during
        // expansion; this shared one is only the fallback for a path the file
        // gave no paint at all, so white is as good an answer as any.
        let fill = if matches!(self.kind.as_str(), "svg" | "character") && self.color.is_empty() {
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
    pub(crate) fn anchor(&self) -> Vec2 {
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

pub(crate) fn parse_color(name: &str) -> Result<Color> {
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
