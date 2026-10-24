//! Axes: a graph's frame, its ticks, and the map from data to pixels (ported
//! from the Python Surface's layout/axes.py, ADR 0016). They hand back pixels
//! rather than drawing, so what an author plots is a shape with a name of
//! theirs; `draw` adds the frame, ticks and labels under names they choose.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::place::{place, resolve, Frame};
use crate::surface::{fill_shape, line_shape, text_at, Radius, Slot};
use crate::{Refusal, Shape};

/// One set of axes, as every Skin holds it.
#[derive(Clone, Deserialize, Serialize)]
pub struct Axes {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    left: f64,
    top: f64,
    w: f64,
    h: f64,
    ink: String,
    label: f64,
    layer: i64,
    #[serde(default)]
    right: f64,
    #[serde(default)]
    bottom: f64,
}

impl Axes {
    fn with_edges(mut self) -> Axes {
        self.right = self.left + self.w;
        self.bottom = self.top + self.h;
        self
    }

    fn at(&self, x: f64, y: f64) -> [f64; 2] {
        let across = (x - self.x0) / (self.x1 - self.x0);
        let up = (y - self.y0) / (self.y1 - self.y0);
        [self.left + across * self.w, self.top + self.h - up * self.h]
    }
}

/// A round step near `span / about`, from the 1-2-5 sequence, so ticks keep
/// their names until the range really changes.
fn nice_step(span: f64, about: i64) -> Result<f64, Refusal> {
    if span <= 0.0 {
        return Err(Refusal(format!(
            "a range needs width, got {}",
            py_float(span)
        )));
    }
    let raw = span / about.max(1) as f64;
    let power = libm::pow(10.0, libm::floor(libm::log10(raw)));
    Ok([1.0, 2.0, 5.0]
        .into_iter()
        .find(|nice| raw <= nice * power)
        .map_or(10.0 * power, |nice| nice * power))
}

/// How many decimals a label needs so that two ticks never read alike.
fn places(step: f64) -> usize {
    (-libm::floor(libm::log10(step) + 1e-9)).max(0.0) as usize
}

/// A float as Python writes it: `1.0`, not `1`.
fn py_float(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e16 {
        format!("{v:.1}")
    } else {
        format!("{v}")
    }
}

#[derive(Deserialize)]
pub struct Make {
    x: [f64; 2],
    y: [f64; 2],
    at: Option<Value>,
    size: Option<Radius>,
    within: Option<Slot>,
    canvas: [f64; 2],
}

/// A map from numbers to pixels: `size` defaults to most of the region, `at`
/// places the box's centre (or an edge, vertically).
pub fn axes(a: Make) -> Result<Axes, Refusal> {
    let region = a.within.unwrap_or_else(|| Slot::centred(a.canvas));
    let (w, h) = match a.size {
        None => (None, None),
        Some(Radius::Round(w)) => (Some(w), None),
        Some(Radius::Oval([w, h])) => (Some(w), Some(h)),
    };
    let w = w.unwrap_or(region.w() * 0.72);
    let h = h.unwrap_or(region.h() * 0.66);
    let p = place(a.at.as_ref())?;
    let cx = p.x.unwrap_or(region.x());
    let cy = resolve(
        [p.y, p.top, p.bottom],
        h / 2.0,
        &["y", "top", "bottom"],
        Some(region.y()),
    )?;
    if a.x[0] == a.x[1] || a.y[0] == a.y[1] {
        return Err(Refusal(format!(
            "a range needs width, got x=({}, {}) y=({}, {})",
            py_float(a.x[0]),
            py_float(a.x[1]),
            py_float(a.y[0]),
            py_float(a.y[1])
        )));
    }
    Ok(Axes {
        x0: a.x[0],
        x1: a.x[1],
        y0: a.y[0],
        y1: a.y[1],
        left: cx - w / 2.0,
        top: cy - h / 2.0,
        w,
        h,
        ink: "#8b96a8".into(),
        label: 20.0,
        layer: 0,
        right: 0.0,
        bottom: 0.0,
    }
    .with_edges())
}

#[derive(Deserialize)]
pub struct Looks {
    axes: Axes,
    ink: Option<String>,
    label: Option<f64>,
    layer: Option<f64>,
}

/// The same axes, restyled: frame, ticks and labels.
pub fn axes_looks(a: Looks) -> Result<Axes, Refusal> {
    Ok(Axes {
        ink: a.ink.unwrap_or(a.axes.ink.clone()),
        label: a.label.unwrap_or(a.axes.label),
        layer: a.layer.map_or(a.axes.layer, |l| l as i64),
        ..a.axes
    }
    .with_edges())
}

#[derive(Deserialize)]
pub struct At {
    axes: Axes,
    x: f64,
    y: f64,
}

/// One data point as a pixel pair, for any shape's `at`.
pub fn axes_at(a: At) -> Result<[f64; 2], Refusal> {
    Ok(a.axes.at(a.x, a.y))
}

#[derive(Deserialize)]
pub struct Samples {
    axes: Axes,
    steps: i64,
    over: Option<[f64; 2]>,
}

