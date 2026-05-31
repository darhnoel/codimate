//! Layer 3 tests: Stagger starts named Animations at fixed offsets.

use codimate_animation::{animation, stagger};
use codimate_core::{circle, rect, scene, tween, Color, ConcreteNode};

/// Golden test: a Stagger has its own name, starts each child after a fixed
/// offset, omits not-yet-started children, and resolves started children using
/// each child's local normalized time.
#[test]
fn stagger_offsets_child_animation_starts() {
    let first = animation(
        "first",
        2.0,
        scene().node(circle().x(tween(0.0, 100.0)).radius(10.0).fill(Color::RED)),
    );
    let second = animation(
        "second",
        2.0,
        scene().node(rect().width(tween(10.0, 40.0)).height(5.0).fill(Color::RED)),
    );

    let s = stagger("demo", 1.0, [first, second]);

    assert_eq!(s.name(), "demo");
    assert_eq!(s.duration(), 3.0);
    assert_eq!(s.resolve(0.0).children.len(), 1);

    let mid = s.resolve(0.5);
    assert_eq!(mid.children.len(), 2);
    match &mid.children[0] {
        ConcreteNode::Circle(circle) => assert_eq!(circle.x, 75.0),
        other => panic!("expected first circle, got {other:?}"),
    }
    match &mid.children[1] {
        ConcreteNode::Rect(rect) => assert_eq!(rect.width, 17.5),
        other => panic!("expected second rect, got {other:?}"),
    }
}
