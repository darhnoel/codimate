//! codimate-animation — Layer 3 (Composition).
//!
//! Duration lives here, not in `codimate-core`.

use codimate_core::{ConcreteScene, Scene};

/// Layer 3 — a named Scene paired with a duration.
pub struct Animation {
    name: String,
    duration: f32,
    scene: Scene,
}

/// Lowercase free constructor so composition reads like English.
///
/// ```
/// use codimate_animation::animation;
/// use codimate_core::{circle, scene};
///
/// let a = animation("intro", 2.0, scene().node(circle().radius(10.0)));
/// assert_eq!(a.name(), "intro");
/// assert_eq!(a.duration(), 2.0);
/// assert_eq!(a.resolve(0.5).children.len(), 1);
/// ```
pub fn animation(name: impl Into<String>, duration: f32, scene: Scene) -> Animation {
    Animation {
        name: name.into(),
        duration,
        scene,
    }
}

impl Animation {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Resolve the animation's Scene at normalized `t`.
    ///
    /// Duration is stored for Layer 3 composition; `resolve` remains pure and
    /// does not read clocks or elapsed wall time.
    pub fn resolve(&self, t: f32) -> ConcreteScene {
        self.scene.resolve(t)
    }
}
