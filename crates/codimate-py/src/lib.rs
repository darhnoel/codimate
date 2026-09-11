//! PyO3 bindings — the Engine side of the Python Authoring Surface (ADR 0008).
//!
//! Python hands over a list of flat Scenes (one per Trace Event) exactly once.
//! Everything per-frame happens here: pair shapes by Item, build tweens, and
//! hand the result to `codimate-export`.
//!
//! The diff *produces* `Animated<T>` rather than replacing it — one
//! `codimate_core::Scene` per segment, sampled with `scene.resolve(t)`. That
//! keeps Invariant 1 (`f(t) → Scene`) intact.

use std::collections::HashMap;

use codimate_animation::Playable;
use codimate_core::{
    scene::Transformable, tween, Animated, Color, ConcreteScene, Geometry, IntoAnimated, Path,
    Primitive, Scene, Segment, Style, TextAlign, Vec2,
};
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

// ============================================================
// Payload — the boundary is a data structure, not an API
// ============================================================

/// One shape from Python. A flat union: a text shape carries unused `w`/`h`.
///
/// Deliberate (ADR 0008) — it makes the diff a uniform field-by-field
/// comparison instead of per-kind special cases. Revisit around 8-10 kinds.
#[derive(FromPyObject, Clone, Debug)]
#[pyo3(from_item_all)]
struct Shape {
    item: String,
    kind: String,
    x: f32,
    y: f32,
    /// For a line, the far end. Unused by every other kind.
    x2: f32,
    y2: f32,
    /// For a line, the stroke width.
    w: f32,
    h: f32,
    r: f32,
    color: String,
    text: String,
    size: f32,
    layer: i32,
    opacity: f32,
}

/// Every `kind` Python may send. An unknown kind is a Python `ValueError`,
/// never a silently missing shape.
const KINDS: [&str; 4] = ["rect", "circle", "text", "line"];

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

            _ => Geometry::rect(tween(self.w, other.w), tween(self.h, other.h)),
        }
    }

    fn style(&self) -> PyResult<Style> {
        let color = parse_color(&self.color)?;
        Ok(if self.kind == "line" {
            // A line is drawn, not filled — `w` is its stroke width.
            Style::new().fill(Color::TRANSPARENT).stroke(self.w, color)
        } else {
            Style::new().fill(color)
        })
    }

    /// Python's `y` is always the **centre** of the shape. Rust text is
    /// baseline-positioned, so convert here — an author never meets a baseline.
    fn anchor(&self) -> Vec2 {
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

fn parse_color(name: &str) -> PyResult<Color> {
    let c = |r: f32, g: f32, b: f32| Color { r, g, b, a: 1.0 };
    Ok(match name {
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
                    .map_err(|_| PyValueError::new_err(format!("bad hex color: {hex}")))
            };
            Color {
                r: v(1)?,
                g: v(3)?,
                b: v(5)?,
                a: 1.0,
            }
        }
        other => {
            return Err(PyValueError::new_err(format!(
                "unknown color {other:?} — use a name or #rrggbb"
            )))
        }
    })
}

// ============================================================
// Motion rules
// ============================================================

type Rule = (String, String, HashMap<String, f32>);

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

fn ease_in_out(t: f32) -> f32 {
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
fn motion_path(a: Vec2, b: Vec2, rules: &[Rule], item: &str) -> PyResult<Animated<Vec2>> {
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
            return Err(PyValueError::new_err(format!(
                "unknown motion path {other:?} — use \"straight\", \"linear\", \"fall\" or \"lift_carry_drop\""
            )))
        }
    })
}

// ============================================================
// The diff — motion is implied by identity
// ============================================================

fn primitive(before: &Shape, after: &Shape, rules: &[Rule]) -> PyResult<Primitive> {
    if before.kind != after.kind {
        return Err(PyValueError::new_err(format!(
            "item {:?} changed kind from {:?} to {:?} — an Item keeps one shape kind",
            after.item, before.kind, after.kind
        )));
    }

    let path = motion_path(before.anchor(), after.anchor(), rules, &after.item)?;

    Ok(Primitive::new(before.geometry(after))
        .pos(path)
        .style(tween(before.style()?, after.style()?))
        .opacity(tween(before.opacity, after.opacity)))
}

