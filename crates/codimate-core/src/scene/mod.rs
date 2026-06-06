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

mod circle;
mod connection;
mod path;
mod pulse;
mod rect;
mod text;

use std::sync::Arc;

use crate::value::{Animated, IntoAnimated};

pub use circle::{circle, Circle, ConcreteCircle};
pub use connection::{connection, Connection};
pub use path::{path_node, ConcretePath, PathNode};
pub use pulse::{pulse_on, Pulse};
pub use rect::{rect, ConcreteRect, Rect};
pub use text::{text, ConcreteText, Text, TextAlign};

pub(crate) type TimeCurve = Arc<dyn Fn(f32) -> f32>;

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

/// Shared Layer 2 interface: pure Node data resolves into concrete data at `t`.
pub trait Node {
    type Concrete;

    fn resolve(&self, t: f32) -> Self::Concrete;
}

/// A supported Node inside a Scene.
#[derive(Clone)]
pub enum SceneNode {
    Circle(Circle),
    Rect(Rect),
    Path(PathNode),
    Text(Text),
    Connection(Connection),
    Pulse(Pulse),
}

/// A resolved Scene child — concrete data only, no rendering backend.
#[derive(Clone, Debug, PartialEq)]
pub enum ConcreteNode {
    Circle(ConcreteCircle),
    Rect(ConcreteRect),
    Path(ConcretePath),
    Text(ConcreteText),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneTransformError {
    ChildCountMismatch {
        from: usize,
        to: usize,
    },
    NodeKindMismatch {
        index: usize,
        from: &'static str,
        to: &'static str,
    },
    WaypointCountMismatch {
        from: usize,
        to: usize,
    },
}

impl From<Circle> for SceneNode {
    fn from(circle: Circle) -> Self {
        SceneNode::Circle(circle)
    }
}

impl From<Rect> for SceneNode {
    fn from(rect: Rect) -> Self {
        SceneNode::Rect(rect)
    }
}

impl From<PathNode> for SceneNode {
    fn from(path: PathNode) -> Self {
        SceneNode::Path(path)
    }
}

impl From<Text> for SceneNode {
    fn from(text: Text) -> Self {
        SceneNode::Text(text)
    }
}

impl From<Connection> for SceneNode {
    fn from(conn: Connection) -> Self {
        SceneNode::Connection(conn)
    }
}

impl From<Pulse> for SceneNode {
    fn from(pulse: Pulse) -> Self {
        SceneNode::Pulse(pulse)
    }
}

impl Node for SceneNode {
    type Concrete = ConcreteNode;

    fn resolve(&self, t: f32) -> Self::Concrete {
        match self {
            SceneNode::Circle(circle) => ConcreteNode::Circle(circle.resolve(t)),
            SceneNode::Rect(rect) => ConcreteNode::Rect(rect.resolve(t)),
            SceneNode::Path(path) => ConcreteNode::Path(path.resolve(t)),
            SceneNode::Text(text) => ConcreteNode::Text(text.resolve(t)),
            SceneNode::Connection(conn) => ConcreteNode::Path(conn.resolve(t)),
            SceneNode::Pulse(pulse) => ConcreteNode::Circle(pulse.resolve(t)),
        }
    }
}

impl SceneNode {
    fn kind(&self) -> &'static str {
        match self {
            SceneNode::Circle(_) => "Circle",
            SceneNode::Rect(_) => "Rect",
            SceneNode::Path(_) => "Path",
            SceneNode::Text(_) => "Text",
            SceneNode::Connection(_) => "Connection",
            SceneNode::Pulse(_) => "Pulse",
        }
    }

    fn with_opacity(self, multiplier: Animated<f32>) -> Self {
        match self {
            SceneNode::Circle(circle) => SceneNode::Circle(circle.with_opacity(multiplier)),
            SceneNode::Rect(rect) => SceneNode::Rect(rect.with_opacity(multiplier)),
            SceneNode::Path(path) => SceneNode::Path(path.with_opacity(multiplier)),
            SceneNode::Text(text) => SceneNode::Text(text.with_opacity(multiplier)),
            SceneNode::Connection(conn) => SceneNode::Connection(conn.with_opacity(multiplier)),
            SceneNode::Pulse(pulse) => SceneNode::Pulse(pulse.with_opacity(multiplier)),
        }
    }

    fn ease(self, curve: TimeCurve) -> Self {
        match self {
            SceneNode::Circle(circle) => SceneNode::Circle(circle.ease(curve)),
            SceneNode::Rect(rect) => SceneNode::Rect(rect.ease(curve)),
            SceneNode::Path(path) => SceneNode::Path(path.ease(curve)),
            SceneNode::Text(text) => SceneNode::Text(text.ease(curve)),
            SceneNode::Connection(conn) => SceneNode::Connection(conn.ease(curve)),
            SceneNode::Pulse(pulse) => SceneNode::Pulse(pulse.ease(curve)),
        }
    }

