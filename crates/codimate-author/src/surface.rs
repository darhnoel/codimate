//! The Builder calls and Queries the Waist answers, each a pure function of its
//! data. Ported line for line from the Python Surface (group.py, handle.py), so
//! numbers and error messages are the ones authors already know.

use serde::Deserialize;
use serde_json::Value;

use crate::place::{centre, Frame};
use crate::{Refusal, Shape};

/// Run `f` on `data` read straight into its arguments, its shape written
/// straight back as JSON: one parse and one write per call, nothing between.
pub fn run<A: for<'de> Deserialize<'de>>(
    data: &str,
    f: impl FnOnce(A) -> Result<Shape, Refusal>,
) -> Result<String, Refusal> {
    Ok(serde_json::to_string(&f(serde_json::from_str(data)?)?)?)
}

/// A shape of `kind` named `key` on `frame`, its centre made absolute.
fn placed(frame: &Frame, key: &[String], kind: &str, (x, y): (f64, f64)) -> Shape {
    Shape {
        item: frame.name(key),
        kind: kind.into(),
        x: frame.ox + x,
        y: frame.oy + y,
        ..Shape::default()
    }
}

#[derive(Deserialize)]
pub struct Rect {
    group: Frame,
    key: Vec<String>,
    h: f64,
    w: Option<f64>,
    at: Option<Value>,
}

/// A rectangle; inside a Group, with no width, the Group's width.
pub fn rect(a: Rect) -> Result<Shape, Refusal> {
    let fallback = (!a.group.path.is_empty()).then_some(a.group.w);
    let w = match a.w {
        Some(w) => w,
        None => crate::place::resolve([None, None, None], 0.0, &["w"], fallback)?,
    };
    let at = centre(&a.group, a.at.as_ref(), a.h / 2.0)?;
    Ok(Shape {
        w,
        h: a.h,
        ..placed(&a.group, &a.key, "rect", at)
    })
}

#[derive(Deserialize)]
pub struct Circle {
    group: Frame,
    key: Vec<String>,
    r: f64,
    at: Option<Value>,
}

pub fn circle(a: Circle) -> Result<Shape, Refusal> {
    let at = centre(&a.group, a.at.as_ref(), a.r)?;
    Ok(Shape {
        r: a.r,
        ..placed(&a.group, &a.key, "circle", at)
    })
}

#[derive(Deserialize)]
pub struct Text {
    group: Frame,
    key: Vec<String>,
    content: String,
    #[serde(default = "sixteen")]
    size: f64,
    #[serde(default)]
    font: Option<String>,
    /// The film's font (`cm.font`), for text that names none.
    #[serde(default)]
    film_font: String,
    at: Option<Value>,
}

fn sixteen() -> f64 {
    16.0
}

/// Text, centred horizontally on its line, drawn above shapes (layer 10).
pub fn text(a: Text) -> Result<Shape, Refusal> {
    let at = centre(&a.group, a.at.as_ref(), a.size / 2.0)?;
    let font = a.font.filter(|f| !f.is_empty()).unwrap_or(a.film_font);
    Ok(Shape {
        text: a.content,
        size: a.size,
        font,
        layer: 10,
        ..placed(&a.group, &a.key, "text", at)
    })
}

#[derive(Deserialize)]
pub struct Fill {
    shape: Shape,
    #[serde(default = "white")]
    color: String,
    edge: Option<String>,
    edge_w: Option<f64>,
}

fn white() -> String {
    "white".into()
}

/// Colour a shape. Saying nothing about the outline leaves it alone; an edge
/// with no width was asked to be visible, so it gets one.
pub fn fill(a: Fill) -> Result<Shape, Refusal> {
    let mut shape = Shape {
        color: a.color,
        ..a.shape
    };
    if let Some(edge) = a.edge {
        shape.edge = edge;
        if a.edge_w.is_none() {
            shape.edge_w = shape.edge_w.max(1.0);
        }
    }
    if let Some(w) = a.edge_w {
        shape.edge_w = w;
    }
    Ok(shape)
}

/// The operations Skins use that are not part of the author's interface.
pub fn internal(op: &str, data: &str) -> Result<String, Refusal> {
    match op {
        "film" => Ok(serde_json::to_string(&film(serde_json::from_str(data)?)?)?),
        _ => Err(Refusal(format!("unknown operation \"{op}\""))),
    }
}

#[derive(Deserialize, serde::Serialize)]
pub struct Film {
    width: f64,
    height: f64,
    durations: Vec<f64>,
    scenes: Vec<Vec<Shape>>,
    cameras: Vec<Value>,
    rules: Vec<Value>,
}

/// A film's data, as text: what a browser plays and what a Conformance
/// Example compares. Written here, once, so every Skin's bytes are the same.
fn film(f: Film) -> Result<String, Refusal> {
    let mut fonts: Vec<&str> = f
        .scenes
        .iter()
        .flatten()
        .map(|s| s.font.as_str())
        .filter(|font| !font.is_empty())
        .collect();
    fonts.sort_unstable();
    fonts.dedup();
    Ok(serde_json::to_string(&serde_json::json!({
        "version": 1,
        "width": f.width,
        "height": f.height,
        "durations": f.durations,
        "scenes": f.scenes,
        "cameras": f.cameras,
        "rules": f.rules,
        "fonts": fonts,
    }))?)
}

