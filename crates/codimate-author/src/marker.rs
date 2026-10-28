//! The marker: a highlighter stroke over any box (ADR 0027), and the highlight
//! that sizes one to its words. Ported from the Python Surface's marker.py and
//! its Handle and Group, number for number: the stroke's tilt and ragged ends
//! come from Python's own random generator, seeded by the marker's name.
//!
//! The Core keeps no state, so a marker's stroke travels as data (a `Mark`)
//! that each Skin keeps beside the shape and sends back with it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::place::{centre, Frame};
use crate::pyrandom::Random;
use crate::surface::text_shape;
use crate::{Refusal, Shape};

const HIGHLIGHTS: [(&str, &str); 6] = [
    ("yellow", "#ffec4f"),
    ("orange", "#ffa33a"),
    ("pink", "#ff8fc7"),
    ("green", "#8ee88a"),
    ("blue", "#7fd1ff"),
    ("purple", "#c9a3ff"),
];
const NAMED: [(&str, &str); 4] = [
    ("white", "#ffffff"),
    ("black", "#000000"),
    ("grey", "#8c8c8c"),
    ("gray", "#8c8c8c"),
];
const OPACITY: f64 = 0.85; // bright on black, with a hint of what is under it
const LAYER: i64 = 9; // under the words, which are layer 10
const SLANT: f64 = 0.2; // the chisel cut of each end, as a fraction of the height
const RAGGED: f64 = 0.05; // how far an end's bumps bite in, as a fraction of height
const BUMPS: usize = 3; // bumps along each end; the point counts never change
const INK: &str = "#171717";
const PAPER: &str = "#ffffff";

/// What a Skin keeps beside a marked shape: a marker's stroke, or the link
/// from a highlight's words to the band over them.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Mark {
    Stroke {
        w: f64,
        h: f64,
        seed: String,
        reveal: f64,
        angle: f64,
        words: Option<String>,
        paper: String,
    },
    Link {
        band: String,
    },
}

fn highlight_colour(name: &str) -> Option<&'static str> {
    HIGHLIGHTS.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
}

/// The stroke's corners, centred on the origin, drawn from its left end to
/// `reveal` of the way along, tilted as a hand draws and turned by `angle`.
fn parts(w: f64, h: f64, reveal: f64, seed: &str, angle: f64) -> Vec<[f64; 2]> {
    let mut rng = Random::new(seed);
    let magnitude = rng.uniform(0.5, 1.5);
    let sign = [-1.0, 1.0][rng.choose_of_two()];
    let tilt = (magnitude * sign).to_radians();
    let bites: Vec<f64> = (0..2 * BUMPS).map(|_| rng.uniform(0.3, 1.0)).collect();
    let drawn = reveal.clamp(0.0, 1.0) * w;
    let slant = (SLANT * h).min(drawn); // a stub too short to slant is cut flat
    let bite = RAGGED * h * if h != 0.0 { slant / (SLANT * h) } else { 0.0 };
    let (left, right) = (-w / 2.0, -w / 2.0 + drawn);
    let (top, bottom) = (-h / 2.0, h / 2.0);
    let end = |x_top: f64, x_bottom: f64, inward: f64, bumps: &[f64]| -> Vec<[f64; 2]> {
        bumps
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let f = (i + 1) as f64 / (BUMPS + 1) as f64;
                [
                    x_top + (x_bottom - x_top) * f + inward * bite * b,
                    top + (bottom - top) * f,
                ]
            })
            .collect()
    };
    let mut body = vec![[left + slant, top], [right, top]];
    body.extend(end(right, right - slant, -1.0, &bites[..BUMPS]));
    body.extend([[right - slant, bottom], [left, bottom]]);
    body.extend(
        end(left + slant, left, 1.0, &bites[BUMPS..])
            .into_iter()
            .rev(),
    );
    let turn = tilt + angle.to_radians();
    let (c, s) = (libm::cos(turn), libm::sin(turn));
    body.into_iter()
        .map(|[x, y]| [x * c - y * s, x * s + y * c])
        .collect()
}

