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

/// Layer 3 — named Animations played back-to-back.
///
/// ```
/// use codimate_animation::{animation, sequence};
/// use codimate_core::{circle, rect, scene};
///
/// let intro = animation("intro", 2.0, scene().node(circle().radius(10.0)));
/// let outro = animation("outro", 3.0, scene().node(rect().width(100.0)));
/// let demo = sequence("demo", [intro, outro]);
///
/// assert_eq!(demo.name(), "demo");
/// assert_eq!(demo.duration(), 5.0);
/// assert_eq!(demo.resolve(1.0).children.len(), 1);
/// ```
pub struct Sequence {
    name: String,
    animations: Vec<Animation>,
}

/// Lowercase free constructor matching old Codimate's named `sequence(...)`
/// authoring grammar.
pub fn sequence(
    name: impl Into<String>,
    animations: impl IntoIterator<Item = Animation>,
) -> Sequence {
    let animations = animations.into_iter().collect::<Vec<_>>();
    assert!(
        !animations.is_empty(),
        "A Sequence must contain at least one Animation."
    );
    Sequence {
        name: name.into(),
        animations,
    }
}

impl Sequence {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn duration(&self) -> f32 {
        self.animations.iter().map(Animation::duration).sum::<f32>()
    }

    /// Resolve the active child Animation at sequence-normalized `t`.
    ///
    /// Child boundaries are hard cuts. Each child receives its own local
    /// normalized `t`, so child Animations remain independently authored.
    pub fn resolve(&self, t: f32) -> ConcreteScene {
        let total_duration = self.duration();
        let elapsed = t * total_duration;
        let last_index = self.animations.len() - 1;
        let mut cursor = 0.0;

        for (index, animation) in self.animations.iter().enumerate() {
            let duration = animation.duration();
            let end = cursor + duration;
            if elapsed < end || index == last_index {
                let local_t = if duration == 0.0 {
                    1.0
                } else {
                    (elapsed - cursor) / duration
                };
                return animation.resolve(local_t);
            }
            cursor = end;
        }

        unreachable!("Sequence always contains at least one Animation")
    }
}
