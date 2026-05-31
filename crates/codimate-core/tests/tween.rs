//! Layer 1 tests: tween(a, b) + the Lerp trait.

use codimate_core::{tween, Color, Vec2};

/// Golden test: linear travel from a to b across normalized t.
#[test]
fn tween_lerps_f32() {
    let r = tween(50.0, 100.0);
    assert_eq!(r.resolve(0.0), 50.0);
    assert_eq!(r.resolve(0.5), 75.0);
    assert_eq!(r.resolve(1.0), 100.0);
}

/// Overshoot guard: t past 1.0 extrapolates, it is NOT clamped — this leaves
/// room for future easing to overshoot.
#[test]
fn tween_does_not_clamp_overshoot() {
    let r = tween(50.0, 100.0);
    assert_eq!(r.resolve(1.5), 125.0);
}

/// Vec2 interpolates each component independently.
#[test]
fn tween_lerps_vec2_componentwise() {
    let p = tween(Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0));
    assert_eq!(p.resolve(0.5), Vec2::new(50.0, 25.0));
}

/// Color interpolates each channel independently (red -> blue at the midpoint).
#[test]
fn tween_lerps_color_channelwise() {
    let blue = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    let c = tween(Color::RED, blue);
    assert_eq!(c.resolve(0.5), Color { r: 0.5, g: 0.0, b: 0.5, a: 1.0 });
}

/// Invariant 7 in action: endpoints may themselves be Animated. Each endpoint
/// is resolved at the same t, then interpolated.
#[test]
fn tween_between_two_animated_endpoints() {
    let a = tween(0.0, 10.0);     // at t=0.5 -> 5.0
    let b = tween(100.0, 200.0);  // at t=0.5 -> 150.0
    let mixed = tween(a, b);      // lerp(5.0, 150.0, 0.5) = 77.5
    assert_eq!(mixed.resolve(0.5), 77.5);
}
