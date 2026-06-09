//! The three-slot Primitive — `{ transform, style, geometry }` (ADR 0006).
//!
//! Two concerns are *universal* (every shape has them, identically) and one is
//! *type-specific*:
//!
//! - `transform` — placement & visibility (pos/scale/rotation/opacity/pivot).
//! - `style`     — appearance (fill + stroke), the universal `Style` leaf.
//! - `geometry`  — the pure shape, authored in **local space** (center-origin
//!   for the parametric shapes).
//!
//! The universal setters live on the [`Transformable`] trait so they are defined
//! once, not re-declared per shape. Geometry setters stay inherent on the shape
//! builders (added when the old nodes are retired).

use super::{AnchorKind, ConcreteTransform, TextAlign, Transform};
use crate::path::Path;
use crate::value::*;

/// The pure shape of a primitive — local-space, no placement, no appearance.
#[derive(Clone)]
pub enum Geometry {
    Circle {
        radius: Animated<f32>,
    },
    Rect {
        width: Animated<f32>,
        height: Animated<f32>,
    },
    Path {
        path: Animated<Path>,
    },
    Text {
        text: Animated<String>,
        font_size: Animated<f32>,
        align: TextAlign,
    },
}

/// A `Geometry` resolved at a specific `t` — plain shape data, still local-space.
#[derive(Clone, Debug, PartialEq)]
pub enum ConcreteGeometry {
    Circle {
        radius: f32,
    },
    Rect {
        width: f32,
        height: f32,
    },
    Path {
        path: Path,
    },
    Text {
        text: String,
        font_size: f32,
        align: TextAlign,
    },
}

impl Geometry {
    pub fn circle(radius: impl IntoAnimated<f32>) -> Self {
        Geometry::Circle {
            radius: radius.into_animated(),
        }
    }

    pub fn rect(width: impl IntoAnimated<f32>, height: impl IntoAnimated<f32>) -> Self {
        Geometry::Rect {
            width: width.into_animated(),
            height: height.into_animated(),
        }
    }

    pub fn path(path: impl IntoAnimated<Path>) -> Self {
        Geometry::Path {
            path: path.into_animated(),
        }
    }

    pub fn text(text: impl IntoAnimated<String>) -> Self {
        Geometry::Text {
            text: text.into_animated(),
            font_size: 16.0.into_animated(),
            align: TextAlign::Left,
        }
    }

    /// The local-space point named by `kind`, from this geometry's own bounds at
    /// `t`. Parametric shapes are center-origin; a `Path` uses its bounding box;
    /// `Text` falls back to the origin (it cannot be measured without a renderer).
    pub fn local_anchor(&self, kind: AnchorKind, t: f32) -> Vec2 {
        match self {
            Geometry::Circle { radius } => {
                let r = radius.resolve(t);
                centered_anchor(kind, r, r)
            }
            Geometry::Rect { width, height } => {
                centered_anchor(kind, width.resolve(t) / 2.0, height.resolve(t) / 2.0)
            }
            Geometry::Path { path } => {
                let p = path.resolve(t);
                let (xmin, ymin, xmax, ymax) = p.bounding_box().unwrap_or((0.0, 0.0, 0.0, 0.0));
                let cx = (xmin + xmax) / 2.0;
                let cy = (ymin + ymax) / 2.0;
                match kind {
                    AnchorKind::Center => Vec2::new(cx, cy),
                    AnchorKind::Top => Vec2::new(cx, ymin),
                    AnchorKind::Bottom => Vec2::new(cx, ymax),
                    AnchorKind::Left => Vec2::new(xmin, cy),
                    AnchorKind::Right => Vec2::new(xmax, cy),
                }
            }
            Geometry::Text { .. } => Vec2::new(0.0, 0.0),
        }
    }

    pub fn resolve(&self, t: f32) -> ConcreteGeometry {
        match self {
            Geometry::Circle { radius } => ConcreteGeometry::Circle {
                radius: radius.resolve(t),
            },
            Geometry::Rect { width, height } => ConcreteGeometry::Rect {
                width: width.resolve(t),
                height: height.resolve(t),
            },
            Geometry::Path { path } => ConcreteGeometry::Path {
                path: path.resolve(t),
            },
            Geometry::Text {
                text,
                font_size,
                align,
            } => ConcreteGeometry::Text {
                text: text.resolve(t),
                font_size: font_size.resolve(t),
                align: *align,
            },
        }
    }
}

