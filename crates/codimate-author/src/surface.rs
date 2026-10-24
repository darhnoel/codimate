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
