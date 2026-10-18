/// Two paths only tween if they have the same structure, so a rect must
/// produce the same segments whether or not its corners are rounded —
/// otherwise animating `radius` from 0 would pop instead of ease.
#[test]
fn rounding_the_corners_keeps_the_path_structure() {
    let mut square = Shape {
        item: "b".into(),
        kind: "rect".into(),
        x: 0.0,
        y: 0.0,
        x2: 0.0,
        y2: 0.0,
        w: 100.0,
        h: 60.0,
        r: 0.0,
        color: "white".into(),
        text: String::new(),
        size: 0.0,
        layer: 0,
        opacity: 1.0,
        ..Default::default()
    };
    let flat = round_rect_path(&square).segments.len();
    square.r = 12.0;
    let round = round_rect_path(&square).segments.len();
    assert_eq!(flat, round, "square and rounded rects must tween");

    // A radius past half the short side is a pill, not a broken path.
    square.r = 9_999.0;
    let pill = round_rect_path(&square);
    assert_eq!(pill.segments.len(), flat);
    for seg in &pill.segments {
        for v in [seg_start(seg)] {
            assert!(
                v.x.abs() <= 50.001 && v.y.abs() <= 30.001,
                "{v:?} escaped the box"
            );
        }
    }
}

/// The pen has to draw a steady fraction of the outline, not a steady
/// fraction of the *segments* — a glyph's segments vary wildly in length,
/// so counting them would make the pen lurch.
#[test]
fn the_pen_draws_by_length_not_by_segment() {
    // A 10x10 square: four sides, perimeter 40.
    let square = Path {
        segments: vec![
            Segment::MoveTo(Vec2::new(0.0, 0.0)),
            Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)),
            Segment::Line(Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)),
            Segment::Line(Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)),
            Segment::Line(Vec2::new(0.0, 10.0), Vec2::new(0.0, 0.0)),
        ],
        closed: true,
    };
    let traced = flatten(&square);
    assert!((traced.length - 40.0).abs() < 1e-3, "{}", traced.length);

    let drawn = |f: f32| {
        trim(&traced, f)
            .segments
            .iter()
            .filter_map(|s| match *s {
                Segment::Line(a, b) => Some(((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()),
                _ => None,
            })
            .sum::<f32>()
    };

    assert!(drawn(0.0) < 1e-3, "nothing is drawn at the start");
    assert!((drawn(0.25) - 10.0).abs() < 1e-3, "{}", drawn(0.25));
    // Half way is mid-side, not on a corner — the cut lands inside a
    // segment, which is what stops the line growing a side at a time.
    assert!((drawn(0.5) - 20.0).abs() < 1e-3, "{}", drawn(0.5));
    assert!((drawn(1.0) - 40.0).abs() < 1e-3, "{}", drawn(1.0));
}

fn seg_start(s: &Segment) -> Vec2 {
    match *s {
        Segment::MoveTo(a) | Segment::Line(a, _) | Segment::Quad(a, _, _) => a,
        Segment::Cubic(a, _, _, _) => a,
        Segment::Close => Vec2::new(0.0, 0.0),
    }
}

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

/// An SVG on disk, because `svg_art` reads a path rather than a string —
/// the payload carries a file name, so the cache can key on it.
fn arc_path(s: &Shape) -> Path {
    arc_at(s.w / 2.0, s.h / 2.0, s.x2, s.y2, s.r > 0.0)
}

fn arc_shape(r: f32, from: f32, to: f32, sector: bool) -> Shape {
    Shape {
        item: "arc".into(),
        kind: "arc".into(),
        w: r * 2.0,
        h: r * 2.0,
        x2: from,
        y2: to,
        r: if sector { 1.0 } else { 0.0 },
        color: "white".into(),
        opacity: 1.0,
        ..Default::default()
    }
}

/// Every point the arc visits is on the circle it claims to be — the
/// eight-span cubic fit is an approximation, so this is what says how good.
#[test]
fn an_arc_stays_on_its_circle() {
    for (from, to) in [(0.0, 90.0), (0.0, 270.0), (0.0, 360.0), (45.0, -120.0)] {
        let path = arc_path(&arc_shape(100.0, from, to, false));
        for seg in &path.segments {
            let ends = match *seg {
                Segment::Cubic(a, _, _, b) => vec![a, b],
                Segment::MoveTo(a) => vec![a],
                _ => vec![],
            };
            for p in ends {
                let radius = (p.x * p.x + p.y * p.y).sqrt();
                assert!(
                    (radius - 100.0).abs() < 0.5,
                    "{from}..{to}: a point sits at {radius}, not 100"
                );
            }
        }
    }
}

/// A sector closes back to its centre and an open arc does not — the
/// difference between a pie slice and a curved line.
#[test]
fn a_sector_returns_to_the_centre() {
    let open = arc_path(&arc_shape(50.0, 0.0, 120.0, false));
    let pie = arc_path(&arc_shape(50.0, 0.0, 120.0, true));

    assert!(!open.closed && pie.closed);
    // Distance from the centre, not `x`: an arc starting at zero degrees
    // begins at the top, where `x` is zero.
    assert!(
        matches!(open.segments.first(), Some(Segment::MoveTo(p)) if p.x.hypot(p.y) > 1.0),
        "an open arc starts on its circle, not at the centre"
    );
    assert!(
        matches!(pie.segments.first(), Some(Segment::MoveTo(p)) if p.x.abs() < 1e-6 && p.y.abs() < 1e-6),
        "a pie starts at its centre"
    );
    assert!(
        matches!(pie.segments.last(), Some(Segment::Line(_, p)) if p.x.abs() < 1e-6 && p.y.abs() < 1e-6),
        "and comes back to it"
    );
}

/// The reason this kind exists rather than a `curve` through points on a
/// circle, and the bug the first version had: halfway through a sweep the
/// shape must be a real arc at the halfway angle.
///
/// Interpolating the path's control points gives a self-crossing sliver
/// instead, which looks fine on an open arc and obviously wrong on a pie.
#[test]
fn a_sweep_passes_through_real_arcs() {
    let closed = arc_shape(100.0, 0.0, 0.0, true);
    let open_to_300 = arc_shape(100.0, 0.0, 300.0, true);

    let geometry = closed.geometry(&open_to_300);
    let codimate_core::ConcreteGeometry::Path { path: halfway } = geometry.resolve(0.5) else {
        panic!("an arc resolves to a path");
    };
    let expected = arc_path(&arc_shape(100.0, 0.0, 150.0, true));

    let ends = |p: &Path| -> Vec<Vec2> {
        p.segments
            .iter()
            .filter_map(|s| match *s {
                Segment::Cubic(_, _, _, b) => Some(b),
                _ => None,
            })
            .collect()
    };
    let (got, want) = (ends(&halfway), ends(&expected));
    assert_eq!(got.len(), want.len());
    for (a, b) in got.iter().zip(&want) {
        assert!(
            (a.x - b.x).abs() < 0.01 && (a.y - b.y).abs() < 0.01,
            "halfway through a 0..300 sweep should be the 150 arc: {a:?} vs {b:?}"
        );
    }
}

fn svg_on_disk(name: &str, body: &str) -> String {
    let path = std::env::temp_dir().join(format!("codimate-test-{name}.svg"));
    std::fs::write(&path, body).expect("temp file");
    path.to_string_lossy().into_owned()
}

fn svg_shape(file: &str, colour: &str, fit: (f32, f32)) -> Shape {
    Shape {
        item: "art".into(),
        kind: "svg".into(),
        x: 0.0,
        y: 0.0,
        w: fit.0,
        h: fit.1,
        text: file.into(),
        color: colour.into(),
        r: 1.0,
        opacity: 1.0,
        ..Default::default()
    }
}

const TWO_SHAPES: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50">
    <rect x="0" y="0" width="40" height="40" fill="#ff0000"/>
    <rect x="60" y="0" width="40" height="40" fill="#0000ff"/>
</svg>"##;

/// The sentinel that makes an import look like the file it came from
/// (ADR 0014). An empty `color` means "as authored"; anything else
/// flattens the drawing. If `style()` ever stopped honouring the empty
/// string, every import would silently go white.
#[test]
fn an_empty_colour_keeps_the_artwork_own_fills() {
    let file = svg_on_disk("authored", TWO_SHAPES);

    let as_drawn = svg_shape(&file, "", (100.0, 100.0));
    let parts = primitives(&as_drawn, &as_drawn, &[]).expect("should expand");
    assert_eq!(parts.len(), 2, "one primitive per path in the file");

    let fills: Vec<Color> = parts.iter().map(|p| p.resolve(0.0).style.fill).collect();
    assert!(
        fills.iter().any(|c| c.r > 0.9 && c.b < 0.1)
            && fills.iter().any(|c| c.b > 0.9 && c.r < 0.1),
        "the red and the blue should both survive, got {fills:?}"
    );

    // And naming a colour overrides every path at once.
    let flattened = svg_shape(&file, "white", (100.0, 100.0));
    let parts = primitives(&flattened, &flattened, &[]).expect("should expand");
    for part in &parts {
        let fill = part.resolve(0.0).style.fill;
        assert!(fill.r > 0.9 && fill.g > 0.9 && fill.b > 0.9, "{fill:?}");
    }
}

/// `size` is a box the drawing fits inside with its aspect kept, so a wide
/// file in a square box does not fill the square — and `focus()` frames
/// what is actually drawn rather than the box that was asked for.
#[test]
fn an_import_fits_its_box_without_distorting() {
    let file = svg_on_disk("fit", TWO_SHAPES);
    let wide = svg_shape(&file, "", (200.0, 200.0));

    let (x0, y0, x1, y1) = bounds(&wide);
    let (w, h) = (x1 - x0, y1 - y0);
    assert!(
        (w - 200.0).abs() < 0.5,
        "the long side fills the box, got {w}"
    );
    // The artwork is 100x40 of drawn content, so the short side must not.
    assert!(h < w * 0.6, "aspect kept: {w} by {h}");
}

fn curve_of(points: &[(f32, f32)], closed: bool) -> Shape {
    let flat: Vec<f32> = points.iter().flat_map(|(x, y)| [*x, *y]).collect();
    let xs: Vec<f32> = points.iter().map(|p| p.0).collect();
    let ys: Vec<f32> = points.iter().map(|p| p.1).collect();
    Shape {
        item: "c".into(),
        kind: "curve".into(),
        x: (xs.iter().cloned().fold(f32::MAX, f32::min)
            + xs.iter().cloned().fold(f32::MIN, f32::max))
            / 2.0,
        y: (ys.iter().cloned().fold(f32::MAX, f32::min)
            + ys.iter().cloned().fold(f32::MIN, f32::max))
            / 2.0,
        points: flat,
        w: if closed { 1.0 } else { 0.0 },
        color: "white".into(),
        opacity: 1.0,
        ..Default::default()
    }
}

/// The whole promise of `curve` is that the line goes through the samples
/// you gave it (ADR 0012). Control points are derived, so the only thing
/// worth asserting is that the derivation did not move the samples.
#[test]
fn a_curve_passes_through_every_point_it_was_given() {
    let points = [(0.0, 0.0), (100.0, -50.0), (200.0, 0.0), (300.0, 80.0)];
    let shape = curve_of(&points, false);
    let path = curve_path(&shape);

    let visited: Vec<Vec2> = path
        .segments
        .iter()
        .filter_map(|s| match *s {
            Segment::Cubic(from, _, _, to) => Some([from, to]),
            _ => None,
        })
        .flatten()
        .collect();

    for (px, py) in points {
        let want = Vec2::new(px - shape.x, py - shape.y);
        assert!(
            visited
                .iter()
                .any(|v| (v.x - want.x).abs() < 0.001 && (v.y - want.y).abs() < 0.001),
            "the curve never reaches {want:?} — visited {visited:?}"
        );
    }
    assert_eq!(
        path.segments.len(),
        points.len(),
        "one MoveTo, then a span per gap"
    );
}

/// Open and closed differ by one span: the closed one comes back.
#[test]
fn a_closed_curve_joins_its_ends() {
    let points = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
    let open = curve_path(&curve_of(&points, false));
    let closed = curve_path(&curve_of(&points, true));

    assert_eq!(closed.segments.len(), open.segments.len() + 1);
    assert!(closed.closed && !open.closed);

    // The last span of a closed curve returns to where the first began.
    let (Some(Segment::MoveTo(start)), Some(Segment::Cubic(_, _, _, end))) =
        (closed.segments.first(), closed.segments.last())
    else {
        panic!("a closed curve should start with a MoveTo and end with a Cubic");
    };
    assert!((start.x - end.x).abs() < 0.001 && (start.y - end.y).abs() < 0.001);
}

/// Two points is a straight line, so `curve` is never wrong to reach for.
#[test]
fn a_curve_through_two_points_is_straight() {
    let path = curve_path(&curve_of(&[(0.0, 0.0), (100.0, 100.0)], false));
    let Some(Segment::Cubic(from, c1, c2, to)) = path.segments.get(1) else {
        panic!("expected one cubic span");
    };
    // Every control point sits on the chord, so the cubic is a line.
    for c in [c1, c2] {
        let t = (c.x - from.x) / (to.x - from.x);
        let on_chord = from.y + t * (to.y - from.y);
        assert!((c.y - on_chord).abs() < 0.001, "{c:?} bulges off the chord");
    }
}

fn box_at(item: &str, x: f32, y: f32, w: f32, h: f32) -> Shape {
    Shape {
        item: item.into(),
        kind: "rect".into(),
        x,
        y,
        w,
        h,
        color: "white".into(),
        opacity: 1.0,
        ..Default::default()
    }
}

/// The camera is aimed by name, so it has to find the thing and frame it —
/// centre on it, and zoom until it fills the frame.
#[test]
fn focus_frames_the_shape_it_names() {
    let shapes = vec![
        box_at("a", 100.0, 100.0, 40.0, 40.0),
        box_at("b", 900.0, 600.0, 40.0, 40.0),
    ];
    let focus = Focus {
        names: vec!["b".into()],
        pad: 0.0,
        min_size: 0.0,
        fixed: vec![],
    };

    let (cx, cy, zoom) = aim(&shapes, &focus, (1280.0, 720.0)).unwrap();
    assert_eq!(
        (cx, cy),
        (900.0, 600.0),
        "centred on what it was told to look at"
    );
    assert!(zoom > 1.0, "zoomed in, not out: {zoom}");

    // and the framed scene puts that shape in the middle of the screen
    let framed = framed(shapes, Some(&focus), (1280.0, 720.0)).unwrap();
    let b = framed.iter().find(|s| s.item == "b").unwrap();
    assert!(
        (b.x - 640.0).abs() < 0.01 && (b.y - 360.0).abs() < 0.01,
        "{b:?}"
    );
}

/// A camera pointing at whitespace is the failure aiming-by-name exists to
/// prevent, so a name that is not in the Scene is an error, not a wide shot.
#[test]
fn focus_on_a_name_that_is_not_there_is_an_error() {
    let shapes = vec![box_at("a", 100.0, 100.0, 40.0, 40.0)];
    let focus = Focus {
        names: vec!["typo".into()],
        pad: 0.0,
        min_size: 0.0,
        fixed: vec![],
    };
    assert!(aim(&shapes, &focus, (1280.0, 720.0)).is_err());
}

/// Focusing a three-pixel dot would otherwise magnify it two hundred times.
#[test]
fn a_tiny_shape_does_not_fill_the_screen() {
    let shapes = vec![box_at("dot", 640.0, 360.0, 3.0, 3.0)];
    let loose = Focus {
        names: vec!["dot".into()],
        pad: 0.0,
        min_size: 240.0,
        fixed: vec![],
    };
    let (_, _, zoom) = aim(&shapes, &loose, (1280.0, 720.0)).unwrap();
    assert!(
        zoom <= 720.0 / 240.0 + 0.01,
        "clamped by min_size, got {zoom}"
    );
}

/// The overlay is the reason `focus` is usable at all: without it a caption
/// is pushed off the frame the first time the camera moves.
#[test]
fn the_overlay_stays_where_it_was_put() {
    let shapes = vec![
        box_at("thing", 200.0, 200.0, 40.0, 40.0),
        box_at("_overlay/title", 640.0, 52.0, 300.0, 30.0),
    ];
    let focus = Focus {
        names: vec!["thing".into()],
        pad: 0.0,
        min_size: 0.0,
        fixed: vec!["_overlay".into()],
    };

    let framed = framed(shapes, Some(&focus), (1280.0, 720.0)).unwrap();
    let title = framed
        .iter()
        .find(|s| s.item.starts_with("_overlay"))
        .unwrap();
    assert_eq!(
        (title.x, title.y),
        (640.0, 52.0),
        "the camera moved the title"
    );
    assert_eq!(
        (title.w, title.h),
        (300.0, 30.0),
        "the camera resized the title"
    );
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