/// Named anchor on a center-origin half-extent box (y grows downward on screen).
fn centered_anchor(kind: AnchorKind, half_w: f32, half_h: f32) -> Vec2 {
    match kind {
        AnchorKind::Center => Vec2::new(0.0, 0.0),
        AnchorKind::Top => Vec2::new(0.0, -half_h),
        AnchorKind::Bottom => Vec2::new(0.0, half_h),
        AnchorKind::Left => Vec2::new(-half_w, 0.0),
        AnchorKind::Right => Vec2::new(half_w, 0.0),
    }
}

/// A Scene primitive: the universal transform & style wrapped around one shape.
#[derive(Clone)]
pub struct Primitive {
    transform: Transform,
    style: Animated<Style>,
    geometry: Geometry,
}

/// A `Primitive` resolved at `t` — decomposed transform + style + local geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct ConcretePrimitive {
    pub transform: ConcreteTransform,
    pub style: Style,
    pub geometry: ConcreteGeometry,
}

impl Primitive {
    pub fn new(geometry: Geometry) -> Self {
        Primitive {
            transform: Transform::new(),
            style: Style::new().into_animated(),
            geometry,
        }
    }

    pub fn circle(radius: impl IntoAnimated<f32>) -> Self {
        Self::new(Geometry::circle(radius))
    }

    pub fn rect(width: impl IntoAnimated<f32>, height: impl IntoAnimated<f32>) -> Self {
        Self::new(Geometry::rect(width, height))
    }

    pub fn path(path: impl IntoAnimated<Path>) -> Self {
        Self::new(Geometry::path(path))
    }

    pub fn text(text: impl IntoAnimated<String>) -> Self {
        Self::new(Geometry::text(text))
    }

    pub fn geometry(&self) -> &Geometry {
        &self.geometry
    }

    /// `f(t) → ConcretePrimitive`. Resolves the pivot point from the geometry's
    /// own bounds, then the transform, style, and geometry — all at the same `t`.
    pub fn resolve(&self, t: f32) -> ConcretePrimitive {
        let pivot_local = self.geometry.local_anchor(self.transform.pivot_kind(), t);
        ConcretePrimitive {
            transform: self.transform.resolve(t, pivot_local),
            style: self.style.resolve(t),
            geometry: self.geometry.resolve(t),
        }
    }

    /// A named anchor in **world** space, resolved *through* the transform.
    ///
    /// The pivot stays fixed under scale/rotation; `pos` is added on top, so at
    /// the identity transform the anchor equals the local anchor.
    pub fn anchor(&self, kind: AnchorKind) -> Animated<Vec2> {
        let geometry = self.geometry.clone();
        let transform = self.transform.clone();
        let pivot_kind = transform.pivot_kind();
        Animated::new(move |t| {
            let local = geometry.local_anchor(kind, t);
            let pivot = geometry.local_anchor(pivot_kind, t);
            let c = transform.resolve(t, pivot);
            let dx = (local.x - pivot.x) * c.scale.x;
            let dy = (local.y - pivot.y) * c.scale.y;
            let rad = c.rotation_deg.to_radians();
            let (sin, cos) = rad.sin_cos();
            Vec2::new(
                c.pos.x + pivot.x + dx * cos - dy * sin,
                c.pos.y + pivot.y + dx * sin + dy * cos,
            )
        })
    }
}

/// The universal authoring surface, shared by every primitive (ADR 0006 §6).
///
/// Implementors expose mutable access to their `transform` and `style`; the
/// builder setters are default methods, so they are defined once. No macro.
pub trait Transformable: Sized {
    fn transform_mut(&mut self) -> &mut Transform;
    fn style_mut(&mut self) -> &mut Animated<Style>;

    fn pos(mut self, pos: impl IntoAnimated<Vec2>) -> Self {
        let t = std::mem::take(self.transform_mut());
        *self.transform_mut() = t.pos(pos);
        self
    }

    fn x(mut self, x: impl IntoAnimated<f32>) -> Self {
        let t = std::mem::take(self.transform_mut());
        *self.transform_mut() = t.x(x);
        self
    }

    fn y(mut self, y: impl IntoAnimated<f32>) -> Self {
        let t = std::mem::take(self.transform_mut());
        *self.transform_mut() = t.y(y);
        self
    }

    fn scale(mut self, s: impl IntoAnimated<f32>) -> Self {
        let t = std::mem::take(self.transform_mut());
        *self.transform_mut() = t.scale(s);
        self
    }

    fn scale_xy(mut self, s: impl IntoAnimated<Vec2>) -> Self {
        let t = std::mem::take(self.transform_mut());
        *self.transform_mut() = t.scale_xy(s);
        self
    }

