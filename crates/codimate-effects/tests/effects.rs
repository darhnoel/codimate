use codimate_core::{
    circle, ease_in, path_node, rect, rect_path, scene, Color, ConcreteNode, SceneTransformError,
};
use codimate_effects::{fade_in, fade_out, reveal, show, try_transform};

fn circle_alpha(node: &ConcreteNode) -> f32 {
    match node {
        ConcreteNode::Circle(circle) => circle.fill.a,
        other => panic!("expected circle, got {other:?}"),
    }
}

fn transform_error(
    result: Result<codimate_effects::Effect, SceneTransformError>,
) -> SceneTransformError {
    match result {
        Ok(_) => panic!("expected transform error"),
        Err(error) => error,
    }
}

#[test]
fn show_resolves_the_scene_unchanged() {
    let effect = show(scene().node(circle().x(12.0).radius(4.0).fill(Color::RED)));

    let resolved = effect.resolve(0.5);

    assert_eq!(resolved.children.len(), 1);
    match &resolved.children[0] {
        ConcreteNode::Circle(circle) => {
            assert_eq!(circle.x, 12.0);
            assert_eq!(circle.radius, 4.0);
            assert_eq!(circle.fill, Color::RED);
        }
        other => panic!("expected circle, got {other:?}"),
    }
}

#[test]
fn fade_in_changes_alpha_without_changing_scene_shape() {
    let effect = fade_in(scene().node(circle().fill(Color::RED)));

    assert_eq!(circle_alpha(&effect.resolve(0.0).children[0]), 0.0);
    assert_eq!(circle_alpha(&effect.resolve(0.5).children[0]), 0.5);
    assert_eq!(circle_alpha(&effect.resolve(1.0).children[0]), 1.0);
}

#[test]
fn fade_out_changes_alpha_without_changing_scene_shape() {
    let effect = fade_out(scene().node(circle().fill(Color::RED)));

    assert_eq!(circle_alpha(&effect.resolve(0.0).children[0]), 1.0);
    assert_eq!(circle_alpha(&effect.resolve(0.5).children[0]), 0.5);
    assert_eq!(circle_alpha(&effect.resolve(1.0).children[0]), 0.0);
}

#[test]
fn effect_ease_reuses_core_easing_curves() {
    let effect = fade_in(scene().node(circle().fill(Color::RED))).ease(ease_in);

    assert_eq!(circle_alpha(&effect.resolve(0.5).children[0]), 0.25);
}

#[test]
fn reveal_prefixes_path_like_nodes() {
    let effect = reveal(scene().node(path_node().path(rect_path(0.0, 0.0, 100.0, 50.0))));

    match &effect.resolve(0.0).children[0] {
        ConcreteNode::Path(path) => assert!(path.path.segments.is_empty()),
        other => panic!("expected path, got {other:?}"),
    }
    match &effect.resolve(1.0).children[0] {
        ConcreteNode::Path(path) => assert_eq!(path.path.segments.len(), 4),
        other => panic!("expected path, got {other:?}"),
    }
}

#[test]
fn transform_interpolates_matching_scene_shapes() {
    let from = scene().node(rect().x(0.0).y(10.0).width(20.0).height(30.0));
    let to = scene().node(rect().x(100.0).y(20.0).width(40.0).height(50.0));
    let effect = try_transform(from, to).expect("matching rect scenes should transform");

    match &effect.resolve(0.5).children[0] {
        ConcreteNode::Rect(rect) => {
            assert_eq!(rect.x, 50.0);
            assert_eq!(rect.y, 15.0);
            assert_eq!(rect.width, 30.0);
            assert_eq!(rect.height, 40.0);
        }
        other => panic!("expected rect, got {other:?}"),
    }
}

#[test]
fn transform_rejects_mismatched_child_counts() {
    let from = scene().node(circle());
    let to = scene().node(circle()).node(circle());

    assert_eq!(
        transform_error(try_transform(from, to)),
        SceneTransformError::ChildCountMismatch { from: 1, to: 2 }
    );
}

#[test]
fn transform_rejects_mismatched_node_kinds() {
    let from = scene().node(circle());
    let to = scene().node(rect());

    assert_eq!(
        transform_error(try_transform(from, to)),
        SceneTransformError::NodeKindMismatch {
            index: 0,
            from: "Circle",
            to: "Rect"
        }
    );
}

#[test]
fn effect_animates_at_explicit_timing_boundary() {
    let animation = fade_in(scene().node(circle().fill(Color::RED))).animate("fade", 0.75);

    assert_eq!(animation.name(), "fade");
    assert_eq!(animation.duration(), 0.75);
    assert_eq!(circle_alpha(&animation.resolve(0.5).children[0]), 0.5);
}
