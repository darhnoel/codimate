//! How things move between two moments: paths, eases and stagger, through
//! the public `build_segment`, `motion_path` and `curve`.

use std::collections::HashMap;

use codimate_core::{Scene, Vec2};
use codimate_reconcile::*;

#[test]
fn lift_carry_drop_actually_leaves_the_ground() {
    let a = Vec2::new(100.0, 455.0);
    let b = Vec2::new(230.0, 455.0);
    let rules = vec![(
        "bar_*".to_string(),
        "lift_carry_drop".to_string(),
        HashMap::from([("clearance".to_string(), 90.0)]),
        String::new(),
        String::new(),
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
    let rules = vec![(
        "*".to_string(),
        "linear".to_string(),
        HashMap::new(),
        String::new(),
        String::new(),
    )];
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
    let rules = vec![(
        "*".to_string(),
        "fall".to_string(),
        HashMap::new(),
        String::new(),
        String::new(),
    )];
    let path = motion_path(a, b, &rules, "ball").unwrap();

    let step = |lo: f32, hi: f32| {
        let (p, q) = (path.resolve(lo), path.resolve(hi));
        (q.x - p.x, q.y - p.y)
    };
    let (first_x, first_y) = step(0.0, 0.1);
    let (last_x, last_y) = step(0.9, 1.0);

    assert!(
        (first_x - last_x).abs() < 0.01,
        "sideways drift must not change"
    );
    assert!(
        last_y > first_y * 8.0,
        "downward speed must build: {first_y} -> {last_y}"
    );
    assert!(
        (path.resolve(1.0).y - 400.0).abs() < 0.01,
        "and still arrive"
    );
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
        ..Default::default()
    };

    let before = vec![bar("bar_3", 100.0), bar("bar_1", 180.0)];
    let after = vec![bar("bar_3", 180.0), bar("bar_1", 100.0)];

    // The rule must survive all the way into the resolved Scene.
    let rules = vec![(
        "bar_*".to_string(),
        "lift_carry_drop".to_string(),
        HashMap::from([("clearance".to_string(), 90.0)]),
        String::new(),
        String::new(),
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

fn mover(item: &str, x: f32, y: f32) -> Shape {
    Shape {
        item: item.into(),
        kind: "rect".into(),
        x,
        y,
        w: 40.0,
        h: 40.0,
        color: "blue".into(),
        opacity: 1.0,
        ..Default::default()
    }
}

fn rule(pattern: &str, path: &str, opts: &[(&str, f32)], ease: &str, order: &str) -> Rule {
    (
        pattern.into(),
        path.into(),
        opts.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        ease.into(),
        order.into(),
    )
}

fn y_of(scene: &Scene, t: f32) -> Vec<f32> {
    scene
        .resolve(t)
        .children
        .iter()
        .filter_map(|c| match c {
            codimate_core::ConcreteNode::Primitive(p) => Some(p.transform.pos.y),
            _ => None,
        })
        .collect()
}

#[test]
fn every_ease_runs_from_nought_to_one() {
    for name in EASES {
        let f = curve(name, 0.5).unwrap();
        assert!(f(0.0).abs() < 1e-4, "{name} starts at 0, got {}", f(0.0));
        assert!(
            (f(1.0) - 1.0).abs() < 1e-4,
            "{name} ends at 1, got {}",
            f(1.0)
        );
    }
    assert!(curve("wobbly", 0.5).is_err(), "an unknown ease is an error");
}

#[test]
fn back_and_spring_overshoot_and_linear_does_not() {
    let peak = |name: &str| {
        let f = curve(name, 0.5).unwrap();
        (0..=100)
            .map(|i| f(i as f32 / 100.0))
            .fold(f32::MIN, f32::max)
    };
    assert!(peak("back") > 1.02 && peak("spring") > 1.02);
    assert!(peak("linear") <= 1.0 && peak("in_out") <= 1.0);
}

#[test]
fn bounce_lands_and_lifts_again_before_resting() {
    let f = curve("bounce", 0.5).unwrap();
    let ys: Vec<f32> = (0..=200).map(|i| f(i as f32 / 200.0)).collect();
    let dips = ys.windows(3).filter(|w| w[1] < w[0] && w[1] < w[2]).count();
    assert!(
        dips >= 2,
        "it should bounce at least twice, dipped {dips} times"
    );
}

#[test]
fn an_arc_bows_off_the_straight_line() {
    let rules = vec![rule("*", "arc", &[("bend", 0.3)], "", "")];
    let path = motion_path(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), &rules, "x").unwrap();
    let mid = path.resolve(0.5);
    assert!(
        mid.y.abs() > 10.0,
        "halfway it is off the line, got {mid:?}"
    );
    let end = path.resolve(1.0);
    assert!((end.x - 100.0).abs() < 1e-3 && end.y.abs() < 1e-3);
}

#[test]
fn a_stagger_starts_a_row_one_after_another() {
    let before: Vec<Shape> = [300.0, 100.0, 200.0]
        .iter()
        .map(|&x| mover(&format!("bar_{x}"), x, 300.0))
        .collect();
    let after: Vec<Shape> = [300.0, 100.0, 200.0]
        .iter()
        .map(|&x| mover(&format!("bar_{x}"), x, 400.0))
        .collect();
    let rules = vec![rule("bar_*", "straight", &[("stagger", 0.3)], "", "")];
    let scene = build_segment(&before, &after, &rules).unwrap();
    // Drawn in name order: bar_100, bar_200, bar_300 — left to right too.
    let early = y_of(&scene, 0.2);
    assert!(early[0] > 300.0, "the leftmost has set off, {early:?}");
    assert_eq!(early[2], 300.0, "the rightmost has not started, {early:?}");
    assert!(
        y_of(&scene, 1.0).iter().all(|&y| (y - 400.0).abs() < 1e-3),
        "all arrive"
    );

    let rules = vec![rule(
        "bar_*",
        "straight",
        &[("stagger", 0.3)],
        "",
        "reverse",
    )];
    let reversed = build_segment(&before, &after, &rules).unwrap();
    let early = y_of(&reversed, 0.2);
    assert!(
        early[2] > 300.0 && early[0] == 300.0,
        "reverse starts on the right, {early:?}"
    );
}

#[test]
fn a_rule_with_no_timing_or_ease_changes_nothing() {
    let before = vec![mover("a", 100.0, 300.0), mover("b", 200.0, 300.0)];
    let after = vec![mover("a", 100.0, 400.0), mover("b", 250.0, 300.0)];
    let plain = build_segment(&before, &after, &[]).unwrap();
    let ruled = build_segment(&before, &after, &[rule("*", "straight", &[], "", "")]).unwrap();
    for i in 0..=10 {
        let t = i as f32 / 10.0;
        assert_eq!(plain.resolve(t), ruled.resolve(t), "differs at t={t}");
    }
}

#[test]
fn an_ease_reaches_size_and_colour_not_only_position() {
    let mut big = mover("a", 100.0, 300.0);
    big.w = 200.0;
    let before = vec![mover("a", 100.0, 300.0)];
    let after = vec![big];
    let linear = build_segment(&before, &after, &[]).unwrap();
    let eased = build_segment(&before, &after, &[rule("*", "straight", &[], "in", "")]).unwrap();
    let width = |s: &Scene| match &s.resolve(0.5).children[0] {
        codimate_core::ConcreteNode::Primitive(p) => match p.geometry {
            codimate_core::ConcreteGeometry::Rect { width, .. } => width,
            _ => panic!("a rect"),
        },
        _ => panic!("a primitive"),
    };
    assert!(width(&eased) < width(&linear) - 10.0, "ease-in grows late");
}
