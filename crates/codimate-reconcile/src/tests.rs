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
        {
            let v = seg_start(seg);
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

/// An SVG on disk, because `svg_art` reads a path rather than a string —
/// the payload carries a file name, so the cache can key on it.
fn arc_path(s: &Shape) -> Path {
    let outline = if s.r > 0.0 {
        Outline::Sector
    } else {
        Outline::Arc
    };
    arc_at(s.w / 2.0, s.h / 2.0, s.x2, s.y2, outline)
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

#[test]
fn bundled_codimate_character_has_connected_joints_and_expression_choices() {
    let art = character_art("codimate:codimate-mascot").expect("bundled mascot rig");
    assert_eq!(art.default_expression.as_deref(), Some("smile"));
    assert_eq!(
        art.paths[0].path.split_contours().len(),
        1,
        "the shared body silhouette must not contain facial cutouts"
    );
    assert_eq!(
        art.expressions,
        [
            "angry",
            "doubt",
            "excited",
            "happy",
            "sad",
            "smile",
            "surprised",
            "talk-closed",
            "talk-open",
            "thinking",
            "worried",
        ]
    );
    let hand = art
        .paths
        .iter()
        .find(|outline| outline.joints.iter().any(|(id, _)| id == "right-hand"))
        .expect("right hand outline");
    assert_eq!(
        hand.joints
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>(),
        ["codimate-mascot", "torso", "right-arm", "right-hand"]
    );
    assert!(art
        .paths
        .iter()
        .all(|path| path.expression.as_deref() != Some("thinking")
            || path.joints.iter().any(|(id, _)| id == "face")));
}

#[test]
fn bundled_character_keeps_navy_limbs_and_glove_borders() {
    let art = character_art("codimate:codimate-mascot").expect("bundled mascot rig");
    let navy = |color: Color| color.r < 0.1 && color.g < 0.2 && color.b < 0.4;
    let limb_paint = |joint: &str| {
        art.paths
            .iter()
            .find(|path| {
                path.joints.iter().any(|(id, _)| id == joint) && path.fill.is_some_and(navy)
            })
            .and_then(|path| path.fill)
            .unwrap_or_else(|| panic!("{joint} needs an authored fill"))
    };

    for (arm, leg) in [("left-arm", "left-leg"), ("right-arm", "right-leg")] {
        assert_eq!(limb_paint(arm), limb_paint(leg), "{arm} must match {leg}");
    }

    for (hand, leg) in [("left-hand", "left-leg"), ("right-hand", "right-leg")] {
        let outline = art
            .paths
            .iter()
            .find(|path| {
                path.joints.iter().any(|(id, _)| id == hand) && path.fill.is_some_and(navy)
            })
            .expect("glove outline");
        assert!(
            outline.fill == Some(limb_paint(leg)) && outline.dark_fill == Some(limb_paint(leg)),
            "{hand} border must match {leg} in both themes"
        );
    }
}

#[test]
fn bundled_character_keeps_gloves_white_in_both_themes() {
    let art = character_art("codimate:codimate-mascot").expect("bundled mascot rig");
    let white = parse_color("#FFFFFF").expect("white glove fill");

    for joint in ["left-hand", "right-hand"] {
        assert!(
            art.paths.iter().any(|path| {
                path.joints.iter().any(|(id, _)| id == joint)
                    && path.fill == Some(white)
                    && path.dark_fill == Some(white)
            }),
            "{joint} needs a solid white inset in both themes"
        );
    }
}

#[test]
fn character_dark_theme_tweens_group_paint_between_scenes() {
    let file = svg_on_disk(
        "character-theme",
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
          <g id="rig" data-codimate-rig="1">
            <g id="glove" data-pivot="50 50" data-dark-fill="#0071E3">
              <path d="M20 20h60v60H20z" fill="#102A50"/>
            </g>
          </g>
        </svg>"##,
    );
    let before = Shape {
        item: "guide".into(),
        kind: "character".into(),
        text: file,
        w: 100.0,
        h: 100.0,
        opacity: 1.0,
        ..Default::default()
    };
    let after = Shape {
        pose: r##"{"theme":"dark"}"##.into(),
        ..before.clone()
    };
    let mut paths = primitives(&before, &after, &[]).expect("themed character");
    assert_eq!(paths.len(), 1);
    let glove = paths.remove(0);
    let light = glove.resolve(0.0).style.fill;
    let middle = glove.resolve(0.5).style.fill;
    let dark = glove.resolve(1.0).style.fill;
    assert!(light.r < 0.1 && light.g < 0.2 && light.b < 0.4, "{light:?}");
    assert!(middle.b > light.b && middle.b < dark.b, "{middle:?}");
    assert!(dark.b > 0.8 && dark.r < 0.1, "{dark:?}");
}

#[test]
fn character_expression_groups_crossfade_between_scenes() {
    let file = svg_on_disk(
        "character-expression",
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
          <g id="rig" data-codimate-rig="1" data-default-expression="smile">
            <g id="body" data-pivot="50 50">
              <path d="M20 20h60v60H20z" fill="#ff0000"/>
              <g id="smile" data-expression="smile"><circle cx="40" cy="40" r="5" fill="#00ff00"/></g>
              <g id="surprised" data-expression="surprised"><circle cx="60" cy="40" r="5" fill="#0000ff"/></g>
            </g>
          </g>
        </svg>"##,
    );
    let before = Shape {
        item: "guide".into(),
        kind: "character".into(),
        text: file,
        w: 100.0,
        h: 100.0,
        opacity: 1.0,
        ..Default::default()
    };
    let after = Shape {
        pose: r##"{"expression":"surprised","joints":{"body":90}}"##.into(),
        ..before.clone()
    };
    let parts = primitives(&before, &after, &[]).expect("animated character");
    assert_eq!(parts.len(), 3, "body plus two expression outlines");
    assert!((parts[1].resolve(0.0).transform.opacity - 1.0).abs() < 1e-5);
    assert!(parts[1].resolve(1.0).transform.opacity < 1e-5);
    assert!(parts[2].resolve(0.0).transform.opacity < 1e-5);
    assert!((parts[2].resolve(1.0).transform.opacity - 1.0).abs() < 1e-5);
}

#[test]
fn character_expression_can_switch_without_crossfading() {
    let file = svg_on_disk(
        "character-expression-cut",
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
          <g id="rig" data-codimate-rig="1" data-default-expression="closed">
            <g id="body" data-pivot="50 50">
              <path d="M20 20h60v60H20z" fill="#ff0000"/>
              <g id="closed" data-expression="closed"><path d="M40 60h20" stroke="#000000"/></g>
              <g id="open" data-expression="open"><ellipse cx="50" cy="60" rx="5" ry="8"/></g>
            </g>
          </g>
        </svg>"##,
    );
    let before = Shape {
        item: "guide".into(),
        kind: "character".into(),
        text: file,
        w: 100.0,
        h: 100.0,
        opacity: 1.0,
        ..Default::default()
    };
    let after = Shape {
        pose: r##"{"expression":"open","expression_transition":"cut"}"##.into(),
        ..before.clone()
    };
    let parts = primitives(&before, &after, &[]).expect("animated character");

    assert!((parts[1].resolve(0.49).transform.opacity - 1.0).abs() < 1e-5);
    assert!(parts[1].resolve(0.5).transform.opacity < 1e-5);
    assert!(parts[2].resolve(0.49).transform.opacity < 1e-5);
    assert!((parts[2].resolve(0.5).transform.opacity - 1.0).abs() < 1e-5);
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
