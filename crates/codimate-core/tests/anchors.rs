use codimate_core::{circle, circle_path, path_node, rect, rect_path, tween, AnchorKind, Vec2};

#[test]
fn concrete_circle_anchor_center() {
    let c = circle().x(100.0).y(200.0).radius(50.0).resolve(0.0);
    assert_eq!(c.anchor(AnchorKind::Center), Vec2::new(100.0, 200.0));
}

#[test]
fn concrete_circle_anchor_top() {
    let c = circle().x(100.0).y(200.0).radius(50.0).resolve(0.0);
    assert_eq!(c.anchor(AnchorKind::Top), Vec2::new(100.0, 150.0));
}

#[test]
fn concrete_circle_anchor_bottom() {
    let c = circle().x(100.0).y(200.0).radius(50.0).resolve(0.0);
    assert_eq!(c.anchor(AnchorKind::Bottom), Vec2::new(100.0, 250.0));
}

#[test]
fn concrete_circle_anchor_left() {
    let c = circle().x(100.0).y(200.0).radius(50.0).resolve(0.0);
    assert_eq!(c.anchor(AnchorKind::Left), Vec2::new(50.0, 200.0));
}

#[test]
fn concrete_circle_anchor_right() {
    let c = circle().x(100.0).y(200.0).radius(50.0).resolve(0.0);
    assert_eq!(c.anchor(AnchorKind::Right), Vec2::new(150.0, 200.0));
}

#[test]
fn animated_circle_anchor_follows_motion() {
    let c = circle().x(tween(0.0, 100.0)).y(200.0).radius(50.0);
    let top = c.anchor(AnchorKind::Top);
    assert_eq!(top.resolve(0.0), Vec2::new(0.0, 150.0));
    assert_eq!(top.resolve(1.0), Vec2::new(100.0, 150.0));
}

#[test]
fn concrete_rect_anchor_center() {
    let r = rect()
        .x(10.0)
        .y(20.0)
        .width(100.0)
        .height(50.0)
        .resolve(0.0);
    assert_eq!(r.anchor(AnchorKind::Center), Vec2::new(60.0, 45.0));
}

#[test]
fn concrete_rect_anchor_top() {
    let r = rect()
        .x(10.0)
        .y(20.0)
        .width(100.0)
        .height(50.0)
        .resolve(0.0);
    assert_eq!(r.anchor(AnchorKind::Top), Vec2::new(60.0, 20.0));
}

#[test]
fn concrete_rect_anchor_bottom() {
    let r = rect()
        .x(10.0)
        .y(20.0)
        .width(100.0)
        .height(50.0)
        .resolve(0.0);
    assert_eq!(r.anchor(AnchorKind::Bottom), Vec2::new(60.0, 70.0));
}

#[test]
fn concrete_rect_anchor_left() {
    let r = rect()
        .x(10.0)
        .y(20.0)
        .width(100.0)
        .height(50.0)
        .resolve(0.0);
    assert_eq!(r.anchor(AnchorKind::Left), Vec2::new(10.0, 45.0));
}

#[test]
fn concrete_rect_anchor_right() {
    let r = rect()
        .x(10.0)
        .y(20.0)
        .width(100.0)
        .height(50.0)
        .resolve(0.0);
    assert_eq!(r.anchor(AnchorKind::Right), Vec2::new(110.0, 45.0));
}

#[test]
fn animated_rect_anchor_follows_resize() {
    let r = rect().x(0.0).y(0.0).width(tween(100.0, 200.0)).height(50.0);
    let right = r.anchor(AnchorKind::Right);
    assert_eq!(right.resolve(0.0), Vec2::new(100.0, 25.0));
    assert_eq!(right.resolve(1.0), Vec2::new(200.0, 25.0));
}

#[test]
fn concrete_path_anchor_center() {
    let p = path_node()
        .path(rect_path(10.0, 20.0, 100.0, 50.0))
        .resolve(0.0);
    assert_eq!(p.anchor(AnchorKind::Center), Vec2::new(60.0, 45.0));
}

#[test]
fn concrete_path_anchor_top() {
    let p = path_node().path(circle_path(50.0, 50.0, 30.0)).resolve(0.0);
    // circle centered at (50,50) with r=30 → top edge at y=20
    assert_eq!(p.anchor(AnchorKind::Top), Vec2::new(50.0, 20.0));
}

#[test]
fn concrete_path_anchor_on_empty_path_falls_back_to_zero() {
    use codimate_core::Path;
    let p = path_node()
        .path(Path {
            segments: vec![],
            closed: false,
        })
        .resolve(0.0);
    assert_eq!(p.anchor(AnchorKind::Center), Vec2::new(0.0, 0.0));
}

#[test]
fn animated_path_anchor_follows_morph() {
    let p = path_node().path(tween(
        rect_path(0.0, 0.0, 100.0, 50.0),
        rect_path(10.0, 20.0, 80.0, 40.0),
    ));
    let bottom = p.anchor(AnchorKind::Bottom);
    assert_eq!(bottom.resolve(0.0), Vec2::new(50.0, 50.0));
    assert_eq!(bottom.resolve(1.0), Vec2::new(50.0, 60.0));
}

#[test]
fn circle_center_anchor_matches_resolved_xy() {
    let c = circle()
        .x(tween(10.0, 50.0))
        .y(tween(20.0, 100.0))
        .radius(5.0);
    let center = c.anchor(AnchorKind::Center);
    assert_eq!(center.resolve(0.0), Vec2::new(10.0, 20.0));
    assert_eq!(center.resolve(0.5), Vec2::new(30.0, 60.0));
}

#[test]
fn rect_anchor_produces_different_positions_for_each_kind() {
    let r = rect().x(0.0).y(0.0).width(100.0).height(100.0);
    let positions: Vec<Vec2> = [
        AnchorKind::Center,
        AnchorKind::Top,
        AnchorKind::Bottom,
        AnchorKind::Left,
        AnchorKind::Right,
    ]
    .iter()
    .map(|k| r.anchor(*k).resolve(0.0))
    .collect();
    // all 5 positions should be different
    let mut unique = positions.clone();
    unique.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap()
            .then(a.y.partial_cmp(&b.y).unwrap())
    });
    unique.dedup();
    assert_eq!(
        unique.len(),
        5,
        "all five anchor kinds should produce distinct positions"
    );
}
