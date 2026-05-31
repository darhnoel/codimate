//! Layer 3 tests: Sequence plays named Animations back-to-back.

use codimate_animation::{animation, sequence};
use codimate_core::{circle, rect, scene, tween, Color, ConcreteNode};

/// Golden test: a Sequence has its own name, sums child durations, and resolves
/// normalized time into the active child Animation's local normalized time.
#[test]
fn sequence_sums_durations_and_resolves_active_animation() {
    let intro = animation(
        "intro",
        2.0,
        scene().node(circle().x(tween(0.0, 100.0)).radius(10.0).fill(Color::RED)),
    );
    let outro = animation(
        "outro",
        3.0,
        scene().node(rect().width(tween(10.0, 40.0)).height(5.0).fill(Color::RED)),
    );

    let s = sequence("demo", [intro, outro]);

    assert_eq!(s.name(), "demo");
    assert_eq!(s.duration(), 5.0);

    match &s.resolve(0.2).children[0] {
        ConcreteNode::Circle(circle) => assert_eq!(circle.x, 50.0),
        other => panic!("expected intro circle, got {other:?}"),
    }

    match &s.resolve(0.4).children[0] {
        ConcreteNode::Rect(rect) => assert_eq!(rect.width, 10.0),
        other => panic!("expected outro rect at boundary, got {other:?}"),
    }

    match &s.resolve(1.0).children[0] {
        ConcreteNode::Rect(rect) => assert_eq!(rect.width, 40.0),
        other => panic!("expected final outro rect, got {other:?}"),
    }
}
