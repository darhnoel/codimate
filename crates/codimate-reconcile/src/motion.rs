//! Motion rules: which way a shape travels, its ease, and when it starts.

use crate::{Animated, Arc, Error, HashMap, Result, Shape, TimeCurve, Vec2};

/// A motion rule from Python: (pattern, path, numeric options, ease, order).
///
/// `ease` and `order` are empty when not given. The numeric options carry a
/// path's own settings (`clearance`, `bend`) and the rule's timing within a
/// beat (`delay`, `stagger`, as fractions of it) and a spring's `bounciness`.
pub type Rule = (String, String, HashMap<String, f32>, String, String);

/// What `ease=` may name.
pub const EASES: [&str; 7] = ["linear", "in", "out", "in_out", "back", "spring", "bounce"];

/// What `order=` may name: who goes first when a rule staggers.
pub const ORDERS: [&str; 4] = ["screen", "reverse", "name", "centre"];

pub(crate) fn rule_for<'a>(rules: &'a [Rule], item: &str) -> Option<&'a Rule> {
    rules.iter().find(|rule| glob_match(&rule.0, item))
}

/// A named easing curve, for a rule's `ease=`. Every curve runs from 0 at
/// `t = 0` to 1 at `t = 1`; `back` and `spring` pass 1 on the way and settle.
pub fn curve(name: &str, bounciness: f32) -> Result<TimeCurve> {
    Ok(match name {
        "linear" => Arc::new(|t: f32| t),
        "in" => Arc::new(|t: f32| t * t * t),
        "out" => Arc::new(|t: f32| 1.0 - (1.0 - t).powi(3)),
        "in_out" => Arc::new(|t: f32| {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        }),
        "back" => Arc::new(codimate_core::easing::back),
        "spring" => {
            // A damped spring, scaled so it lands exactly on 1 at t = 1.
            // `bounciness` 0 settles with barely a wobble; 1 rings a few times.
            let b = bounciness.clamp(0.0, 1.0);
            let damping = 7.0 - 4.0 * b;
            let freq = std::f32::consts::TAU * (0.8 + 1.4 * b);
            let raw = move |t: f32| {
                1.0 - (-damping * t).exp() * ((freq * t).cos() + damping / freq * (freq * t).sin())
            };
            let end = raw(1.0);
            Arc::new(move |t: f32| raw(t) / end)
        }
        "bounce" => Arc::new(bounce),
        other => {
            return Err(Error(format!(
                "unknown ease {other:?} — use one of {}",
                EASES.join(", ")
            )))
        }
    })
}

/// Lands and bounces three times, each lower, the way a dropped ball does.
pub(crate) fn bounce(t: f32) -> f32 {
    const N: f32 = 7.5625;
    const D: f32 = 2.75;
    if t < 1.0 / D {
        N * t * t
    } else if t < 2.0 / D {
        let u = t - 1.5 / D;
        N * u * u + 0.75
    } else if t < 2.5 / D {
        let u = t - 2.25 / D;
        N * u * u + 0.9375
    } else {
        let u = t - 2.625 / D;
        N * u * u + 0.984375
    }
}

/// Each changing shape's own clock within the beat, for the rules that ask
/// for one (`delay`, `stagger` or `ease`). Shapes under one rule start in
/// `order`, `stagger` of the beat apart, all taking the same time, the last
/// finishing as the beat ends. A shape not in the map keeps the beat's time.
pub(crate) fn clocks(
    shapes: &[(&Shape, bool)],
    rules: &[Rule],
) -> Result<HashMap<String, TimeCurve>> {
    let mut groups: Vec<Vec<&Shape>> = vec![Vec::new(); rules.len()];
    for (shape, changing) in shapes {
        if !changing {
            continue;
        }
        if let Some(i) = rules
            .iter()
            .position(|rule| glob_match(&rule.0, &shape.item))
        {
            groups[i].push(shape);
        }
    }
    let mut out = HashMap::new();
    for (rule, mut members) in rules.iter().zip(groups) {
        let opt = |k: &str| rule.2.get(k).copied().unwrap_or(0.0).max(0.0);
        let (delay, mut stagger) = (opt("delay").min(0.95), opt("stagger"));
        if members.is_empty() || (delay == 0.0 && stagger == 0.0 && rule.3.is_empty()) {
            continue;
        }
        let ease = if rule.3.is_empty() {
            None
        } else {
            let bounciness = rule.2.get("bounciness").copied().unwrap_or(0.5);
            Some(curve(&rule.3, bounciness)?)
        };
        order(&mut members, &rule.4)?;
        let n = members.len() as f32;
        // Every shape gets at least 5% of the beat; a stagger too wide for that
        // is narrowed rather than run past the end of the beat.
        if n > 1.0 && 1.0 - delay - (n - 1.0) * stagger < 0.05 {
            stagger = (1.0 - delay - 0.05).max(0.0) / (n - 1.0);
        }
        let span = 1.0 - delay - (n - 1.0) * stagger;
        for (k, shape) in members.iter().enumerate() {
            let start = delay + k as f32 * stagger;
            let ease = ease.clone();
            out.insert(
                shape.item.clone(),
                Arc::new(move |t: f32| {
                    let local = ((t - start) / span).clamp(0.0, 1.0);
                    ease.as_ref().map_or(local, |f| f(local))
                }) as TimeCurve,
            );
        }
    }
    Ok(out)
}