/// The colour `t` of the way from `a` to `#rrggbb` `b`. A name this does not
/// know is not blended, only swapped half way. Rounds as Python's round().
fn blend(a: &str, b: &str, t: f64) -> String {
    let t = t.clamp(0.0, 1.0);
    let a = NAMED
        .iter()
        .chain(HIGHLIGHTS.iter())
        .find(|(n, _)| *n == a)
        .map_or(a, |(_, c)| *c);
    let hex = |s: &str| -> Option<[f64; 3]> {
        if !(s.starts_with('#') && s.len() == 7) {
            return None;
        }
        let byte = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok().map(f64::from);
        Some([byte(1)?, byte(3)?, byte(5)?])
    };
    match (hex(a), hex(b)) {
        (Some(pa), Some(pb)) => {
            let mixed: Vec<String> = pa
                .iter()
                .zip(pb)
                .map(|(x, y)| format!("{:02x}", (x + (y - x) * t).round_ties_even() as i64))
                .collect();
            format!("#{}", mixed.concat())
        }
        _ if t < 0.5 => a.to_string(),
        _ => b.to_string(),
    }
}

/// Lay a stroke out at `reveal` on its shape, and darken its words with it.
fn redraw(stroke: &mut Mark, reveal: f64, band: &mut Shape, words: Option<&mut Shape>) {
    let Mark::Stroke {
        w,
        h,
        seed,
        reveal: r,
        angle,
        paper,
        ..
    } = stroke
    else {
        return;
    };
    *r = reveal;
    band.points = parts(*w, *h, reveal, seed, *angle)
        .into_iter()
        .flat_map(|[dx, dy]| [band.x + dx, band.y + dy])
        .collect();
    if let Some(words) = words {
        words.color = blend(paper, INK, reveal);
    }
}

#[derive(Deserialize)]
pub struct MarkerArgs {
    group: Frame,
    key: Vec<String>,
    w: f64,
    h: f64,
    at: Option<Value>,
}

#[derive(Serialize)]
pub struct Marked {
    shape: Shape,
    mark: Mark,
}

/// A highlighter stroke over any box: drawn on fully, yellow, slightly
/// translucent, under the words' layer. The same every time for one name.
pub fn marker(a: MarkerArgs) -> Result<Marked, Refusal> {
    let (x, y) = centre(&a.group, a.at.as_ref(), a.h / 2.0)?;
    let name = a.group.name(&a.key);
    let mut shape = Shape {
        item: name.clone(),
        kind: "polygon".into(),
        x: a.group.ox + x,
        y: a.group.oy + y,
        w: 1.0,
        layer: LAYER,
        opacity: OPACITY,
        color: highlight_colour("yellow")
            .expect("yellow is a built-in highlight")
            .into(),
        ..Shape::default()
    };
    let mut mark = Mark::Stroke {
        w: a.w,
        h: a.h,
        seed: format!("marker {name}"),
        reveal: 1.0,
        angle: 0.0,
        words: None,
        paper: PAPER.into(),
    };
    redraw(&mut mark, 1.0, &mut shape, None);
    Ok(Marked { shape, mark })
}

#[derive(Deserialize)]
pub struct HighlightArgs {
    group: Frame,
    key: Vec<String>,
    content: String,
    size: f64,
    font: Option<String>,
    #[serde(default)]
    film_font: String,
    at: Option<Value>,
}

#[derive(Serialize)]
pub struct Highlighted {
    shapes: Vec<Shape>,
    marks: BTreeMap<String, Mark>,
}

/// Words under a highlighter stroke sized to them, padded by a quarter of
/// `size`; marked, the words are near-black.
pub fn highlight(a: HighlightArgs) -> Result<Highlighted, Refusal> {
    let font = a
        .font
        .clone()
        .filter(|f| !f.is_empty())
        .unwrap_or(a.film_font.clone());
    let (w, h) = codimate_render::measure_text(&a.content, a.size as f32, &font);
    let (w, h) = (w as f64, h as f64);
    let pad = a.size / 4.0;
    let (x, y) = centre(&a.group, a.at.as_ref(), h / 2.0)?;
    let at = Some(serde_json::json!([x, y]));
    // The band is named after the words: "pivot" is under "pivot marker".
    let band_key = vec![format!("{} marker", a.key.join("/"))];
    let Marked {
        shape: mut band,
        mut mark,
    } = marker(MarkerArgs {
        group: a.group.clone(),
        key: band_key,
        w: w + 2.0 * pad,
        h: h + 2.0 * pad,
        at: at.clone(),
    })?;
    let mut words = text_shape(&a.group, &a.key, &a.content, a.size, &font, (x, y));
    if let Mark::Stroke { words: link, .. } = &mut mark {
        *link = Some(words.item.clone());
    }
    redraw(&mut mark, 1.0, &mut band, Some(&mut words));
    let mut marks = BTreeMap::new();
    marks.insert(
        words.item.clone(),
        Mark::Link {
            band: band.item.clone(),
        },
    );
    marks.insert(band.item.clone(), mark);
    Ok(Highlighted {
        shapes: vec![band, words],
        marks,
    })
}

