use crate::{Animation, Playable};
use codimate_core::ConcreteScene;

/// Layer 3 — named Animations played at the same time.
///
/// ```
/// use codimate_animation::{animation, parallel};
/// use codimate_core::{circle, rect, scene};
///
/// let pulse = animation("pulse", 2.0, scene().node(circle().radius(10.0)));
/// let bar = animation("bar", 4.0, scene().node(rect().width(100.0)));
/// let demo = parallel("demo", [pulse, bar]);
///
/// assert_eq!(demo.name(), "demo");
/// assert_eq!(demo.duration(), 4.0);
/// assert_eq!(demo.resolve(0.5).children.len(), 2);
/// ```
pub struct Parallel {
    name: String,
    animations: Vec<Animation>,
}

/// Lowercase free constructor for named parallel composition.
pub fn parallel(
    name: impl Into<String>,
    animations: impl IntoIterator<Item = Animation>,
) -> Parallel {
    let animations = animations.into_iter().collect::<Vec<_>>();
    assert!(
        !animations.is_empty(),
        "A Parallel must contain at least one Animation."
    );
    Parallel {
        name: name.into(),
        animations,
    }
}

impl Parallel {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn duration(&self) -> f32 {
        self.animations
            .iter()
            .map(Animation::duration)
            .fold(0.0, f32::max)
    }

    /// Resolve every child Animation at the same elapsed time.
    ///
    /// Shorter children hold their final state once their local normalized time
    /// reaches `1.0`.
    #[must_use]
    pub fn resolve(&self, t: f32) -> ConcreteScene {
        let total_duration = self.duration();
        let elapsed = t * total_duration;
        let children = self
            .animations
            .iter()
            .flat_map(|animation| {
                let duration = animation.duration();
                let local_t = if duration == 0.0 {
                    1.0
                } else {
                    (elapsed / duration).min(1.0)
                };
                animation.resolve(local_t).children
            })
            .collect();

        ConcreteScene { children }
    }
}

impl Playable for Parallel {
    fn name(&self) -> &str {
        Parallel::name(self)
    }

    fn duration(&self) -> f32 {
        Parallel::duration(self)
    }

    fn resolve(&self, t: f32) -> ConcreteScene {
        Parallel::resolve(self, t)
    }
}
