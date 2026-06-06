//! codimate-layout — Slot-based View layout helpers.
//!
//! A `Slot` is a View-only layout position with a top-left point and size.
//! It is not concept state, not a Scene Node, and not a runtime layout pass.
//! Use Slots in View code before reaching for raw coordinates.

use codimate_core::{
    text, AnchorKind, Animated, Color, IntoAnimated, Path, PathNode, SceneNode, Segment, Style,
    Text, Vec2,
};

/// A View-only layout position with an animated top-left corner and fixed size.
///
/// The `origin` may be constant (a plain `Vec2`) or animated (e.g. via `tween`)
/// so that derived slots (children, neighbors) follow the parent's motion.
#[derive(Clone)]
pub struct Slot {
    origin: Animated<Vec2>,
    pub w: f32,
    pub h: f32,
}

impl Slot {
    /// A static slot at a fixed position.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Slot {
            origin: Vec2::new(x, y).into_animated(),
            w,
            h,
        }
    }

    /// The top-left corner (the slot's origin).
    pub fn top_left(&self) -> Animated<Vec2> {
        self.origin.clone()
    }

    /// The slot's center point.
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
    pub fn top(&self) -> Animated<Vec2> {
        let origin = self.origin.clone();
        let hw = self.w / 2.0;
        Animated::new(move |t| {
            let o = origin.resolve(t);
            Vec2::new(o.x + hw, o.y)
        })
    }

    /// Center of the bottom edge.
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
    pub fn left(&self) -> Animated<Vec2> {
        let origin = self.origin.clone();
        let hh = self.h / 2.0;
        Animated::new(move |t| {
            let o = origin.resolve(t);
            Vec2::new(o.x, o.y + hh)
        })
    }

    /// Center of the right edge.
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

// ---------------------------------------------------------------------------
// Row / Column free-function builders
// ---------------------------------------------------------------------------

/// Build a row of slots.
pub fn row() -> RowBuilder {
    RowBuilder::new()
}

/// Build a column of slots.
pub fn column() -> ColumnBuilder {
    ColumnBuilder::new()
}

fn build_slots(
    origin: Option<Animated<Vec2>>,
    cell_size: Option<Vec2>,
    gap: f32,
    count: usize,
    is_column: bool,
) -> Vec<Slot> {
    let origin = origin.unwrap_or(Vec2::new(0.0, 0.0).into_animated());
    let sz = cell_size.unwrap_or(Vec2::new(100.0, 40.0));
    (0..count)
        .map(|i| {
            let origin = origin.clone();
            let offset = i as f32 * ((if is_column { sz.y } else { sz.x }) + gap);
            Slot {
                origin: Animated::new(move |t| {
                    let o = origin.resolve(t);
                    if is_column {
                        Vec2::new(o.x, o.y + offset)
                    } else {
                        Vec2::new(o.x + offset, o.y)
                    }
                }),
                w: sz.x,
                h: sz.y,
            }
        })
        .collect()
}

macro_rules! define_builder {
    ($name:ident, $slots:ident, $is_col:expr) => {
        pub struct $name {
            origin: Option<Animated<Vec2>>,
            cell_size: Option<Vec2>,
            gap: f32,
        }

        impl $name {
            fn new() -> Self {
                $name {
                    origin: None,
                    cell_size: None,
                    gap: 0.0,
                }
            }

            pub fn origin(mut self, origin: impl IntoAnimated<Vec2>) -> Self {
                self.origin = Some(origin.into_animated());
                self
            }

            pub fn cell_size(mut self, size: Vec2) -> Self {
                self.cell_size = Some(size);
                self
            }

            pub fn gap(mut self, gap: f32) -> Self {
                self.gap = gap;
                self
            }

            pub fn count(self, count: usize) -> $slots {
                $slots(build_slots(
                    self.origin,
                    self.cell_size,
                    self.gap,
                    count,
                    $is_col,
                ))
            }
        }
    };
}

macro_rules! define_slots {
    ($name:ident) => {
        pub struct $name(Vec<Slot>);

        impl $name {
            pub fn len(&self) -> usize {
                self.0.len()
            }

            pub fn remove(self, index: usize) -> Slot {
                self.0.into_iter().nth(index).unwrap()
            }
        }

        impl std::ops::Index<usize> for $name {
            type Output = Slot;
            fn index(&self, index: usize) -> &Slot {
                &self.0[index]
            }
        }
    };
}

define_builder!(RowBuilder, RowSlots, false);
define_builder!(ColumnBuilder, ColumnSlots, true);
define_slots!(RowSlots);
define_slots!(ColumnSlots);

// ---------------------------------------------------------------------------
// Box builder — utility for rounded-rect PathNodes from Slots
// ---------------------------------------------------------------------------

/// Build a styled rounded-rect PathNode from a slot or animated center.
///
/// Created by [`box_in`] or [`box_at`].
pub struct BoxBuilder {
    center: Animated<Vec2>,
    size: Vec2,
    radius: Animated<f32>,
    fill: Animated<Color>,
    stroke_width: Animated<f32>,
    stroke_color: Animated<Color>,
}

/// Construct a `BoxBuilder` from a Slot (uses the slot's full bounds).
pub fn box_in(slot: &Slot) -> BoxBuilder {
    box_at(slot.center(), Vec2::new(slot.w, slot.h))
}

