//! Layer 2 tests: the first Node, Circle, and resolve(t) -> ConcreteCircle.

use codimate_core::{circle, tween, Color};

/// Golden test: resolve maps every property; animated x sweeps while default y
/// holds (proves x/y are independent), and plain values pass with no ceremony.
#[test]
fn circle_resolves_each_property() {
    let c = circle().x(tween(0.0, 100.0)).radius(20.0).fill(Color::RED);
    let r = c.resolve(0.5);
    assert_eq!(r.x, 50.0); // animated x resolved at t
    assert_eq!(r.y, 0.0); // default holds — independent of x
    assert_eq!(r.radius, 20.0); // plain value, no ceremony
    assert_eq!(r.fill, Color::RED);
}

/// Unset properties fall back to defaults: x=y=radius=0.0, fill=opaque white.
#[test]
fn circle_defaults() {
    let r = circle().resolve(0.0);
    assert_eq!(r.x, 0.0);
    assert_eq!(r.y, 0.0);
    assert_eq!(r.radius, 0.0);
    assert_eq!(
        r.fill,
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0
        }
    );
}