/// How Python shows a name in a message: `'p'`, or the joined path.
fn shown(key: &[String]) -> String {
    format!("'{}'", key.join("/"))
}

#[derive(Deserialize)]
pub struct Line {
    group: Frame,
    key: Vec<String>,
    start: [f64; 2],
    end: [f64; 2],
    w: f64,
}

/// A line between two points. Both ends move with the Group.
pub fn line(a: Line) -> Result<Shape, Refusal> {
    let [x2, y2] = a.end;
    Ok(Shape {
        x2: a.group.ox + x2,
        y2: a.group.oy + y2,
        w: a.w,
        ..placed(&a.group, &a.key, "line", (a.start[0], a.start[1]))
    })
}

#[derive(Deserialize)]
pub struct Arrow {
    group: Frame,
    key: Vec<String>,
    start: [f64; 2],
    end: [f64; 2],
    w: f64,
    head: f64,
}

/// An arrow as one filled polygon: a shaft `w` wide and a head `head` long,
/// so it carries one name and travels as one thing.
pub fn arrow(a: Arrow) -> Result<Shape, Refusal> {
    let ([x0, y0], [x1, y1]) = (a.start, a.end);
    let (dx, dy) = (x1 - x0, y1 - y0);
    let span = match dx.hypot(dy) {
        0.0 => 1.0,
        s => s,
    };
    let (ux, uy) = (dx / span, dy / span);
    let (px, py) = (-uy, ux);
    let head = a.head.min(span);
    let (bx, by) = (x1 - ux * head, y1 - uy * head);
    let (half, wing) = (a.w / 2.0, (head * 0.55).max(a.w));
    let points = vec![
        [x0 + px * half, y0 + py * half],
        [bx + px * half, by + py * half],
        [bx + px * wing, by + py * wing],
        [x1, y1],
        [bx - px * wing, by - py * wing],
        [bx - px * half, by - py * half],
        [x0 - px * half, y0 - py * half],
    ];
    polygon(Polygon {
        group: a.group,
        key: a.key,
        points,
        closed: true,
    })
}

#[derive(Deserialize)]
pub struct Polygon {
    group: Frame,
    key: Vec<String>,
    points: Vec<[f64; 2]>,
    closed: bool,
}

/// Anchored at the middle of its points, so it travels as one thing. The
/// points themselves are not moved by the Group, as in the Python Surface.
pub fn polygon(a: Polygon) -> Result<Shape, Refusal> {
    if a.points.is_empty() {
        return Err(Refusal(format!("polygon {} has no points", shown(&a.key))));
    }
    Ok(Shape {
        points: a.points.iter().flatten().copied().collect(),
        w: if a.closed { 1.0 } else { 0.0 },
        ..placed(&a.group, &a.key, "polygon", middle(&a.points))
    })
}

/// The middle of the box around `points`.
fn middle(points: &[[f64; 2]]) -> (f64, f64) {
    let span = |i: usize| {
        let values = points.iter().map(|p| p[i]);
        let low = values.clone().fold(f64::INFINITY, f64::min);
        let high = values.fold(f64::NEG_INFINITY, f64::max);
        (low + high) / 2.0
    };
    (span(0), span(1))
}

#[derive(Deserialize)]
pub struct Curve {
    group: Frame,
    key: Vec<String>,
    points: Vec<[f64; 2]>,
    w: f64,
    closed: bool,
}

/// A smooth line through every point. `w` is the closed flag in the payload,
/// as for a polygon, so the stroke width travels as `edge_w`.
pub fn curve(a: Curve) -> Result<Shape, Refusal> {
    if a.points.len() < 2 {
        return Err(Refusal(format!(
            "curve {} needs at least 2 points",
            shown(&a.key)
        )));
    }
    Ok(Shape {
        points: a.points.iter().flatten().copied().collect(),
        w: if a.closed { 1.0 } else { 0.0 },
        edge_w: a.w,
        ..placed(&a.group, &a.key, "curve", middle(&a.points))
    })
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Radius {
    Round(f64),
    Oval([f64; 2]),
}

#[derive(Deserialize)]
pub struct Arc {
    group: Frame,
    key: Vec<String>,
    r: Radius,
    sweep: [f64; 2],
    at: Option<Value>,
}

/// A slice of a circle. The bounding box rides in `w`/`h`, the sweep in
/// `x2`/`y2`, which are a line's ends elsewhere and free here.
pub fn arc(a: Arc) -> Result<Shape, Refusal> {
    let (rx, ry) = match a.r {
        Radius::Round(r) => (r, r),
        Radius::Oval([rx, ry]) => (rx, ry),
    };
    let at = centre(&a.group, a.at.as_ref(), 0.0)?;
    Ok(Shape {
        w: rx * 2.0,
        h: ry * 2.0,
        x2: a.sweep[0],
        y2: a.sweep[1],
        ..placed(&a.group, &a.key, "arc", at)
    })
}
