//! Layer 2 tests: Rect follows the same Node pattern as Circle.

use codimate_core::{rect, tween, Color};

/// Golden test: resolve maps every property; animated x and width resolve
/// independently while plain y/height/fill pass with no ceremony.
#[test]
fn rect_resolves_each_property() {
    let r = rect()
        .x(tween(0.0, 100.0))
        .y(10.0)
        .width(tween(20.0, 60.0))
        .height(30.0)
        .fill(Color::RED);
    let resolved = r.resolve(0.5);
    assert_eq!(resolved.x, 50.0);
    assert_eq!(resolved.y, 10.0);
    assert_eq!(resolved.width, 40.0);
    assert_eq!(resolved.height, 30.0);
    assert_eq!(resolved.fill, Color::RED);
}

/// Unset properties fall back to defaults:
/// x=y=width=height=0.0, fill=opaque white.
#[test]
fn rect_defaults() {
    let r = rect().resolve(0.0);
    assert_eq!(r.x, 0.0);
    assert_eq!(r.y, 0.0);
    assert_eq!(r.width, 0.0);
    assert_eq!(r.height, 0.0);
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
