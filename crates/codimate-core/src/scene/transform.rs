//! The uniform Transform — the placement & visibility every primitive shares.
//!
//! This is the motionity import (ADR 0006): one transform, identical across all
//! shapes — translation, scale, rotation (in **degrees**), opacity, and a pivot.
//! Geometry is authored in local space (center-origin); the Transform places it
//! in the world.
//!
//! `Transform` is geometry-agnostic: it does not know a circle from a rect. The
//! owning `Primitive` resolves the concrete pivot *point* from its geometry's
//! local bounds and hands it to [`Transform::resolve`].

use super::AnchorKind;
use crate::value::{Animated, IntoAnimated, Vec2};

/// The uniform per-primitive transform. Every field is `Animated` and timeless.
///
/// Defaults are the identity: origin position, unit scale, no rotation, fully
/// opaque, center pivot.
#[derive(Clone)]
pub struct Transform {
    pos: Animated<Vec2>,
    scale: Animated<Vec2>,
    rotation: Animated<f32>,
    opacity: Animated<f32>,
    pivot: AnchorKind,
}

/// A `Transform` resolved at a specific `t` — plain values, no renderer.
///
/// `rotation_deg` is degrees; the renderer converts to radians at the tiny-skia
/// boundary. `pivot` is the resolved local-space pivot point. Decomposed by
/// design (ADR 0006 §7) — it becomes a 2×3 matrix only when grouping lands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConcreteTransform {
    pub pos: Vec2,
    pub scale: Vec2,
    pub rotation_deg: f32,
    pub pivot: Vec2,
    pub opacity: f32,
}

impl Transform {
    /// The identity transform: `pos (0,0)`, `scale (1,1)`, `rotation 0`,
    /// `opacity 1`, `pivot Center`.
    pub fn new() -> Self {
        Transform {
            pos: Vec2::new(0.0, 0.0).into_animated(),
            scale: Vec2::new(1.0, 1.0).into_animated(),
            rotation: 0.0.into_animated(),
            opacity: 1.0.into_animated(),
            pivot: AnchorKind::Center,
        }
    }

    /// Set translation. Accepts `(f32, f32)`, `Vec2`, or an `Animated<Vec2>`.
    pub fn pos(mut self, pos: impl IntoAnimated<Vec2>) -> Self {
        self.pos = pos.into_animated();
        self
    }

    /// Set the x component of translation, holding y at resolve time.
    pub fn x(mut self, x: impl IntoAnimated<f32>) -> Self {
        let x = x.into_animated();
        let pos = self.pos;
        self.pos = Animated::new(move |t| Vec2::new(x.resolve(t), pos.resolve(t).y));
        self
    }

    /// Set the y component of translation, holding x at resolve time.
    pub fn y(mut self, y: impl IntoAnimated<f32>) -> Self {
        let y = y.into_animated();
        let pos = self.pos;
        self.pos = Animated::new(move |t| Vec2::new(pos.resolve(t).x, y.resolve(t)));
        self
    }

    /// Uniform scale on both axes.
    pub fn scale(mut self, s: impl IntoAnimated<f32>) -> Self {
        let s = s.into_animated();
        self.scale = Animated::new(move |t| {
            let v = s.resolve(t);
            Vec2::new(v, v)
        });
        self
    }

    /// Non-uniform scale (squash / stretch).
    pub fn scale_xy(mut self, scale: impl IntoAnimated<Vec2>) -> Self {
        self.scale = scale.into_animated();
        self
    }

    /// Rotation in **degrees**.
    pub fn rotate(mut self, deg: impl IntoAnimated<f32>) -> Self {
        self.rotation = deg.into_animated();
        self
    }

    /// Opacity in `0..1`.
    pub fn opacity(mut self, opacity: impl IntoAnimated<f32>) -> Self {
        self.opacity = opacity.into_animated();
        self
    }

    /// The point scale & rotation pivot around (default `Center`).
    pub fn pivot(mut self, pivot: AnchorKind) -> Self {
        self.pivot = pivot;
        self
    }

    /// Which named pivot this transform uses — the owning `Primitive` maps it to
    /// a local point via its geometry bounds.
    pub fn pivot_kind(&self) -> AnchorKind {
        self.pivot
    }

    /// `f(t) → ConcreteTransform`. The caller supplies `pivot_point`, the
    /// local-space point named by [`pivot_kind`](Self::pivot_kind), resolved from
    /// the owning geometry's bounds.
    pub fn resolve(&self, t: f32, pivot_point: Vec2) -> ConcreteTransform {
        ConcreteTransform {
            pos: self.pos.resolve(t),
            scale: self.scale.resolve(t),
            rotation_deg: self.rotation.resolve(t),
            pivot: pivot_point,
            opacity: self.opacity.resolve(t),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::tween;

    #[test]
    fn identity_resolves_to_identity() {
        let c = Transform::new().resolve(0.5, Vec2::new(0.0, 0.0));
        assert_eq!(c.pos, Vec2::new(0.0, 0.0));
        assert_eq!(c.scale, Vec2::new(1.0, 1.0));
        assert_eq!(c.rotation_deg, 0.0);
        assert_eq!(c.opacity, 1.0);
    }

    #[test]
    fn x_and_y_are_independent() {
        let t = Transform::new().x(tween(0.0, 100.0)).y(50.0);
        assert_eq!(t.resolve(0.0, Vec2::new(0.0, 0.0)).pos, Vec2::new(0.0, 50.0));
        assert_eq!(
            t.resolve(1.0, Vec2::new(0.0, 0.0)).pos,
            Vec2::new(100.0, 50.0)
        );
    }

    #[test]
    fn pos_accepts_tuple() {
        let t = Transform::new().pos((10.0, 20.0));
        assert_eq!(t.resolve(0.0, Vec2::new(0.0, 0.0)).pos, Vec2::new(10.0, 20.0));
    }

    #[test]
    fn uniform_scale_sets_both_axes() {
        let t = Transform::new().scale(2.0);
        assert_eq!(t.resolve(0.0, Vec2::new(0.0, 0.0)).scale, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn rotate_is_in_degrees() {
        let t = Transform::new().rotate(tween(0.0, 360.0));
        assert_eq!(t.resolve(0.5, Vec2::new(0.0, 0.0)).rotation_deg, 180.0);
    }

    #[test]
    fn pivot_point_passes_through() {
        let t = Transform::new().pivot(AnchorKind::Top);
        assert_eq!(t.pivot_kind(), AnchorKind::Top);
        let c = t.resolve(0.0, Vec2::new(0.0, -50.0));
        assert_eq!(c.pivot, Vec2::new(0.0, -50.0));
    }
}
