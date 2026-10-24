//! The Builder calls and Queries the Waist answers, each a pure function of its
//! data. Ported line for line from the Python Surface (group.py, handle.py), so
//! numbers and error messages are the ones authors already know.

use serde::Deserialize;
use serde_json::Value;

use crate::place::{centre, Frame};
use crate::{Refusal, Shape};

/// Run `f` on `data` read straight into its arguments, its answer written
/// straight back as JSON: one parse and one write per call, nothing between.
pub fn run<A: for<'de> Deserialize<'de>, T: serde::Serialize>(
    data: &str,
    f: impl FnOnce(A) -> Result<T, Refusal>,
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
        // What a Rule may say, for Skins that list the choices.
        "choices" => {
            Ok(serde_json::json!({"paths": PATHS, "eases": EASES, "orders": ORDERS}).to_string())
        }
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
    /// Read as typed values, so the Core writes every number in them: a rule
    /// from Python (`90.0`) and from JavaScript (`90`) are the same bytes.
    rules: Vec<(
        String,
        String,
        std::collections::BTreeMap<String, f64>,
        String,
        String,
    )>,
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

/// A Slot: a place to put something, its centre and size, and the edge
/// things placed there line up on.
#[derive(Clone, Deserialize, serde::Serialize)]
pub struct Slot {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    #[serde(default = "centred")]
    anchor: String,
}

fn centred() -> String {
    "center".into()
}

impl Slot {
    /// The single point this Slot anchors things at.
    fn point(&self, anchor: Option<&str>) -> Result<(f64, f64), Refusal> {
        match anchor.unwrap_or(&self.anchor) {
            "center" => Ok((self.x, self.y)),
            "bottom" => Ok((self.x, self.y + self.h / 2.0)),
            "top" => Ok((self.x, self.y - self.h / 2.0)),
            other => Err(Refusal(format!(
                "unknown anchor '{other}' — use center, top or bottom"
            ))),
        }
    }
}

#[derive(Deserialize)]
pub struct Group {
    group: Frame,
    key: Vec<String>,
    slot: Option<Slot>,
    at: Option<Value>,
    anchor: Option<String>,
    w: Option<f64>,
}

/// A place to draw a thing made of several shapes: where a Slot says, or
/// placed by `at` like any shape. A Group with neither, inside another, sits
/// at its parent's point: a group that exists only to name things.
pub fn group(a: Group) -> Result<Frame, Refusal> {
    let ((gx, gy), gw) = match &a.slot {
        Some(slot) => (slot.point(a.anchor.as_deref())?, a.w.unwrap_or(slot.w)),
        None => (
            centre(&a.group, a.at.as_ref(), 0.0)?,
            a.w.unwrap_or(a.group.w),
        ),
    };
    let mut path = a.group.path.clone();
    path.push(a.key.join("/"));
    Ok(Frame {
        path,
        ox: a.group.ox + gx,
        oy: a.group.oy + gy,
        w: gw,
    })
}

// -- Queries: answered at once, never drawn -----------------------------------

#[derive(Deserialize)]
pub struct Ngon {
    sides: i64,
    r: f64,
    at: [f64; 2],
    turn: f64,
}

/// The corners of a regular polygon; the first points straight up.
pub fn ngon(a: Ngon) -> Result<Vec<[f64; 2]>, Refusal> {
    use std::f64::consts::PI;
    if a.sides < 3 {
        return Err(Refusal(format!(
            "a polygon needs at least 3 sides, got {}",
            a.sides
        )));
    }
    let step = 2.0 * PI / a.sides as f64;
    let start = a.turn.to_radians() - PI / 2.0;
    Ok((0..a.sides)
        .map(|i| {
            let angle = start + step * i as f64;
            [a.at[0] + a.r * angle.cos(), a.at[1] + a.r * angle.sin()]
        })
        .collect())
}

#[derive(Deserialize)]
pub struct Star {
    points: i64,
    r: f64,
    inner: Option<f64>,
    at: [f64; 2],
    turn: f64,
}

/// The corners of a star: outer points and valleys in turn.
pub fn star(a: Star) -> Result<Vec<[f64; 2]>, Refusal> {
    use std::f64::consts::PI;
    let inner = a.inner.unwrap_or(a.r * 0.42);
    let step = PI / a.points as f64;
    let start = a.turn.to_radians() - PI / 2.0;
    Ok((0..a.points * 2)
        .map(|i| {
            let reach = if i % 2 == 0 { a.r } else { inner };
            let angle = start + step * i as f64;
            [a.at[0] + reach * angle.cos(), a.at[1] + reach * angle.sin()]
        })
        .collect())
}

#[derive(Deserialize)]
pub struct Run {
    #[serde(alias = "items")]
    count: usize,
    gap: f64,
    size: Option<Radius>,
    at: Option<Value>,
    within: Option<Slot>,
    canvas: [f64; 2],
}

impl Run {
    /// The region the run divides up: the whole canvas unless given one.
    fn region(&self) -> Slot {
        self.within.clone().unwrap_or(Slot {
            x: self.canvas[0] / 2.0,
            y: self.canvas[1] / 2.0,
            w: self.canvas[0],
            h: self.canvas[1],
            anchor: centred(),
        })
    }

    /// One number is the width (the height follows); a pair is both.
    fn size(&self) -> (Option<f64>, Option<f64>) {
        match self.size {
            None => (None, None),
            Some(Radius::Round(w)) => (Some(w), None),
            Some(Radius::Oval([w, h])) => (Some(w), Some(h)),
        }
    }

    /// Evenly space `count` things of `size` along an axis of `extent`,
    /// centred on `centre`; unsized, they fill 70% of it.
    fn spread(&self, size: Option<f64>, extent: f64, centre: f64) -> (Vec<f64>, f64) {
        let n = self.count as f64;
        let size = size.unwrap_or_else(|| ((extent * 0.7 - self.gap * (n - 1.0)) / n).max(1.0));
        let span = n * size + (n - 1.0) * self.gap;
        let first = centre - span / 2.0 + size / 2.0;
        let at = (0..self.count)
            .map(|i| first + i as f64 * (size + self.gap))
            .collect();
        (at, size)
    }
}

/// A row of Slots on a shared baseline near the bottom of its region,
/// anchored at bottom-centre.
pub fn row(a: Run) -> Result<Vec<Slot>, Refusal> {
    if a.count == 0 {
        return Ok(Vec::new());
    }
    let place = crate::place::place(a.at.as_ref())?;
    let region = a.region();
    let (w, h) = a.size();
    let (xs, w) = a.spread(w, region.w, region.x);
    let h = h.unwrap_or(w);
    let floor = place
        .bottom
        .unwrap_or(region.y + region.h / 2.0 - region.h * 0.22);
    let y = place.y.unwrap_or(floor - h / 2.0);
    Ok(xs
        .into_iter()
        .map(|x| Slot {
            x,
            y,
            w,
            h,
            anchor: "bottom".into(),
        })
        .collect())
}

/// A column of Slots around a centre line, anchored at their centres.
pub fn column(a: Run) -> Result<Vec<Slot>, Refusal> {
    if a.count == 0 {
        return Ok(Vec::new());
    }
    let place = crate::place::place(a.at.as_ref())?;
    let region = a.region();
    let (w, h) = a.size();
    let (ys, h) = a.spread(h, region.h, place.y.unwrap_or(region.y));
    let w = w.unwrap_or(h);
    let x = place.x.unwrap_or(region.x);
    Ok(ys
        .into_iter()
        .map(|y| Slot {
            x,
            y,
            w,
            h,
            anchor: centred(),
        })
        .collect())
}

#[derive(Deserialize)]
pub struct Measure {
    text: String,
    size: f64,
    #[serde(default)]
    font: Option<String>,
    /// The film's font (`cm.font`), for a measurement that names none.
    #[serde(default)]
    film_font: String,
}

/// How wide and tall `text` is at `size`, by the Engine's own fonts and
/// fallback. A font that will not load is refused, not measured as another.
pub fn measure(a: Measure) -> Result<[f64; 2], Refusal> {
    let font = a.font.filter(|f| !f.is_empty()).unwrap_or(a.film_font);
    if !font.is_empty() {
        codimate_fonts::FontRegistry::global()
            .load(&font)
            .map_err(Refusal)?;
    }
    let (w, h) = codimate_render::measure_text(&a.text, a.size as f32, &font);
    Ok([w as f64, h as f64])
}

#[derive(Deserialize)]
pub struct Ease {
    t: f64,
}

/// The easing curve the Engine applies between two moments.
pub fn ease(a: Ease) -> Result<f64, Refusal> {
    Ok(codimate_reconcile::ease(a.t as f32) as f64)
}

// -- Values: things an author builds and hands to `explain` -------------------

const PATHS: [&str; 5] = ["straight", "arc", "linear", "fall", "lift_carry_drop"];
const EASES: [&str; 7] = ["linear", "in", "out", "in_out", "back", "spring", "bounce"];
const ORDERS: [&str; 4] = ["screen", "reverse", "name", "centre"];

#[derive(Deserialize)]
pub struct RuleArgs {
    pattern: Vec<String>,
    position: String,
    ease: Option<String>,
    order: String,
    #[serde(default)]
    options: serde_json::Map<String, Value>,
}

/// A motion rule, checked: its fields, and the payload the Engine takes.
#[derive(serde::Serialize)]
pub struct Rule {
    pattern: String,
    position: String,
    ease: Option<String>,
    order: String,
    options: serde_json::Map<String, Value>,
    payload: (
        String,
        String,
        serde_json::Map<String, Value>,
        String,
        String,
    ),
}

fn one_of(what: &str, value: &str, choices: &[&str]) -> Result<(), Refusal> {
    if choices.contains(&value) {
        return Ok(());
    }
    Err(Refusal(format!(
        "unknown {what} '{value}' — use one of {}",
        choices.join(", ")
    )))
}

/// How things matching a pattern change. First matching rule wins.
pub fn rule(a: RuleArgs) -> Result<Rule, Refusal> {
    one_of("path", &a.position, &PATHS)?;
    if let Some(ease) = &a.ease {
        one_of("ease", ease, &EASES)?;
    }
    one_of("order", &a.order, &ORDERS)?;
    let mut options = serde_json::Map::new();
    for (name, value) in a.options {
        let number = value
            .as_f64()
            .ok_or_else(|| Refusal(format!("{name} must be a number")))?;
        options.insert(name, Value::from(number));
    }
    for name in ["delay", "stagger"] {
        let fraction = options.get(name).and_then(Value::as_f64).unwrap_or(0.0);
        if !(0.0..1.0).contains(&fraction) {
            return Err(Refusal(format!(
                "{name} is a fraction of the beat, 0 to under 1"
            )));
        }
    }
    let pattern = a.pattern.join("/");
    Ok(Rule {
        payload: (
            pattern.clone(),
            a.position.clone(),
            options.clone(),
            a.ease.clone().unwrap_or_default(),
            a.order.clone(),
        ),
        pattern,
        position: a.position,
        ease: a.ease,
        order: a.order,
        options,
    })
}