/// Construct a `BoxBuilder` from an animated center and fixed size.
pub fn box_at(center: impl IntoAnimated<Vec2>, size: Vec2) -> BoxBuilder {
    BoxBuilder {
        center: center.into_animated(),
        size,
        radius: 0.0.into_animated(),
        fill: Color::WHITE.into_animated(),
        stroke_width: 0.0.into_animated(),
        stroke_color: Color::WHITE.into_animated(),
    }
}

impl BoxBuilder {
    pub fn radius(mut self, r: impl IntoAnimated<f32>) -> Self {
        self.radius = r.into_animated();
        self
    }

    pub fn style(mut self, s: impl IntoAnimated<Style>) -> Self {
        let s = s.into_animated();
        let fill = s.clone();
        let sw = s.clone();
        self.fill = Animated::new(move |t| fill.resolve(t).fill);
        self.stroke_width = Animated::new(move |t| sw.resolve(t).stroke_width);
        self.stroke_color = Animated::new(move |t| s.resolve(t).stroke_color);
        self
    }

    pub fn fill(mut self, c: impl IntoAnimated<Color>) -> Self {
        self.fill = c.into_animated();
        self
    }

    pub fn stroke(
        mut self,
        width: impl IntoAnimated<f32>,
        color: impl IntoAnimated<Color>,
    ) -> Self {
        self.stroke_width = width.into_animated();
        self.stroke_color = color.into_animated();
        self
    }

    /// Build the final `PathNode`.
    pub fn into_node(self) -> PathNode {
        let center = self.center;
        let w = self.size.x;
        let h = self.size.y;
        let radius = self.radius;
        let path = Animated::new(move |t| {
            let c = center.resolve(t);
            let r = radius.resolve(t);
            rounded_rect_path(c.x - w / 2.0, c.y - h / 2.0, w, h, r)
        });
        PathNode::new()
            .path(path)
            .fill(self.fill)
            .stroke(self.stroke_width, self.stroke_color)
    }
}

impl From<BoxBuilder> for SceneNode {
    fn from(b: BoxBuilder) -> Self {
        SceneNode::Path(b.into_node())
    }
}

/// Build a rounded-rect path. Always produces 8 segments (4 lines + 4 corner cubics),
/// even at `r = 0` where corner cubics become degenerate.
fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let r = r.max(0.0);
    let k = r * 0.552_284_9;
    Path {
        segments: vec![
            Segment::Line(Vec2::new(x + r, y), Vec2::new(x + w - r, y)),
            Segment::Cubic(
                Vec2::new(x + w - r, y),
                Vec2::new(x + w - r + k, y),
                Vec2::new(x + w, y + r - k),
                Vec2::new(x + w, y + r),
            ),
            Segment::Line(Vec2::new(x + w, y + r), Vec2::new(x + w, y + h - r)),
            Segment::Cubic(
                Vec2::new(x + w, y + h - r),
                Vec2::new(x + w, y + h - r + k),
                Vec2::new(x + w - r + k, y + h),
                Vec2::new(x + w - r, y + h),
            ),
            Segment::Line(Vec2::new(x + w - r, y + h), Vec2::new(x + r, y + h)),
            Segment::Cubic(
                Vec2::new(x + r, y + h),
                Vec2::new(x + r - k, y + h),
                Vec2::new(x, y + h - r + k),
                Vec2::new(x, y + h - r),
            ),
            Segment::Line(Vec2::new(x, y + h - r), Vec2::new(x, y + r)),
            Segment::Cubic(
                Vec2::new(x, y + r),
                Vec2::new(x, y + r - k),
                Vec2::new(x + r - k, y),
                Vec2::new(x + r, y),
            ),
        ],
        closed: true,
    }
}

// ---------------------------------------------------------------------------
// Text helpers
// ---------------------------------------------------------------------------

/// A `Text` node centered within the given slot, with `TextAlign::Center`.
pub fn centered_text(
    slot: &Slot,
    content: impl Into<String>,
    font_size: f32,
    fill: impl IntoAnimated<Color>,
) -> Text {
    let content = content.into();
    let center = slot.center();
    let cx = center.clone().map(|v| v.x);
    let cy = center.map(move |v| v.y + font_size * 0.34);
    text()
        .x(cx)
        .y(cy)
        .text(content)
        .font_size(font_size)
        .fill(fill)
        .align(codimate_core::TextAlign::Center)
}

/// A `Text` node whose top-left is positioned at the slot's top-left.
pub fn text_at(
    slot: &Slot,
    content: impl Into<String>,
    font_size: f32,
    fill: impl IntoAnimated<Color>,
) -> Text {
    let content = content.into();
    let origin = slot.origin.clone();
    let x = origin.clone().map(|v| v.x);
    let y = origin.map(|v| v.y);
    text()
        .x(x)
        .y(y)
        .text(content)
        .font_size(font_size)
        .fill(fill)
}

// ---------------------------------------------------------------------------
// Viewport
// ---------------------------------------------------------------------------

/// The output size a concrete Scene should be laid out within.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// The viewport itself as a Slot (full screen).
    pub fn slot(&self) -> Slot {
        Slot::new(0.0, 0.0, self.width, self.height)
    }
}

/// A concrete Scene paired with viewport layout context.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutFrame {
    pub viewport: Viewport,
    pub scene: codimate_core::ConcreteScene,
}

/// Pure layout boundary: no rendering, no I/O.
pub fn layout_scene(scene: codimate_core::ConcreteScene, viewport: Viewport) -> LayoutFrame {
    LayoutFrame { viewport, scene }
}