/// Sort a rule's shapes into the order they start in.
pub(crate) fn order(members: &mut [&Shape], how: &str) -> Result<()> {
    let at = |s: &Shape| s.anchor();
    match how {
        // Left to right, then top to bottom: a row reads across, a column down.
        "" | "screen" | "reverse" => {
            members.sort_by(|a, b| {
                let (p, q) = (at(a), at(b));
                (p.x.round(), p.y)
                    .partial_cmp(&(q.x.round(), q.y))
                    .expect("positions are never NaN")
            });
            if how == "reverse" {
                members.reverse();
            }
        }
        "name" => members.sort_by(|a, b| a.item.cmp(&b.item)),
        "centre" | "center" => {
            let n = members.len().max(1) as f32;
            let (cx, cy) = members.iter().fold((0.0, 0.0), |(x, y), m| {
                let p = at(m);
                (x + p.x / n, y + p.y / n)
            });
            let away = |s: &Shape| {
                let p = at(s);
                (p.x - cx).hypot(p.y - cy)
            };
            members.sort_by(|a, b| {
                away(a)
                    .partial_cmp(&away(b))
                    .expect("distances are never NaN")
            });
        }
        other => {
            return Err(Error(format!(
                "unknown order {other:?} — use one of {}",
                ORDERS.join(", ")
            )))
        }
    }
    Ok(())
}

/// Greedy wildcard match — `*` and `?`. Avoids a glob dependency for 15 lines.
pub(crate) fn glob_match(pattern: &str, s: &str) -> bool {
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

/// An ease that starts and ends slowly, cubic in and out, from 0 at `t = 0` to 1 at `t = 1`.
#[must_use]
pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

pub(crate) fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
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

    let rule = rule_for(rules, item);

    let (kind, opts, eased) = match rule {
        Some((_, kind, opts, ease, _)) => (kind.as_str(), opts.clone(), !ease.is_empty()),
        None => ("straight", HashMap::new(), false),
    };
    // A rule's own `ease=` replaces the built-in ease of `straight` and `arc`:
    // the path is then pure geometry, and the shape's clock does the easing.
    let along = move |t: f32| if eased { t } else { ease_in_out(t) };

    Ok(match kind {
        "straight" => Animated::new(move |t| lerp(a, b, along(t))),

        // A curve bowed to one side: a quadratic through a control point off
        // the middle of the line, `bend` of its length away (negative bows
        // the other way). Turns a slide into a gesture.
        "arc" => {
            let bend = opts.get("bend").copied().unwrap_or(0.3);
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let c = Vec2::new((a.x + b.x) / 2.0 - dy * bend, (a.y + b.y) / 2.0 + dx * bend);
            Animated::new(move |t| {
                let p = along(t);
                let (u, v) = ((1.0 - p) * (1.0 - p), 2.0 * (1.0 - p) * p);
                Vec2::new(u * a.x + v * c.x + p * p * b.x, u * a.y + v * c.y + p * p * b.y)
            })
        }

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
                "unknown motion path {other:?} — use \"straight\", \"arc\", \"linear\", \"fall\" or \"lift_carry_drop\""
            )))
        }
    })
}

/// The easing applied between two moments.
#[must_use]
pub fn ease(t: f32) -> f32 {
    ease_in_out(t.clamp(0.0, 1.0))
}
