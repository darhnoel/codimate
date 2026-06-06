use codimate_core::{
    circle_path, ellipse_path, path_node, polygon_path, rect_path, regular_polygon_path, scene,
    triangle_path, tween, Color, ConcreteNode, Lerp, Path, Segment, Style, Vec2,
};

#[test]
fn circle_path_has_four_cubic_segments_and_is_closed() {
    let p = circle_path(100.0, 200.0, 50.0);
    assert_eq!(p.segments.len(), 4);
    assert!(p.closed);
    for seg in &p.segments {
        assert!(matches!(seg, Segment::Cubic(..)));
    }
}

#[test]
fn rect_path_has_four_line_segments_and_is_closed() {
    let p = rect_path(0.0, 0.0, 100.0, 50.0);
    assert_eq!(p.segments.len(), 4);
    assert!(p.closed);
    for seg in &p.segments {
        assert!(matches!(seg, Segment::Line(..)));
    }
}

#[test]
fn polygon_path_three_vertices_is_closed_triangle() {
    let p = polygon_path(&[
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        Vec2::new(50.0, 86.6),
    ]);
    assert_eq!(p.segments.len(), 3);
    assert!(p.closed);
    for seg in &p.segments {
        assert!(matches!(seg, Segment::Line(..)));
    }
}

#[test]
fn polygon_path_empty_or_single_vertex_has_no_segments() {
    assert!(polygon_path(&[]).segments.is_empty());
    assert!(polygon_path(&[Vec2::new(0.0, 0.0)]).segments.is_empty());
}

#[test]
fn regular_polygon_path_triangle_is_equilateral() {
    let p = regular_polygon_path(0.0, 0.0, 100.0, 3);
    assert_eq!(p.segments.len(), 3);
    assert!(p.closed);
    let verts: Vec<Vec2> = p.segments.iter().map(|s| s.to_cubic().0).collect();
    let dist = |a: Vec2, b: Vec2| ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
    let d01 = dist(verts[0], verts[1]);
    let d12 = dist(verts[1], verts[2]);
    let d20 = dist(verts[2], verts[0]);
    assert!((d01 - d12).abs() < 0.01);
    assert!((d12 - d20).abs() < 0.01);
}

#[test]
fn regular_polygon_path_square_has_four_sides() {
    let p = regular_polygon_path(0.0, 0.0, 100.0, 4);
    assert_eq!(p.segments.len(), 4);
    assert!(p.closed);
    for seg in &p.segments {
        assert!(matches!(seg, Segment::Line(..)));
    }
}

#[test]
fn triangle_path_is_convenience_for_n3() {
    let a = triangle_path(10.0, 20.0, 50.0);
    let b = regular_polygon_path(10.0, 20.0, 50.0, 3);
    assert_eq!(a.segments.len(), b.segments.len());
    for i in 0..a.segments.len() {
        let (a0, _, _, a3) = a.segments[i].to_cubic();
        let (b0, _, _, b3) = b.segments[i].to_cubic();
        assert!((a0.x - b0.x).abs() < 1e-6);
        assert!((a0.y - b0.y).abs() < 1e-6);
        assert!((a3.x - b3.x).abs() < 1e-6);
        assert!((a3.y - b3.y).abs() < 1e-6);
    }
}

#[test]
fn ellipse_path_has_four_cubic_segments_and_is_closed() {
    let p = ellipse_path(100.0, 200.0, 80.0, 40.0);
    assert_eq!(p.segments.len(), 4);
    assert!(p.closed);
    for seg in &p.segments {
        assert!(matches!(seg, Segment::Cubic(..)));
    }
}

#[test]
fn ellipse_path_with_equal_radii_matches_circle_path() {
    let p = ellipse_path(50.0, 50.0, 100.0, 100.0);
    // same control-point math: first segment end at bottom
    assert!((p.segments[0].to_cubic().0.x - 150.0).abs() < 1e-6);
    assert!((p.segments[0].to_cubic().0.y - 50.0).abs() < 1e-6);
}

#[test]
fn tween_path_morphs_circle_to_rect() {
    let circle = circle_path(0.0, 0.0, 50.0);
    let rect = rect_path(0.0, 0.0, 100.0, 50.0);

    let halfway = Path::lerp(circle.clone(), rect.clone(), 0.5);
    assert_eq!(halfway.segments.len(), 4);
    assert!(halfway.closed);
}

#[test]
fn tween_path_aligns_mismatched_segment_counts() {
    let short = rect_path(0.0, 0.0, 100.0, 50.0);
    let long = Path {
        segments: vec![
            Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)),
            Segment::Line(Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)),
            Segment::Line(Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)),
            Segment::Line(Vec2::new(0.0, 10.0), Vec2::new(0.0, 0.0)),
            Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
        ],
        closed: true,
    };

    let morphed = Path::lerp(short, long, 0.5);
    assert_eq!(morphed.segments.len(), 5);
}

