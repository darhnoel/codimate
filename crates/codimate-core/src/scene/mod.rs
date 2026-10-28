//! Layer 2 — Scene.
//!
//! What exists at a moment in time. A [`Scene`] is a tree of Nodes; every Node
//! is pure data whose stylistic properties are `Animated<T>`, and resolves into
//! concrete data at a given `t` (Invariant 3: Nodes do not render themselves).
//!
//! Each Node type lives in its own module (`circle`, `rect`, `path`, `text`,
//! `connection`, `pulse`); this file holds the shared vocabulary that ties them
//! into a Scene: [`AnchorKind`], the [`Node`] trait, [`SceneNode`], and the
//! [`Scene`] root.

mod anchor;
mod circle;
mod connection;
mod error;
mod node;
mod path;
mod primitive;
mod pulse;
mod rect;
mod scene;
mod text;
mod transform;

pub use anchor::*;
pub use circle::{circle, Circle, ConcreteCircle};
pub use connection::{connection, Connection};
pub use error::*;
pub use node::*;
pub use path::{path_node, ConcretePath, PathNode};
pub use primitive::{
    primitive_circle, primitive_path, primitive_rect, primitive_text, ConcreteGeometry,
    ConcretePrimitive, Geometry, Pixels, Primitive, Transformable,
};
pub use pulse::{pulse_on, Pulse};
pub use rect::{rect, ConcreteRect, Rect};
pub use scene::*;
pub use text::{text, ConcreteText, Text, TextAlign};
pub use transform::{ConcreteTransform, Transform};
