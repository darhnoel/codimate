//! codimate-layout — Slot-based View layout helpers.
//!
//! A `Slot` is a View-only layout position with a top-left point and size.
//! It is not concept state, not a Scene Node, and not a runtime layout pass.
//! Use Slots in View code before reaching for raw coordinates.

use codimate_core::{AnchorKind, Animated, IntoAnimated, Vec2};

/// A View-only layout position with an animated top-left corner and fixed size.
///
/// The `origin` may be constant (a plain `Vec2`) or animated (e.g. via `tween`)
/// so that derived slots (children, neighbors) follow the parent's motion.
#[derive(Clone)]
pub struct Slot {
    pub(crate) origin: Animated<Vec2>,
    pub w: f32,
    pub h: f32,
}

impl Slot {
    /// A static slot at a fixed position.
    #[must_use]
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Slot {
            origin: Vec2::new(x, y).into_animated(),
            w,
            h,
        }
    }

    /// The top-left corner (the slot's origin).
    #[must_use]
    pub fn top_left(&self) -> Animated<Vec2> {
        self.origin.clone()
    }

    /// The slot's center point.
    #[must_use]
    pub fn center(&self) -> Animated<Vec2> {
        let origin = self.origin.clone();
        let hw = self.w / 2.0;
        let hh = self.h / 2.0;
        Animated::new(move |t| {
            let o = origin.resolve(t);
            Vec2::new(o.x + hw, o.y + hh)
        })
    }

    /// Center of the top edge.
    #[must_use]
    pub fn top(&self) -> Animated<Vec2> {
        let origin = self.origin.clone();
        let hw = self.w / 2.0;
        Animated::new(move |t| {
            let o = origin.resolve(t);
            Vec2::new(o.x + hw, o.y)
        })
    }

    /// Center of the bottom edge.
    #[must_use]
    pub fn bottom(&self) -> Animated<Vec2> {
        let origin = self.origin.clone();
        let hw = self.w / 2.0;
        let h = self.h;
        Animated::new(move |t| {
            let o = origin.resolve(t);
            Vec2::new(o.x + hw, o.y + h)
        })
    }

    /// Center of the left edge.
    #[must_use]
    pub fn left(&self) -> Animated<Vec2> {
        let origin = self.origin.clone();
        let hh = self.h / 2.0;
        Animated::new(move |t| {
            let o = origin.resolve(t);
            Vec2::new(o.x, o.y + hh)
        })
    }

    /// Center of the right edge.
    #[must_use]
    pub fn right(&self) -> Animated<Vec2> {
        let origin = self.origin.clone();
        let hh = self.h / 2.0;
        let w = self.w;
        Animated::new(move |t| {
            let o = origin.resolve(t);
            Vec2::new(o.x + w, o.y + hh)
        })
    }

    /// A named anchor point derived from the slot bounds.
    #[must_use]
    pub fn anchor(&self, kind: AnchorKind) -> Animated<Vec2> {
        match kind {
            AnchorKind::Center => self.center(),
            AnchorKind::Top => self.top(),
            AnchorKind::Bottom => self.bottom(),
            AnchorKind::Left => self.left(),
            AnchorKind::Right => self.right(),
        }
    }

    /// A child slot centered within this one.
    #[must_use]
    pub fn centered_child(&self, size: Vec2) -> Self {
        let origin = self.origin.clone();
        let pw = self.w;
        let ph = self.h;
        Slot {
            origin: Animated::new(move |t| {
                let o = origin.resolve(t);
                Vec2::new(o.x + (pw - size.x) / 2.0, o.y + (ph - size.y) / 2.0)
            }),
            w: size.x,
            h: size.y,
        }
    }

    /// Divide this slot into a row of `count` children, left-aligned inside the parent.
    #[must_use]
    pub fn row(&self, size: Vec2, gap: f32, count: usize) -> Vec<Self> {
        (0..count)
            .map(|i| {
                let origin = self.origin.clone();
                let ox = i as f32 * (size.x + gap);
                Slot {
                    origin: Animated::new(move |t| {
                        let o = origin.resolve(t);
                        Vec2::new(o.x + ox, o.y)
                    }),
                    w: size.x,
                    h: size.y,
                }
            })
            .collect()
    }

    /// Divide this slot into a column of `count` children, top-aligned inside the parent.
    #[must_use]
    pub fn column(&self, size: Vec2, gap: f32, count: usize) -> Vec<Self> {
        (0..count)
            .map(|i| {
                let origin = self.origin.clone();
                let oy = i as f32 * (size.y + gap);
                Slot {
                    origin: Animated::new(move |t| {
                        let o = origin.resolve(t);
                        Vec2::new(o.x, o.y + oy)
                    }),
                    w: size.x,
                    h: size.y,
                }
            })
            .collect()
    }

    /// A new slot of the given size placed below this one with `gap`, centered horizontally.
    #[must_use]
    pub fn below(&self, size: Vec2, gap: f32) -> Self {
        let origin = self.origin.clone();
        let pw = self.w;
        let ph = self.h;
        Slot {
            origin: Animated::new(move |t| {
                let o = origin.resolve(t);
                Vec2::new(o.x + (pw - size.x) / 2.0, o.y + ph + gap)
            }),
            w: size.x,
            h: size.y,
        }
    }

    /// A new slot of the given size placed above this one with `gap`, centered horizontally.
    #[must_use]
    pub fn above(&self, size: Vec2, gap: f32) -> Self {
        let origin = self.origin.clone();
        let pw = self.w;
        Slot {
            origin: Animated::new(move |t| {
                let o = origin.resolve(t);
                Vec2::new(o.x + (pw - size.x) / 2.0, o.y - gap - size.y)
            }),
            w: size.x,
            h: size.y,
        }
    }

    /// A new slot of the given size placed to the left with `gap`, centered vertically.
    #[must_use]
    pub fn left_of(&self, size: Vec2, gap: f32) -> Self {
        let origin = self.origin.clone();
        let ph = self.h;
        Slot {
            origin: Animated::new(move |t| {
                let o = origin.resolve(t);
                Vec2::new(o.x - gap - size.x, o.y + (ph - size.y) / 2.0)
            }),
            w: size.x,
            h: size.y,
        }
    }

    /// A new slot of the given size placed to the right with `gap`, centered vertically.
    #[must_use]
    pub fn right_of(&self, size: Vec2, gap: f32) -> Self {
        let origin = self.origin.clone();
        let pw = self.w;
        let ph = self.h;
        Slot {
            origin: Animated::new(move |t| {
                let o = origin.resolve(t);
                Vec2::new(o.x + pw + gap, o.y + (ph - size.y) / 2.0)
            }),
            w: size.x,
            h: size.y,
        }
    }
}