#[derive(Deserialize)]
pub struct MarkedArgs {
    op: String,
    name: String,
    data: serde_json::Map<String, Value>,
    shapes: BTreeMap<String, Shape>,
    marks: BTreeMap<String, Mark>,
}

#[derive(Serialize)]
pub struct Changed {
    shapes: BTreeMap<String, Shape>,
    marks: BTreeMap<String, Mark>,
}

/// A Handle method on a marked shape. A stroke takes its highlighter colours,
/// turns about its centre with the hand's tilt, and draws on as it is
/// written; a highlight's words, filled, become the paper the band blends
/// them back to. Anything else is the method as for any shape.
pub fn marked(mut a: MarkedArgs) -> Result<Changed, Refusal> {
    let mark = a.marks.get(&a.name).cloned();
    let number = |key: &str| a.data.get(key).and_then(Value::as_f64);
    match (a.op.as_str(), mark) {
        ("fill", Some(Mark::Stroke { .. })) => {
            let colour = a.data["color"].as_str().unwrap_or("white");
            let colour = highlight_colour(colour).unwrap_or(colour).to_string();
            a.data.insert("color".into(), Value::from(colour));
            plain(&mut a)?;
        }
        ("fill", Some(Mark::Link { band })) => {
            plain(&mut a)?;
            let paper = a.data["color"].as_str().unwrap_or("white").to_string();
            let mut stroke = a.marks[&band].clone();
            let reveal = match &mut stroke {
                Mark::Stroke {
                    paper: p, reveal, ..
                } => {
                    *p = paper;
                    *reveal
                }
                Mark::Link { .. } => 1.0,
            };
            let mut band_shape = a.shapes[&band].clone();
            let mut words = a.shapes[&a.name].clone();
            redraw(&mut stroke, reveal, &mut band_shape, Some(&mut words));
            a.shapes.insert(band.clone(), band_shape);
            a.shapes.insert(a.name.clone(), words);
            a.marks.insert(band, stroke);
        }
        ("turn", Some(Mark::Stroke { .. })) => {
            let pivot = a.data["pivot"].as_str().unwrap_or("center");
            if !["center", "top", "bottom", "left", "right"].contains(&pivot) {
                return Err(Refusal(format!(
                    "unknown pivot '{pivot}' — use one of center, top, bottom, left, right"
                )));
            }
            let degrees = number("degrees").unwrap_or(0.0);
            let mut stroke = a.marks[&a.name].clone();
            let reveal = match &mut stroke {
                Mark::Stroke { angle, reveal, .. } => {
                    *angle = degrees;
                    *reveal
                }
                Mark::Link { .. } => 1.0,
            };
            restroke(&mut a, stroke, reveal);
        }
        ("write", Some(Mark::Stroke { reveal, .. })) => {
            let _ = reveal;
            if let Some(new) = number("reveal") {
                let stroke = a.marks[&a.name].clone();
                restroke(&mut a, stroke, new);
            }
        }
        _ => plain(&mut a)?,
    }
    Ok(Changed {
        shapes: a.shapes,
        marks: a.marks,
    })
}

/// The method as for any shape, on the marked shape.
fn plain(a: &mut MarkedArgs) -> Result<(), Refusal> {
    let mut data = a.data.clone();
    data.insert("shape".into(), serde_json::to_value(&a.shapes[&a.name])?);
    let answer: Value =
        serde_json::from_str(&crate::call(&a.op, &Value::Object(data).to_string()))?;
    if let Some(error) = answer.get("error").and_then(Value::as_str) {
        return Err(Refusal(error.into()));
    }
    a.shapes.insert(
        a.name.clone(),
        serde_json::from_value(answer["ok"].clone())?,
    );
    Ok(())
}

/// Redraw the marked shape's stroke at `reveal`, darkening its words too.
fn restroke(a: &mut MarkedArgs, mut stroke: Mark, reveal: f64) {
    let words_name = match &stroke {
        Mark::Stroke { words, .. } => words.clone(),
        Mark::Link { .. } => None,
    };
    let mut band = a.shapes[&a.name].clone();
    let mut words = words_name.as_ref().and_then(|n| a.shapes.get(n).cloned());
    redraw(&mut stroke, reveal, &mut band, words.as_mut());
    a.shapes.insert(a.name.clone(), band);
    if let (Some(name), Some(words)) = (words_name, words) {
        a.shapes.insert(name, words);
    }
    a.marks.insert(a.name.clone(), stroke);
}
