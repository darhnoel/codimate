//! Layer 1 tests: Animated::ease + built-in curves.

use codimate_core::{back, ease_in, ease_in_out, ease_out, tween};

/// Golden test: ease_in (t*t) remaps time, so the midpoint lands at 25, not 50.
/// Endpoints stay pinned at 0 and 100.
#[test]
fn ease_in_remaps_midpoint() {
    let r = tween(0.0, 100.0).ease(ease_in);
    assert_eq!(r.resolve(0.0), 0.0);
    assert_eq!(r.resolve(0.5), 25.0); // curve(0.5)=0.25 -> lerp -> 25
    assert_eq!(r.resolve(1.0), 100.0);
}

/// ease_out is the mirror: midpoint lands at 75.
#[test]
fn ease_out_remaps_midpoint() {
    let r = tween(0.0, 100.0).ease(ease_out);
    assert_eq!(r.resolve(0.5), 75.0); // curve(0.5)=0.75 -> lerp -> 75
}

/// ease_in_out is symmetric: still 50 at the midpoint, but pinned ends.
#[test]
fn ease_in_out_is_symmetric() {
    let r = tween(0.0, 100.0).ease(ease_in_out);
    assert_eq!(r.resolve(0.0), 0.0);
    assert_eq!(r.resolve(0.5), 50.0);
    assert_eq!(r.resolve(1.0), 100.0);
}

/// Overshoot guard: `back` pushes past the target near the end. Proves easing
/// relies on tween's no-clamp extrapolation — if clamping is ever re-added,
/// this fails.
#[test]
fn back_overshoots_past_target() {
    let r = tween(0.0, 100.0).ease(back);
    assert!(r.resolve(0.9) > 100.0, "back should overshoot above the target");
    assert_eq!(r.resolve(1.0), 100.0); // still settles exactly on target
}

/// A user can pass their own curve with no ceremony.
#[test]
fn custom_curve_closure() {
    let r = tween(0.0, 100.0).ease(|t| t * t * t); // cubic ease-in
    assert_eq!(r.resolve(0.5), 12.5); // 0.125 -> lerp -> 12.5
}
