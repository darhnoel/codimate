use super::{
    Circle, ConcreteCircle, ConcretePath, ConcretePrimitive, ConcreteRect, ConcreteText,
    Connection, PathNode, Primitive, Pulse, Rect, Text,
};
use crate::value::Animated;
use crate::{SceneTransformError, TimeCurve};

/// Shared Layer 2 interface: pure Node data resolves into concrete data at `t`.
pub trait Node {
    /// The plain data this node becomes at a given time.
    type Concrete;

    /// The node's plain data at time `t`.
    fn resolve(&self, t: f32) -> Self::Concrete;
}

/// A supported Node inside a Scene.
#[derive(Clone)]
pub enum SceneNode {
    Primitive(Primitive),
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
    Primitive(ConcretePrimitive),
    Circle(ConcreteCircle),
    Rect(ConcreteRect),
    Path(ConcretePath),
    Text(ConcreteText),
}

impl From<Primitive> for SceneNode {
    fn from(primitive: Primitive) -> Self {
        SceneNode::Primitive(primitive)
    }
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
            SceneNode::Primitive(primitive) => ConcreteNode::Primitive(primitive.resolve(t)),
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
            SceneNode::Primitive(_) => "Primitive",
            SceneNode::Circle(_) => "Circle",
            SceneNode::Rect(_) => "Rect",
            SceneNode::Path(_) => "Path",
            SceneNode::Text(_) => "Text",
            SceneNode::Connection(_) => "Connection",
            SceneNode::Pulse(_) => "Pulse",
        }
    }

    pub(crate) fn with_opacity(self, multiplier: Animated<f32>) -> Self {
        match self {
            SceneNode::Primitive(primitive) => {
                SceneNode::Primitive(primitive.with_opacity(multiplier))
            }
            SceneNode::Circle(circle) => SceneNode::Circle(circle.with_opacity(multiplier)),
            SceneNode::Rect(rect) => SceneNode::Rect(rect.with_opacity(multiplier)),
            SceneNode::Path(path) => SceneNode::Path(path.with_opacity(multiplier)),
            SceneNode::Text(text) => SceneNode::Text(text.with_opacity(multiplier)),
            SceneNode::Connection(conn) => SceneNode::Connection(conn.with_opacity(multiplier)),
            SceneNode::Pulse(pulse) => SceneNode::Pulse(pulse.with_opacity(multiplier)),
        }
    }

    pub(crate) fn ease(self, curve: TimeCurve) -> Self {
        match self {
            SceneNode::Primitive(primitive) => SceneNode::Primitive(primitive.ease(curve)),
            SceneNode::Circle(circle) => SceneNode::Circle(circle.ease(curve)),
            SceneNode::Rect(rect) => SceneNode::Rect(rect.ease(curve)),
            SceneNode::Path(path) => SceneNode::Path(path.ease(curve)),
            SceneNode::Text(text) => SceneNode::Text(text.ease(curve)),
            SceneNode::Connection(conn) => SceneNode::Connection(conn.ease(curve)),
            SceneNode::Pulse(pulse) => SceneNode::Pulse(pulse.ease(curve)),
        }
    }

    pub(crate) fn reveal(self, progress: Animated<f32>) -> Self {
        match self {
            SceneNode::Primitive(primitive) => SceneNode::Primitive(primitive.reveal(progress)),
            SceneNode::Circle(circle) => SceneNode::Circle(circle.with_opacity(progress)),
            SceneNode::Rect(rect) => SceneNode::Rect(rect.with_opacity(progress)),
            SceneNode::Path(path) => SceneNode::Path(path.reveal(progress)),
            SceneNode::Text(text) => SceneNode::Text(text.with_opacity(progress)),
            SceneNode::Connection(conn) => SceneNode::Connection(conn.reveal(progress)),
            SceneNode::Pulse(pulse) => SceneNode::Pulse(pulse.reveal(progress)),
        }
    }

    pub(crate) fn try_lerp_to(&self, to: &Self, index: usize) -> Result<Self, SceneTransformError> {
        match (self, to) {
            (SceneNode::Primitive(from), SceneNode::Primitive(to)) => {
                Ok(SceneNode::Primitive(from.try_lerp_to(to)?))
            }
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
