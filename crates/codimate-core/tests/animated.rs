//! Layer 1 foundation tests: Animated<T> + IntoAnimated<T>.

use codimate_core::{Animated, Color, IntoAnimated, Vec2};

/// Golden test: a constant ignores `t` — same value at every moment.
#[test]
fn constant_resolves_invariant_across_t() {
    let a = 5.0_f32.into_animated();
    assert_eq!(a.resolve(0.0), 5.0);
    assert_eq!(a.resolve(0.5), 5.0);
    assert_eq!(a.resolve(1.0), 5.0);
}

/// Golden test: feeding an Animated back through into_animated() (the identity
/// path via std's reflexive From) preserves its behavior — no double-wrapping.
#[test]
fn identity_passthrough_preserves_value() {
    let a = 5.0_f32.into_animated();
    let b = a.into_animated();
    assert_eq!(b.resolve(0.7), 5.0);
}

/// The labeled escape hatch builds custom motion from a pure closure.
#[test]
fn new_builds_custom_motion() {
    let a = Animated::new(|t| 50.0 + t * 50.0);
    assert_eq!(a.resolve(0.0), 50.0);
    assert_eq!(a.resolve(1.0), 100.0);
}

/// The closed leaf set { f32, Color, Vec2 } all reach Animated with no ceremony.
#[test]
fn color_and_vec2_are_leaves() {
    let c: Animated<Color> = Color::RED.into_animated();
    assert_eq!(c.resolve(0.3), Color::RED);

    let p = Vec2::new(1.0, 2.0).into_animated();
    assert_eq!(p.resolve(0.9), Vec2::new(1.0, 2.0));
}
