use crate::value::IntoAnimated;
use crate::{ConcreteNode, Node, SceneNode, SceneTransformError};
use std::sync::Arc;

/// A remapping of a segment's time `t` (0..1), applied to everything a shape
/// does at once: its timing within the beat, and its easing.
pub type TimeCurve = Arc<dyn Fn(f32) -> f32 + Send + Sync>;

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
    #[must_use]
    pub fn new() -> Self {
        Scene {
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn node(mut self, node: impl Into<SceneNode>) -> Self {
        self.children.push(node.into());
        self
    }

    /// Alias for [`node`](Self::node), for builder chains that read `scene().add(...)`.
    #[must_use]
    pub fn add(self, node: impl Into<SceneNode>) -> Self {
        self.node(node)
    }

    #[must_use]
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

    #[must_use]
    pub fn ease(self, curve: impl Fn(f32) -> f32 + Send + Sync + 'static) -> Self {
        let curve = Arc::new(curve);
        Scene {
            children: self
                .children
                .into_iter()
                .map(|node| node.ease(curve.clone()))
                .collect(),
        }
    }

    #[must_use]
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
    #[must_use]
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
#[must_use]
pub fn scene() -> Scene {
    Scene::new()
}