    fn rotate(mut self, deg: impl IntoAnimated<f32>) -> Self {
        let t = std::mem::take(self.transform_mut());
        *self.transform_mut() = t.rotate(deg);
        self
    }

    fn opacity(mut self, opacity: impl IntoAnimated<f32>) -> Self {
        let t = std::mem::take(self.transform_mut());
        *self.transform_mut() = t.opacity(opacity);
        self
    }

    fn pivot(mut self, pivot: AnchorKind) -> Self {
        let t = std::mem::take(self.transform_mut());
        *self.transform_mut() = t.pivot(pivot);
        self
    }

    /// Replace the whole style.
    fn style(mut self, style: impl IntoAnimated<Style>) -> Self {
        *self.style_mut() = style.into_animated();
        self
    }

    /// Set just the fill, keeping the rest of the style.
    fn fill(mut self, fill: impl IntoAnimated<Color>) -> Self {
        let style = std::mem::replace(self.style_mut(), Style::new().into_animated());
        let fill = fill.into_animated();
        *self.style_mut() = Animated::new(move |t| {
            let mut s = style.resolve(t);
            s.fill = fill.resolve(t);
            s
        });
        self
    }

    /// Set just the stroke (width + color), keeping the fill.
    fn stroke(
        mut self,
        width: impl IntoAnimated<f32>,
        color: impl IntoAnimated<Color>,
    ) -> Self {
        let style = std::mem::replace(self.style_mut(), Style::new().into_animated());
        let width = width.into_animated();
        let color = color.into_animated();
        *self.style_mut() = Animated::new(move |t| {
            let mut s = style.resolve(t);
            s.stroke_width = width.resolve(t);
            s.stroke_color = color.resolve(t);
            s
        });
        self
    }
}

impl Transformable for Primitive {
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn style_mut(&mut self) -> &mut Animated<Style> {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::tween;

    #[test]
    fn circle_resolves_transform_style_geometry() {
        let c = Primitive::circle(50.0)
            .fill(Color::RED)
            .x(100.0)
            .rotate(90.0)
            .opacity(0.5)
            .resolve(1.0);
        assert_eq!(c.geometry, ConcreteGeometry::Circle { radius: 50.0 });
        assert_eq!(c.transform.pos, Vec2::new(100.0, 0.0));
        assert_eq!(c.transform.rotation_deg, 90.0);
        assert_eq!(c.transform.opacity, 0.5);
        assert_eq!(c.style.fill, Color::RED);
    }

    #[test]
    fn fill_keeps_stroke_and_vice_versa() {
        let s = Primitive::rect(10.0, 10.0)
            .stroke(3.0, Color::CYAN)
            .fill(Color::RED)
            .resolve(0.0)
            .style;
        assert_eq!(s.fill, Color::RED);
        assert_eq!(s.stroke_width, 3.0);
        assert_eq!(s.stroke_color, Color::CYAN);
    }

    #[test]
    fn animated_setters_flow_through() {
        let c = Primitive::circle(tween(10.0, 20.0)).x(tween(0.0, 400.0));
        assert_eq!(c.resolve(0.0).transform.pos, Vec2::new(0.0, 0.0));
        let mid = c.resolve(0.5);
        assert_eq!(mid.transform.pos.x, 200.0);
        assert_eq!(mid.geometry, ConcreteGeometry::Circle { radius: 15.0 });
    }

    #[test]
    fn world_anchor_at_identity_equals_local() {
        // radius 50, no transform: Right anchor is local (50, 0).
        let c = Primitive::circle(50.0);
        assert_eq!(c.anchor(AnchorKind::Right).resolve(0.0), Vec2::new(50.0, 0.0));
    }

    #[test]
    fn world_anchor_follows_translation() {
        let c = Primitive::circle(50.0).x(100.0);
        assert_eq!(
            c.anchor(AnchorKind::Right).resolve(0.0),
            Vec2::new(150.0, 0.0)
        );
    }

    #[test]
    fn world_anchor_respects_rotation_about_center() {
        // 90° rotation about the center pivot turns the Right anchor (50,0) into
        // (0,50) on a y-down screen.
        let c = Primitive::circle(50.0).rotate(90.0);
        let p = c.anchor(AnchorKind::Right).resolve(0.0);
        assert!((p.x - 0.0).abs() < 1e-4, "x was {}", p.x);
        assert!((p.y - 50.0).abs() < 1e-4, "y was {}", p.y);
    }
}