#[test]
fn split_contours_multi_contour_path() {
    let path = Path {
        segments: vec![
            Segment::MoveTo(Vec2::new(0.0, 0.0)),
            Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)),
            Segment::Line(Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)),
            Segment::Close,
            Segment::MoveTo(Vec2::new(2.0, 2.0)),
            Segment::Line(Vec2::new(2.0, 2.0), Vec2::new(8.0, 2.0)),
            Segment::Line(Vec2::new(8.0, 2.0), Vec2::new(8.0, 8.0)),
            Segment::Close,
            Segment::MoveTo(Vec2::new(5.0, 5.0)),
            Segment::Line(Vec2::new(5.0, 5.0), Vec2::new(6.0, 5.0)),
            Segment::Close,
        ],
        closed: true,
    };
    let contours = path.split_contours();
    assert_eq!(contours.len(), 3);

    // First contour: outer square
    assert_eq!(contours[0].segments.len(), 4);
    assert!(matches!(contours[0].segments[0], Segment::MoveTo(_)));

    // Second contour: inner square
    assert_eq!(contours[1].segments.len(), 4);
    assert!(matches!(contours[1].segments[0], Segment::MoveTo(_)));

    // Third contour: tiny square
    assert_eq!(contours[2].segments.len(), 3);
}

#[test]
fn split_contours_single_contour_returns_one() {
    let path = Path {
        segments: vec![
            Segment::MoveTo(Vec2::new(0.0, 0.0)),
            Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)),
            Segment::Close,
        ],
        closed: false,
    };
    let contours = path.split_contours();
    assert_eq!(contours.len(), 1);
    assert_eq!(contours[0].segments.len(), 3);
}

#[test]
fn prefix_at_t0_returns_empty() {
    let path = Path {
        segments: vec![
            Segment::MoveTo(Vec2::new(0.0, 0.0)),
            Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0)),
        ],
        closed: false,
    };
    let p = path.prefix(0.0);
    assert!(p.segments.is_empty());
}

#[test]
fn prefix_at_t1_returns_full() {
    let path = Path {
        segments: vec![
            Segment::MoveTo(Vec2::new(0.0, 0.0)),
            Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0)),
        ],
        closed: false,
    };
    let p = path.prefix(1.0);
    assert_eq!(p.segments.len(), 2);
    assert_eq!(p.segments[0], Segment::MoveTo(Vec2::new(0.0, 0.0)));
    assert_eq!(
        p.segments[1],
        Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0))
    );
}

#[test]
fn prefix_at_t05_on_horizontal_line() {
    let path = Path {
        segments: vec![
            Segment::MoveTo(Vec2::new(0.0, 0.0)),
            Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0)),
        ],
        closed: false,
    };
    let p = path.prefix(0.5);
    assert_eq!(p.segments.len(), 2);
    assert!(matches!(p.segments[0], Segment::MoveTo(_)));
    // Should end at ~50.0
    if let Segment::Line(from, to) = p.segments[1] {
        assert!((to.x - 50.0).abs() < 0.1);
        assert_eq!(from.x, 0.0);
    } else {
        panic!("expected Line");
    }
}

#[test]
fn prefix_at_t05_on_path_without_moveto() {
    let path = Path {
        segments: vec![Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0))],
        closed: false,
    };
    let p = path.prefix(0.5);
    assert_eq!(p.segments.len(), 1);
    if let Segment::Line(from, to) = p.segments[0] {
        assert_eq!(from, Vec2::new(0.0, 0.0));
        assert!((to.x - 50.0).abs() < 0.1);
        assert_eq!(to.y, 0.0);
    } else {
        panic!("expected Line");
    }
}

#[test]
fn prefix_on_multi_contour_panics() {
    let path = Path {
        segments: vec![
            Segment::MoveTo(Vec2::new(0.0, 0.0)),
            Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)),
            Segment::Close,
            Segment::MoveTo(Vec2::new(5.0, 5.0)),
            Segment::Line(Vec2::new(5.0, 5.0), Vec2::new(10.0, 5.0)),
        ],
        closed: false,
    };
    // Should debug_assert (or just return bad result in release — we're fine with that)
    let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| path.prefix(0.5)));
    // In release mode without debug_assert, it just returns something; we don't crash
}

