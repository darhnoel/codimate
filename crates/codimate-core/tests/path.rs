use codimate_core::{circle_path, path_node, rect_path, scene, tween, Color, ConcreteNode, Lerp, Path, Segment, Vec2};

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
        .path(tween(circle_path(0.0, 0.0, 20.0), circle_path(100.0, 50.0, 40.0)))
        .fill(Color::RED);

    let resolved = node.resolve(0.0);
    assert_eq!(resolved.path.segments.len(), 4);
    assert!(resolved.path.closed);
    assert_eq!(resolved.fill, Color::RED);
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
