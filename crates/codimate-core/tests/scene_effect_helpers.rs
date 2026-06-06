use codimate_core::{
    circle, connection, ease_in, path_node, rect, rect_path, scene, Color, ConcreteNode,
    SceneTransformError, Vec2,
};

fn circle_alpha(node: &ConcreteNode) -> f32 {
    match node {
        ConcreteNode::Circle(circle) => circle.fill.a,
        other => panic!("expected circle, got {other:?}"),
    }
}

fn transform_error(
    result: Result<codimate_core::Scene, SceneTransformError>,
) -> SceneTransformError {
    match result {
        Ok(_) => panic!("expected transform error"),
        Err(error) => error,
    }
}

#[test]
fn scene_opacity_multiplies_color_alpha_without_changing_children() {
    let s = scene().node(circle().fill(Color::RED)).with_opacity(0.25);
    let resolved = s.resolve(0.0);

    assert_eq!(resolved.children.len(), 1);
    assert_eq!(circle_alpha(&resolved.children[0]), 0.25);
}

#[test]
fn scene_ease_remaps_local_time_for_all_children() {
    let s = scene()
        .node(circle().x(codimate_core::tween(0.0, 100.0)))
        .ease(ease_in);

    match &s.resolve(0.5).children[0] {
        ConcreteNode::Circle(circle) => assert_eq!(circle.x, 25.0),
        other => panic!("expected circle, got {other:?}"),
    }
}

#[test]
fn scene_try_lerp_to_interpolates_matching_children() {
    let from = scene().node(rect().x(0.0).width(20.0));
    let to = scene().node(rect().x(100.0).width(40.0));
    let transformed = from.try_lerp_to(&to).expect("matching scenes should lerp");

    match &transformed.resolve(0.5).children[0] {
        ConcreteNode::Rect(rect) => {
            assert_eq!(rect.x, 50.0);
            assert_eq!(rect.width, 30.0);
        }
        other => panic!("expected rect, got {other:?}"),
    }
}

#[test]
fn scene_try_lerp_to_rejects_mismatched_kinds() {
    let from = scene().node(circle());
    let to = scene().node(rect());

    assert_eq!(
        transform_error(from.try_lerp_to(&to)),
        SceneTransformError::NodeKindMismatch {
            index: 0,
            from: "Circle",
            to: "Rect"
        }
    );
}

#[test]
fn scene_reveal_fades_non_path_nodes() {
    let s = scene().node(rect().fill(Color::RED)).reveal(0.25);

    match &s.resolve(0.0).children[0] {
        ConcreteNode::Rect(rect) => assert_eq!(rect.fill.a, 0.25),
        other => panic!("expected rect, got {other:?}"),
    }
}

#[test]
fn scene_reveal_prefixes_path_nodes() {
    let s = scene()
        .node(path_node().path(rect_path(0.0, 0.0, 100.0, 50.0)))
        .reveal(codimate_core::tween(0.0, 1.0));

    match &s.resolve(0.0).children[0] {
        ConcreteNode::Path(path) => assert!(path.path.segments.is_empty()),
        other => panic!("expected path, got {other:?}"),
    }
    match &s.resolve(0.5).children[0] {
        ConcreteNode::Path(path) => {
            assert!(!path.path.segments.is_empty());
            assert!(path.path.segments.len() < 4);
        }
        other => panic!("expected path, got {other:?}"),
    }
    match &s.resolve(1.0).children[0] {
        ConcreteNode::Path(path) => assert_eq!(path.path.segments.len(), 4),
        other => panic!("expected path, got {other:?}"),
    }
}

#[test]
fn scene_reveal_prefixes_connections() {
    let conn = connection(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0));
    let s = scene().node(conn).reveal(codimate_core::tween(0.0, 1.0));

    match &s.resolve(0.0).children[0] {
        ConcreteNode::Path(path) => assert!(path.path.segments.is_empty()),
        other => panic!("expected path, got {other:?}"),
    }
    match &s.resolve(0.5).children[0] {
        ConcreteNode::Path(path) => {
            assert_eq!(path.path.segments.len(), 1);
            if let codimate_core::Segment::Line(_, to) = path.path.segments[0] {
                assert!((to.x - 50.0).abs() < 0.1);
            } else {
                panic!("expected line segment");
            }
        }
        other => panic!("expected path, got {other:?}"),
    }
}
