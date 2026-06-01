use codimate_core::{circle, connection, scene, tween, AnchorKind, Color, ConcreteNode, Vec2};

#[test]
fn connection_default_resolves_to_single_line() {
    let start = Vec2::new(10.0, 20.0);
    let end = Vec2::new(100.0, 80.0);
    let conn = connection(start, end).resolve(0.0);

    assert_eq!(conn.path.segments.len(), 1);
    assert_eq!(conn.stroke_width, 1.0);
    assert_eq!(conn.stroke_color, Color::WHITE);
    assert_eq!(conn.fill, Color::TRANSPARENT);
}

#[test]
fn connection_stroke_sets_width_and_color() {
    let start = Vec2::new(0.0, 0.0);
    let end = Vec2::new(50.0, 0.0);
    let conn = connection(start, end).stroke(3.0, Color::RED).resolve(0.0);

    assert_eq!(conn.stroke_width, 3.0);
    assert_eq!(conn.stroke_color, Color::RED);
}

#[test]
fn connection_arrow_appends_arrowhead_segments() {
    let start = Vec2::new(0.0, 0.0);
    let end = Vec2::new(100.0, 0.0);
    let plain = connection(start, end).resolve(0.0);
    assert_eq!(plain.path.segments.len(), 1);

    let with_arrow = connection(start, end).arrow(10.0).resolve(0.0);
    assert_eq!(with_arrow.path.segments.len(), 5);
}

#[test]
fn connection_in_scene_resolves_to_path() {
    let a = circle().x(50.0).y(100.0).radius(20.0);
    let b = circle().x(200.0).y(100.0).radius(20.0);

    let s = scene().node(a.clone()).node(b.clone()).node(
        connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left)).stroke(2.0, Color::RED),
    );

    let concrete = s.resolve(0.0);
    assert_eq!(concrete.children.len(), 3);

    match &concrete.children[2] {
        ConcreteNode::Path(path) => {
            assert_eq!(path.stroke_width, 2.0);
            assert_eq!(path.stroke_color, Color::RED);
            assert_eq!(path.path.segments.len(), 1);
        }
        other => panic!("expected Path, got {other:?}"),
    }
}

#[test]
fn connection_anchor_coordinates_resolve_correctly() {
    let a = circle().x(50.0).y(100.0).radius(20.0);
    let b = circle().x(200.0).y(100.0).radius(20.0);

    let conn = connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left));
    let resolved = conn.resolve(0.0);
    assert_eq!(resolved.path.segments.len(), 1);
}

#[test]
fn connection_tracks_moving_anchors() {
    let a = circle().x(tween(0.0, 100.0)).y(100.0).radius(10.0);
    let b = circle().x(200.0).y(100.0).radius(10.0);

    let conn = connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left));
    assert_eq!(conn.resolve(0.0).path.segments.len(), 1);
    assert_eq!(conn.resolve(1.0).path.segments.len(), 1);
}

#[test]
fn connection_line_endpoints_mirror_anchor_positions() {
    use codimate_core::Segment;

    let a = circle().x(50.0).y(100.0).radius(20.0);
    let b = circle().x(200.0).y(100.0).radius(20.0);

    let conn = connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left));
    let resolved = conn.resolve(0.0);

    // Right edge of a: (50+20, 100) = (70, 100)
    // Left edge of b: (200-20, 100) = (180, 100)
    if let Segment::Line(from, to) = &resolved.path.segments[0] {
        assert_eq!(*from, Vec2::new(70.0, 100.0));
        assert_eq!(*to, Vec2::new(180.0, 100.0));
    } else {
        panic!("expected Line segment");
    }
}
