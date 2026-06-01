use codimate_core::{circle, connection, pulse_on, scene, tween, AnchorKind, Color, ConcreteNode};

#[test]
fn pulse_at_progress_0_is_at_start_anchor() {
    let a = circle().x(50.0).y(100.0).radius(20.0);
    let b = circle().x(200.0).y(100.0).radius(20.0);

    let p = pulse_on(
        connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left)),
        0.0,
    )
    .radius(4.0)
    .fill(Color::CYAN);

    let dot = p.resolve(0.0);
    // Right edge of a at (70, 100) = start of connection line
    assert!((dot.x - 70.0).abs() < 0.1);
    assert!((dot.y - 100.0).abs() < 0.1);
    assert_eq!(dot.radius, 4.0);
    assert_eq!(dot.fill, Color::CYAN);
}

#[test]
fn pulse_at_progress_1_is_at_end_anchor() {
    let a = circle().x(50.0).y(100.0).radius(20.0);
    let b = circle().x(200.0).y(100.0).radius(20.0);

    let p = pulse_on(
        connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left)),
        1.0,
    );
    let dot = p.resolve(0.0);
    // Left edge of b at (180, 100) = end of connection line
    assert!((dot.x - 180.0).abs() < 0.1);
    assert!((dot.y - 100.0).abs() < 0.1);
}

#[test]
fn pulse_at_progress_05_is_midpoint() {
    let a = circle().x(0.0).y(0.0).radius(0.0);
    let b = circle().x(100.0).y(0.0).radius(0.0);

    let p = pulse_on(
        connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left)),
        0.5,
    );
    let dot = p.resolve(0.0);
    // Line from (0,0) to (100,0), midpoint at (50,0)
    assert!((dot.x - 50.0).abs() < 0.1);
    assert!((dot.y - 0.0).abs() < 0.1);
}

#[test]
fn pulse_follows_animated_progress() {
    let a = circle().x(0.0).y(0.0).radius(0.0);
    let b = circle().x(100.0).y(0.0).radius(0.0);

    let conn = connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left));
    let p = pulse_on(conn, tween(0.0, 1.0));

    let dot0 = p.resolve(0.0);
    assert!((dot0.x - 0.0).abs() < 0.1);

    let dot1 = p.resolve(1.0);
    assert!((dot1.x - 100.0).abs() < 0.1);

    let dot05 = p.resolve(0.5);
    assert!((dot05.x - 50.0).abs() < 0.1);
}

#[test]
fn pulse_radius_and_fill_can_be_overridden() {
    let a = circle().x(0.0).y(0.0).radius(0.0);
    let b = circle().x(10.0).y(0.0).radius(0.0);

    let p = pulse_on(
        connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left)),
        0.0,
    )
    .radius(8.0)
    .fill(Color::RED);

    let dot = p.resolve(0.0);
    assert_eq!(dot.radius, 8.0);
    assert_eq!(dot.fill, Color::RED);
}

#[test]
fn pulse_in_scene_resolves_to_circle() {
    let a = circle().x(50.0).y(100.0).radius(20.0);
    let b = circle().x(200.0).y(100.0).radius(20.0);

    let conn = connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left));
    let s = scene()
        .node(a.clone())
        .node(b.clone())
        .node(conn.clone())
        .node(pulse_on(conn, 0.5).radius(5.0).fill(Color::CYAN));

    let concrete = s.resolve(0.0);
    assert_eq!(concrete.children.len(), 4);

    match &concrete.children[3] {
        ConcreteNode::Circle(c) => {
            assert_eq!(c.radius, 5.0);
            assert_eq!(c.fill, Color::CYAN);
        }
        other => panic!("expected Circle for pulse, got {other:?}"),
    }
}