/// Where `line` samples: `steps + 1` x values across the range, or `over`.
/// Every sample is kept, even outside the box: curves only tween when their
/// point counts match.
pub fn axes_samples(a: Samples) -> Result<Vec<f64>, Refusal> {
    let [lo, hi] = a.over.unwrap_or([a.axes.x0, a.axes.x1]);
    let n = a.steps.max(1);
    Ok((0..=n)
        .map(|i| lo + (hi - lo) * i as f64 / n as f64)
        .collect())
}

#[derive(Deserialize)]
pub struct Points {
    axes: Axes,
    xs: Vec<f64>,
    ys: Vec<f64>,
}

/// The samples and the author's values for them, as pixels.
pub fn axes_points(a: Points) -> Result<Vec<[f64; 2]>, Refusal> {
    Ok(a.xs
        .iter()
        .zip(&a.ys)
        .map(|(x, y)| a.axes.at(*x, *y))
        .collect())
}

#[derive(Deserialize)]
pub struct Ticks {
    axes: Axes,
    axis: String,
    about: i64,
}

/// `(n, value, label)` per tick; `n`, the step multiple, names it.
pub fn axes_ticks(a: Ticks) -> Result<Vec<(i64, f64, String)>, Refusal> {
    ticks(&a.axes, &a.axis, a.about)
}

fn ticks(axes: &Axes, axis: &str, about: i64) -> Result<Vec<(i64, f64, String)>, Refusal> {
    let (lo, hi) = if axis == "x" {
        (axes.x0, axes.x1)
    } else {
        (axes.y0, axes.y1)
    };
    let step = nice_step(hi - lo, about)?;
    let digits = places(step);
    let first = libm::ceil(lo / step - 1e-9) as i64;
    let last = libm::floor(hi / step + 1e-9) as i64;
    Ok((first..=last)
        .map(|n| {
            let value = n as f64 * step;
            let label = format!("{value:.digits$}");
            let blank = label
                .trim_start_matches('-')
                .trim_matches(['0', '.'])
                .is_empty();
            (n, value, if blank { "0".into() } else { label })
        })
        .collect())
}

#[derive(Deserialize)]
pub struct Draw {
    axes: Axes,
    group: Frame,
    #[serde(default)]
    film_font: String,
    name: Vec<String>,
    about: i64,
    grid: bool,
}

/// The frame, ticks and labels, each named under `name`.
pub fn axes_draw(a: Draw) -> Result<Vec<Shape>, Refusal> {
    let p = &a.axes;
    let (ink, layer) = (p.ink.clone(), p.layer);
    let zero_y = p.y0 <= 0.0 && 0.0 <= p.y1;
    let zero_x = p.x0 <= 0.0 && 0.0 <= p.x1;
    let base = if zero_y { p.at(p.x0, 0.0)[1] } else { p.bottom };
    let spine = if zero_x { p.at(0.0, p.y0)[0] } else { p.left };
    let key = |parts: &[&str]| -> Vec<String> {
        a.name
            .iter()
            .cloned()
            .chain(parts.iter().map(|s| s.to_string()))
            .collect()
    };
    let styled = |shape: Shape, opacity: f64, layer: i64| Shape {
        layer,
        opacity,
        ..fill_shape(shape, ink.clone(), None, None)
    };
    let mut shapes = vec![
        styled(
            line_shape(
                &a.group,
                &key(&["x-axis"]),
                [p.left, base],
                [p.right, base],
                1.6,
            ),
            1.0,
            layer,
        ),
        styled(
            line_shape(
                &a.group,
                &key(&["y-axis"]),
                [spine, p.top],
                [spine, p.bottom],
                1.6,
            ),
            1.0,
            layer,
        ),
    ];
    for (axis, along) in [("x", true), ("y", false)] {
        for (n, value, words) in ticks(p, axis, a.about)? {
            if n == 0 && (if along { zero_x } else { zero_y }) {
                continue; // the other axis already draws through it
            }
            let (x, y) = if along {
                (p.at(value, 0.0)[0], base)
            } else {
                (spine, p.at(0.0, value)[1])
            };
            let n = n.to_string();
            if a.grid {
                let (s, e) = if along {
                    ([x, p.top], [x, p.bottom])
                } else {
                    ([p.left, y], [p.right, y])
                };
                shapes.push(styled(
                    line_shape(&a.group, &key(&["grid", axis, &n]), s, e, 1.0),
                    0.18,
                    layer - 1,
                ));
            }
            let (s, e) = if along {
                ([x, y - 5.0], [x, y + 5.0])
            } else {
                ([x - 5.0, y], [x + 5.0, y])
            };
            shapes.push(styled(
                line_shape(&a.group, &key(&["tick", axis, &n]), s, e, 1.6),
                1.0,
                layer,
            ));
            // A y label ends a fixed gap before its tick, however long it is.
            let at = if along {
                json!({"x": x, "top": y + 10.0})
            } else {
                let width =
                    codimate_render::measure_text(&words, p.label as f32, &a.film_font).0 as f64;
                json!({"x": x - 10.0 - width / 2.0, "y": y})
            };
            let label = text_at(
                &a.group,
                &key(&["label", axis, &n]),
                &words,
                p.label,
                &a.film_font,
                &at,
            )?;
            shapes.push(styled(label, 1.0, layer));
        }
    }
    Ok(shapes)
}
