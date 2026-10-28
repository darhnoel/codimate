/// A named point on a shape's boundary.
///
/// Anchors are pure — they resolve into `Vec2` from a shape's geometry at time `t`.
///
/// ```
/// use codimate_core::{circle, AnchorKind};
/// let c = circle().x(100.0).y(200.0).radius(50.0);
/// let top = c.anchor(AnchorKind::Top);
/// assert_eq!(top.resolve(0.0), codimate_core::Vec2::new(100.0, 150.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnchorKind {
    Center,
    Top,
    Bottom,
    Left,
    Right,
}
