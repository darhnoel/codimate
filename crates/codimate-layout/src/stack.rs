//! codimate-layout — Slot-based View layout helpers.
//!
//! A `Slot` is a View-only layout position with a top-left point and size.
//! It is not concept state, not a Scene Node, and not a runtime layout pass.
//! Use Slots in View code before reaching for raw coordinates.

use crate::Slot;
use codimate_core::{Animated, IntoAnimated, Vec2};

// ---------------------------------------------------------------------------
// Row / Column free-function builders
// ---------------------------------------------------------------------------

/// Build a row of slots.
#[must_use]
pub fn row() -> RowBuilder {
    RowBuilder::new()
}

/// Build a column of slots.
#[must_use]
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

            #[must_use]
            pub fn origin(mut self, origin: impl IntoAnimated<Vec2>) -> Self {
                self.origin = Some(origin.into_animated());
                self
            }

            #[must_use]
            pub fn cell_size(mut self, size: Vec2) -> Self {
                self.cell_size = Some(size);
                self
            }

            #[must_use]
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

            /// Takes the slot at `index` out of the list.
            ///
            /// # Panics
            ///
            /// Panics if `index` is not less than `len()`.
            pub fn remove(self, index: usize) -> Slot {
                self.0
                    .into_iter()
                    .nth(index)
                    .expect("index is less than len()")
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