    fn reveal(self, progress: Animated<f32>) -> Self {
        match self {
            SceneNode::Circle(circle) => SceneNode::Circle(circle.with_opacity(progress)),
            SceneNode::Rect(rect) => SceneNode::Rect(rect.with_opacity(progress)),
            SceneNode::Path(path) => SceneNode::Path(path.reveal(progress)),
            SceneNode::Text(text) => SceneNode::Text(text.with_opacity(progress)),
            SceneNode::Connection(conn) => SceneNode::Connection(conn.reveal(progress)),
            SceneNode::Pulse(pulse) => SceneNode::Pulse(pulse.reveal(progress)),
        }
    }

    fn try_lerp_to(&self, to: &Self, index: usize) -> Result<Self, SceneTransformError> {
        match (self, to) {
            (SceneNode::Circle(from), SceneNode::Circle(to)) => {
                Ok(SceneNode::Circle(from.try_lerp_to(to)))
            }
            (SceneNode::Rect(from), SceneNode::Rect(to)) => {
                Ok(SceneNode::Rect(from.try_lerp_to(to)))
            }
            (SceneNode::Path(from), SceneNode::Path(to)) => {
                Ok(SceneNode::Path(from.try_lerp_to(to)))
            }
            (SceneNode::Text(from), SceneNode::Text(to)) => {
                Ok(SceneNode::Text(from.try_lerp_to(to)))
            }
            (SceneNode::Connection(from), SceneNode::Connection(to)) => {
                Ok(SceneNode::Connection(from.try_lerp_to(to)?))
            }
            (SceneNode::Pulse(from), SceneNode::Pulse(to)) => {
                Ok(SceneNode::Pulse(from.try_lerp_to(to)?))
            }
            (from, to) => Err(SceneTransformError::NodeKindMismatch {
                index,
                from: from.kind(),
                to: to.kind(),
            }),
        }
    }
}

/// A Scene (Layer 2): a tree of Nodes resolved by the same normalized `t`.
///
/// ```
/// use codimate_core::{circle, rect, scene, Color};
///
/// let s = scene()
///     .node(circle().x(10.0).radius(5.0).fill(Color::RED))
///     .node(rect().width(100.0).height(40.0).fill(Color::RED));
///
/// let concrete = s.resolve(0.5);
/// assert_eq!(concrete.children.len(), 2);
/// ```
#[derive(Clone)]
pub struct Scene {
    children: Vec<SceneNode>,
}

/// A Scene resolved at a specific `t` — all children are concrete data.
#[derive(Clone, Debug, PartialEq)]
pub struct ConcreteScene {
    pub children: Vec<ConcreteNode>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            children: Vec::new(),
        }
    }

    pub fn node(mut self, node: impl Into<SceneNode>) -> Self {
        self.children.push(node.into());
        self
    }

    pub fn with_opacity(self, multiplier: impl IntoAnimated<f32>) -> Self {
        let multiplier = multiplier.into_animated();
        Scene {
            children: self
                .children
                .into_iter()
                .map(|node| node.with_opacity(multiplier.clone()))
                .collect(),
        }
    }

    pub fn ease(self, curve: impl Fn(f32) -> f32 + 'static) -> Self {
        let curve = Arc::new(curve);
        Scene {
            children: self
                .children
                .into_iter()
                .map(|node| node.ease(curve.clone()))
                .collect(),
        }
    }

    pub fn reveal(self, progress: impl IntoAnimated<f32>) -> Self {
        let progress = progress.into_animated();
        Scene {
            children: self
                .children
                .into_iter()
                .map(|node| node.reveal(progress.clone()))
                .collect(),
        }
    }

    pub fn try_lerp_to(&self, to: &Self) -> Result<Self, SceneTransformError> {
        if self.children.len() != to.children.len() {
            return Err(SceneTransformError::ChildCountMismatch {
                from: self.children.len(),
                to: to.children.len(),
            });
        }

        let children = self
            .children
            .iter()
            .zip(&to.children)
            .enumerate()
            .map(|(index, (from, to))| from.try_lerp_to(to, index))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Scene { children })
    }

    /// `f(t) → ConcreteScene` — resolves every child Node at the same `t`.
    pub fn resolve(&self, t: f32) -> ConcreteScene {
        ConcreteScene {
            children: self.children.iter().map(|node| node.resolve(t)).collect(),
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for Scene {
    type Concrete = ConcreteScene;

    fn resolve(&self, t: f32) -> Self::Concrete {
        Scene::resolve(self, t)
    }
}

/// Lowercase free constructor so scene roots read like English.
pub fn scene() -> Scene {
    Scene::new()
}
