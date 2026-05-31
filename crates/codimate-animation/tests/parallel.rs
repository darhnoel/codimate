//! Layer 3 tests: Parallel plays named Animations together.

use codimate_animation::{animation, parallel};
use codimate_core::{circle, rect, scene, tween, Color, ConcreteNode};

/// Golden test: a Parallel has its own name, uses the longest child duration,
/// resolves every child at the same elapsed time, and holds shorter children at
/// their final state after they finish.
#[test]
fn parallel_uses_longest_duration_and_combines_children() {
    let pulse = animation(
        "pulse",
        2.0,
        scene().node(circle().x(tween(0.0, 100.0)).radius(10.0).fill(Color::RED)),
    );
    let bar = animation(
        "bar",
        4.0,
        scene().node(rect().width(tween(10.0, 50.0)).height(5.0).fill(Color::RED)),
    );

    let p = parallel("demo", [pulse, bar]);

    assert_eq!(p.name(), "demo");
    assert_eq!(p.duration(), 4.0);

    let mid = p.resolve(0.5);
    assert_eq!(mid.children.len(), 2);
    match &mid.children[0] {
        ConcreteNode::Circle(circle) => assert_eq!(circle.x, 100.0),
        other => panic!("expected held final circle, got {other:?}"),
    }
    match &mid.children[1] {
        ConcreteNode::Rect(rect) => assert_eq!(rect.width, 30.0),
        other => panic!("expected halfway rect, got {other:?}"),
    }
}