/// One segment's Scene: every Item that exists in `before` or `after`, tweened.
fn build_segment(before: &[Shape], after: &[Shape], rules: &[Rule]) -> PyResult<Scene> {
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
        let prim = match (b.get(&shape.item), a.get(&shape.item)) {
            // Present in both — the ordinary case.
            (Some(from), Some(to)) => primitive(from, to, rules)?,

            // Entering — fade in where it lands, no travel.
            (None, Some(to)) => {
                let mut faded = to.clone();
                faded.opacity = 0.0;
                primitive(&faded, to, rules)?
            }

            // Exiting — fade out where it was.
            (Some(from), None) => {
                let mut faded = from.clone();
                faded.opacity = 0.0;
                primitive(from, &faded, rules)?
            }

            (None, None) => unreachable!("item came from one of the two lists"),
        };
        scene = scene.add(prim);
    }

    Ok(scene)
}

// ============================================================
// Playable
// ============================================================

struct Explanation {
    /// `(start_seconds, duration, scene)` — scenes hold their own local `t`.
    segments: Vec<(f32, f32, Scene)>,
    total: f32,
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
// The single crossing
// ============================================================

/// Render an explanation to `output`.
///
/// `scenes` has one entry per Trace Event plus the initial Scene, so it is
/// always `durations.len() + 1` long. Zero-length segments are dropped rather
/// than producing an empty time interval.
#[pyfunction]
#[pyo3(signature = (scenes, rules, durations, output, width=1280.0, height=720.0, fps=30.0, scale=1.0))]
#[allow(clippy::too_many_arguments)]
fn render(
    scenes: Vec<Vec<Shape>>,
    rules: Vec<Rule>,
    durations: Vec<f32>,
    output: String,
    width: f32,
    height: f32,
    fps: f32,
    scale: f32,
) -> PyResult<()> {
    if scale <= 0.0 {
        return Err(PyValueError::new_err("scale must be positive"));
    }
    if scenes.len() != durations.len() + 1 {
        return Err(PyValueError::new_err(format!(
            "got {} scenes for {} durations — expected {}",
            scenes.len(),
            durations.len(),
            durations.len() + 1
        )));
    }

    for shape in scenes.iter().flatten() {
        if !KINDS.contains(&shape.kind.as_str()) {
            return Err(PyValueError::new_err(format!(
                "unknown kind {:?} on item {:?} — Codimate draws {}",
                shape.kind,
                shape.item,
                KINDS.join(", ")
            )));
        }
    }

    let mut segments = Vec::new();
    let mut cursor = 0.0f32;

    for (i, duration) in durations.iter().enumerate() {
        if *duration <= 0.0 {
            continue;
        }
        segments.push((
            cursor,
            *duration,
            build_segment(&scenes[i], &scenes[i + 1], &rules)?,
        ));
        cursor += duration;
    }

    if segments.is_empty() {
        return Err(PyValueError::new_err(
            "nothing to render — no events with a positive duration",
        ));
    }

    let explanation = Explanation {
        segments,
        total: cursor,
    };

    // `pixel_scale` rasterizes at the larger size rather than upscaling
    // afterwards, so 1080p is genuinely drawn at 1080p.
    let config = ExportConfig::new(fps, Viewport::new(width, height)).pixel_scale(scale);

    export_mp4(&explanation, &config, &output)
        .map_err(|e| PyValueError::new_err(format!("export failed: {e:?}")))
}

/// The easing the Engine applies between two moments.
///
/// Exposed so an author can draw or reason about pacing without writing a
/// second copy of the curve — a second copy can drift, and then a diagram
/// about Codimate stops being about Codimate.
#[pyfunction]
fn ease(t: f32) -> f32 {
    ease_in_out(t.clamp(0.0, 1.0))
}

#[pymodule]
fn _codimate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(render, m)?)?;
    m.add_function(wrap_pyfunction!(ease, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
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
}
