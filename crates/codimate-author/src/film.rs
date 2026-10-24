//! What an Explanation knows about itself once its scenes are made: its
//! timeline, its chapters, and which labels something is drawn over (ADR 0017).
//! Ported from the Python Surface's explanation.py and scene/coverage.py.

use serde::Deserialize;
use serde_json::Value;

use crate::{Refusal, Shape};

#[derive(Deserialize)]
pub struct Event {
    name: String,
    #[serde(default)]
    chapter: Option<Value>,
}

/// An Explanation as a Skin sends it.
#[derive(Deserialize)]
pub struct Film {
    durations: Vec<f64>,
    events: Vec<Event>,
    #[serde(default)]
    scenes: Vec<Vec<Shape>>,
    #[serde(default)]
    cameras: Vec<Option<Value>>,
    #[serde(default)]
    film_font: String,
}

/// `round(x, digits)` as Python rounds: the exact value, to nearest, ties to even.
fn py_round(x: f64, digits: usize) -> f64 {
    format!("{x:.digits$}").parse().unwrap_or(x)
}

fn beats(f: &Film) -> Vec<(f64, f64, String)> {
    let mut names = vec!["(opening)".to_string()];
    names.extend(f.events.iter().map(|e| e.name.clone()));
    names.push("(final hold)".into());
    let mut at = 0.0;
    names
        .into_iter()
        .zip(&f.durations)
        .map(|(name, length)| {
            let beat = (py_round(at, 3), *length, name);
            at += length;
            beat
        })
        .collect()
}

/// Every beat as (start, duration, event name), in seconds.
pub fn timeline(f: Film) -> Result<Vec<(f64, f64, String)>, Refusal> {
    Ok(beats(&f))
}

/// Where each part of the film starts: an event's `chapter` starts one as
/// that event's beat begins.
pub fn chapters(f: Film) -> Result<Vec<(f64, String)>, Refusal> {
    let beats = beats(&f);
    Ok(f.events
        .iter()
        .enumerate()
        .filter_map(|(k, e)| {
            let chapter = e.chapter.as_ref()?;
            let words = match chapter {
                Value::String(s) if !s.is_empty() => s.clone(),
                Value::Null | Value::Bool(false) => return None,
                Value::String(_) => return None,
                other => other.to_string(),
            };
            Some((py_round(beats[k + 1].0, 3), words))
        })
        .collect())
}

/// Where a shape sits: (left, top, width, height). Text is measured in the
/// film's font, as the Python Surface did; a formula needs Typst, and with
/// none (a browser) says nothing rather than guess.
fn bounds(s: &Shape, film_font: &str) -> Option<[f64; 4]> {
    match s.kind.as_str() {
        "rect" | "image" | "svg" => Some([s.x - s.w / 2.0, s.y - s.h / 2.0, s.w, s.h]),
        "circle" | "arc" => Some([s.x - s.r, s.y - s.r, s.r * 2.0, s.r * 2.0]),
        "line" => Some([
            s.x.min(s.x2),
            s.y.min(s.y2),
            (s.x2 - s.x).abs(),
            (s.y2 - s.y).abs(),
        ]),
        "polygon" | "curve" => {
            if s.points.len() < 2 {
                return None;
            }
            let xs: Vec<f64> = s.points.iter().step_by(2).copied().collect();
            let ys: Vec<f64> = s.points.iter().skip(1).step_by(2).copied().collect();
            let (x0, x1) = (fold(&xs, f64::min), fold(&xs, f64::max));
            let (y0, y1) = (fold(&ys, f64::min), fold(&ys, f64::max));
            Some([x0, y0, x1 - x0, y1 - y0])
        }
        "text" => {
            let (w, h) = codimate_render::measure_text(&s.text, s.size as f32, film_font);
            let (w, h) = (w as f64, h as f64);
            Some([s.x - w / 2.0, s.y - h / 2.0, w, h])
        }
        "formula" => {
            let (w, h) = codimate_reconcile::formula_size(&s.text, s.size as f32).ok()?;
            let (w, h) = (w as f64, h as f64);
            Some([s.x - w / 2.0, s.y - h / 2.0, w, h])
        }
        _ => None,
    }
}

fn fold(values: &[f64], f: fn(f64, f64) -> f64) -> f64 {
    values[1..].iter().fold(values[0], |a, b| f(a, *b))
}

/// Every label something is drawn on, as (above, below): nothing visible may
/// be drawn above a text or a formula, and no two of them may overlap. A
/// camera moves the world, not overlays, so with one the two are not compared.
fn covered(scene: &[Shape], focused: bool, film_font: &str) -> Vec<(String, String)> {
    let mut shapes: Vec<&Shape> = scene
        .iter()
        .filter(|s| s.opacity > 0.0 && (s.color != "none" || s.edge_w > 0.0))
        .filter(|s| !(s.kind == "text" && s.item.contains("/blur/")))
        .collect();
    shapes.sort_by(|a, b| a.layer.cmp(&b.layer).then_with(|| a.item.cmp(&b.item)));
    let boxes: Vec<Option<[f64; 4]>> = shapes.iter().map(|s| bounds(s, film_font)).collect();
    let overlay = |s: &Shape| s.item.starts_with("_overlay");
    let mut found = Vec::new();
    for (i, label) in shapes.iter().enumerate() {
        let Some(label_box) = boxes[i].filter(|_| label.kind == "text" || label.kind == "formula")
        else {
            continue;
        };
        for (j, other) in shapes.iter().enumerate().skip(i + 1) {
            let Some(outer) = boxes[j] else { continue };
            if focused && overlay(label) != overlay(other) {
                continue;
            }
            if touches(other, outer, label_box) {
                found.push((other.item.clone(), label.item.clone()));
            }
        }
    }
    found
}