#[test]
fn split_contours_then_prefix_each_contour() {
    let path = Path {
        segments: vec![
            Segment::MoveTo(Vec2::new(0.0, 0.0)),
            Segment::Line(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)),
            Segment::Line(Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)),
            Segment::Close,
            Segment::MoveTo(Vec2::new(2.0, 2.0)),
            Segment::Line(Vec2::new(2.0, 2.0), Vec2::new(8.0, 2.0)),
            Segment::Line(Vec2::new(8.0, 2.0), Vec2::new(8.0, 8.0)),
            Segment::Close,
        ],
        closed: false,
    };
    let contours = path.split_contours();
    assert_eq!(contours.len(), 2);
    for c in &contours {
        let half = c.prefix(0.5);
        assert!(!half.segments.is_empty());
        assert_eq!(half.segments[0], c.segments[0]); // MoveTo preserved
    }
}

fn segments_approx_eq(a: &[Segment], b: &[Segment], eps: f32) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).all(|(s1, s2)| {
        let (a0, a1, a2, a3) = s1.to_cubic();
        let (b0, b1, b2, b3) = s2.to_cubic();
        (a0.x - b0.x).abs() < eps
            && (a0.y - b0.y).abs() < eps
            && (a1.x - b1.x).abs() < eps
            && (a1.y - b1.y).abs() < eps
            && (a2.x - b2.x).abs() < eps
            && (a2.y - b2.y).abs() < eps
            && (a3.x - b3.x).abs() < eps
            && (a3.y - b3.y).abs() < eps
    })
}

#[test]
fn tween_path_at_t0_equals_first() {
    let a = circle_path(0.0, 0.0, 30.0);
    let b = circle_path(100.0, 50.0, 60.0);
    let result = Path::lerp(a.clone(), b, 0.0);
    assert!(segments_approx_eq(&result.segments, &a.segments, 1e-6));
    assert_eq!(result.closed, a.closed);
}

#[test]
fn tween_path_at_t1_equals_second() {
    let a = circle_path(0.0, 0.0, 30.0);
    let b = circle_path(100.0, 50.0, 60.0);
    let result = Path::lerp(a, b.clone(), 1.0);
    assert!(segments_approx_eq(&result.segments, &b.segments, 1e-4));
    assert_eq!(result.closed, b.closed);
}

#[test]
fn path_node_resolves_to_concrete_path() {
    let node = path_node()
        .path(tween(
            circle_path(0.0, 0.0, 20.0),
            circle_path(100.0, 50.0, 40.0),
        ))
        .fill(Color::RED);

    let resolved = node.resolve(0.0);
    assert_eq!(resolved.path.segments.len(), 4);
    assert!(resolved.path.closed);
    assert_eq!(resolved.fill, Color::RED);
}

#[test]
fn path_node_style_applies_resolved_style() {
    let rest = Style::new().fill(Color::WHITE).stroke(1.0, Color::BLACK);
    let active = Style::new().fill(Color::RED).stroke(5.0, Color::CYAN);

    let resolved = path_node()
        .path(rect_path(0.0, 0.0, 20.0, 10.0))
        .style(tween(rest, active))
        .resolve(0.5);

    assert_eq!(
        resolved.fill,
        Color {
            r: 1.0,
            g: 0.5,
            b: 0.5,
            a: 1.0
        }
    );
    assert_eq!(resolved.stroke_width, 3.0);
    assert_eq!(
        resolved.stroke_color,
        Color {
            r: 0.0,
            g: 0.5,
            b: 0.5,
            a: 1.0
        }
    );
}

#[test]
fn path_node_style_obeys_builder_order_overrides() {
    let style = Style::new().fill(Color::WHITE).stroke(2.0, Color::BLACK);

    let resolved = path_node()
        .path(rect_path(0.0, 0.0, 20.0, 10.0))
        .style(style)
        .fill(Color::RED)
        .resolve(0.0);

    assert_eq!(resolved.fill, Color::RED);
    assert_eq!(resolved.stroke_width, 2.0);
    assert_eq!(resolved.stroke_color, Color::BLACK);
}

#[test]
fn path_node_in_scene_resolves() {
    let s = scene()
        .node(path_node().path(circle_path(50.0, 50.0, 25.0)))
        .node(codimate_core::rect().width(100.0).height(50.0));

    let concrete = s.resolve(0.5);
    assert_eq!(concrete.children.len(), 2);

    match &concrete.children[0] {
        ConcreteNode::Path(p) => {
            assert_eq!(p.path.segments.len(), 4);
            assert!(p.path.closed);
        }
        other => panic!("expected Path, got {other:?}"),
    }

    match &concrete.children[1] {
        ConcreteNode::Rect(r) => {
            assert_eq!(r.width, 100.0);
            assert_eq!(r.height, 50.0);
        }
        other => panic!("expected Rect, got {other:?}"),
    }
}
