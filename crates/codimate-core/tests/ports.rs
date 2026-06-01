//! Layer 2 tests: evenly-divided anchor Ports on Rect (fan-in connections).

use codimate_core::{rect, tween, AnchorKind, Vec2};

/// Two ports along the bottom edge land at 1/4 and 3/4 of the width — evenly
/// spaced with margins, so fan-in arrows don't overlap.
#[test]
fn bottom_ports_divide_edge_evenly() {
    // rect at (100,200) size 120x40 → bottom edge y = 240, x in [100, 220]
    let r = rect().x(100.0).y(200.0).width(120.0).height(40.0);
    assert_eq!(
        r.anchor_port(AnchorKind::Bottom, 0, 2).resolve(0.0),
        Vec2::new(130.0, 240.0) // 100 + 0.25 * 120
    );
    assert_eq!(
        r.anchor_port(AnchorKind::Bottom, 1, 2).resolve(0.0),
        Vec2::new(190.0, 240.0) // 100 + 0.75 * 120
    );
}

/// Port 0 of 1 is just the edge midpoint — i.e. the plain anchor for that edge.
#[test]
fn single_port_equals_edge_midpoint() {
    let r = rect().x(0.0).y(0.0).width(80.0).height(40.0);
    assert_eq!(
        r.anchor_port(AnchorKind::Top, 0, 1).resolve(0.0),
        r.anchor(AnchorKind::Top).resolve(0.0),
    );
}

/// Ports are derived from the shape's animated geometry, so they track the box.
#[test]
fn ports_track_animated_geometry() {
    let r = rect().x(tween(0.0, 100.0)).y(0.0).width(50.0).height(60.0);
    // Right edge, port 0 of 2 → 1/4 down the height; x follows rect.x + width.
    let p = r.anchor_port(AnchorKind::Right, 0, 2);
    assert_eq!(p.resolve(0.0), Vec2::new(50.0, 15.0)); // x = 0 + 50, y = 0.25 * 60
    assert_eq!(p.resolve(1.0), Vec2::new(150.0, 15.0)); // x = 100 + 50
}
