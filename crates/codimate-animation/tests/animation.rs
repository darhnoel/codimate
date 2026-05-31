//! Layer 3 tests: Animation is duration plus a pure Scene resolver.

use codimate_animation::animation;
use codimate_core::{circle, scene, tween, Color, ConcreteCircle, ConcreteNode};

/// Golden test: duration lives in Animation, while resolve still uses
/// normalized `t` to produce a ConcreteScene.
#[test]
fn animation_stores_duration_and_resolves_scene() {
    let a = animation(
        2.0,
        scene().node(circle().x(tween(0.0, 100.0)).radius(10.0).fill(Color::RED)),
    );

    assert_eq!(a.duration(), 2.0);
    assert_eq!(
        a.resolve(0.5).children,
        vec![ConcreteNode::Circle(ConcreteCircle {
            x: 50.0,
            y: 0.0,
            radius: 10.0,
            fill: Color::RED,
        })]
    );
}