/// Whether `shape` crosses the label's box by more than two pixels: a
/// measured box includes the glyphs' blank side bearings.
fn touches(shape: &Shape, outer: [f64; 4], label: [f64; 4]) -> bool {
    let slack = 2.0;
    let [lx, ly, lw, lh] = [
        label[0] + slack,
        label[1] + slack,
        label[2] - 2.0 * slack,
        label[3] - 2.0 * slack,
    ];
    if shape.kind == "line" {
        // A diagonal line's box covers a lot the line does not: clip the
        // segment itself against the label, grown by half the stroke.
        let grow = shape.w / 2.0;
        return crosses(
            (shape.x, shape.y),
            (shape.x2, shape.y2),
            [lx - grow, ly - grow, lx + lw + grow, ly + lh + grow],
        );
    }
    let [ox, oy, ow, oh] = outer;
    ox < lx + lw && lx < ox + ow && oy < ly + lh && ly < oy + oh
}

/// Liang–Barsky: does the segment enter the rectangle (left, top, right, bottom)?
fn crosses(
    (x0, y0): (f64, f64),
    (x1, y1): (f64, f64),
    [left, top, right, bottom]: [f64; 4],
) -> bool {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for (p, q) in [
        (-dx, x0 - left),
        (dx, right - x0),
        (-dy, y0 - top),
        (dy, bottom - y0),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                lo = lo.max(t);
            } else {
                hi = hi.min(t);
            }
            if lo > hi {
                return false;
            }
        }
    }
    true
}

#[derive(Deserialize)]
pub struct Bounds {
    shape: Shape,
    #[serde(default)]
    film_font: String,
}

/// Where one shape sits, for Python's index (or nothing, for a formula with
/// no Typst).
pub fn shape_bounds(a: Bounds) -> Result<Option<[f64; 4]>, Refusal> {
    Ok(bounds(&a.shape, &a.film_font))
}

#[derive(Deserialize)]
pub struct OneScene {
    scene: Vec<Shape>,
    camera: Option<Value>,
    #[serde(default)]
    film_font: String,
}

/// The covered labels of one Scene.
pub fn covered_scene(a: OneScene) -> Result<Vec<(String, String)>, Refusal> {
    Ok(covered(
        &a.scene,
        a.camera.is_some_and(|c| !c.is_null()),
        &a.film_font,
    ))
}

/// Every label something was drawn on, (seconds, above, below), checked once
/// per beat on the picture it settles into, reported where it starts.
pub fn film_covered(f: Film) -> Result<Vec<(f64, String, String)>, Refusal> {
    let mut found = Vec::new();
    let mut before: Vec<(String, String)> = Vec::new();
    for (i, (at, secs, _)) in beats(&f).into_iter().enumerate() {
        let k = (i + 1).min(f.scenes.len().saturating_sub(1));
        let focused = f
            .cameras
            .get(k)
            .is_some_and(|c| c.as_ref().is_some_and(|c| !c.is_null()));
        let now = covered(&f.scenes[k], focused, &f.film_font);
        // A collision that lasts is reported where it starts.
        for (above, below) in now.iter().filter(|pair| !before.contains(pair)) {
            found.push((py_round(at + secs, 2), above.clone(), below.clone()));
        }
        before = now;
    }
    found.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then_with(|| (&a.1, &a.2).cmp(&(&b.1, &b.2)))
    });
    Ok(found)
}

/// `covered`, worded: what a render prints when it is not empty.
pub fn report(f: Film) -> Result<String, Refusal> {
    let found = film_covered(f)?;
    if found.is_empty() {
        return Ok(String::new());
    }
    let quoted = |name: &str| {
        let room = 32;
        if name.chars().count() <= room {
            format!("\"{name}\"")
        } else {
            format!("\"{}…\"", name.chars().take(room - 1).collect::<String>())
        }
    };
    let many = found.len() != 1;
    let mut lines = vec![format!(
        "{} label{} covered:",
        found.len(),
        if many { "s are" } else { " is" }
    )];
    for (at, above, below) in found {
        let stamp = format!(
            "{}:{:05.2}",
            (at / 60.0).floor() as i64,
            at.rem_euclid(60.0)
        );
        lines.push(format!(
            "  {stamp}  {} is drawn over {}",
            quoted(&above),
            quoted(&below)
        ));
    }
    Ok(lines.join("\n") + "\n\nrendered anyway.")
}
