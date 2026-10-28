use crate::{Animation, Playable};
use codimate_core::ConcreteScene;

/// Layer 3 — named Animations started at fixed offsets.
///
/// ```
/// use codimate_animation::{animation, stagger};
/// use codimate_core::{circle, rect, scene};
///
/// let first = animation("first", 2.0, scene().node(circle().radius(10.0)));
/// let second = animation("second", 2.0, scene().node(rect().width(100.0)));
/// let demo = stagger("demo", 1.0, [first, second]);
///
/// assert_eq!(demo.name(), "demo");
/// assert_eq!(demo.duration(), 3.0);
/// assert_eq!(demo.resolve(0.0).children.len(), 1);
/// ```
pub struct Stagger {
    name: String,
    offset: f32,
    animations: Vec<Animation>,
}

/// Lowercase free constructor for named staggered composition.
pub fn stagger(
    name: impl Into<String>,
    offset: f32,
    animations: impl IntoIterator<Item = Animation>,
) -> Stagger {
    let animations = animations.into_iter().collect::<Vec<_>>();
    assert!(
        !animations.is_empty(),
        "A Stagger must contain at least one Animation."
    );
    Stagger {
        name: name.into(),
        offset,
        animations,
    }
}

impl Stagger {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn duration(&self) -> f32 {
        self.animations
            .iter()
            .enumerate()
            .map(|(index, animation)| self.offset * index as f32 + animation.duration())
            .fold(0.0, f32::max)
    }

    /// Resolve every started child Animation at stagger-normalized `t`.
    ///
    /// Not-yet-started children are absent. Finished children hold their final
    /// state once their local normalized time reaches `1.0`.
    #[must_use]
    pub fn resolve(&self, t: f32) -> ConcreteScene {
        let total_duration = self.duration();
        let elapsed = t * total_duration;
        let children = self
            .animations
            .iter()
            .enumerate()
            .filter_map(|(index, animation)| {
                let start = self.offset * index as f32;
                if elapsed < start {
                    return None;
                }

                let duration = animation.duration();
                let local_t = if duration == 0.0 {
                    1.0
                } else {
                    ((elapsed - start) / duration).min(1.0)
                };
                Some(animation.resolve(local_t).children)
            })
            .flatten()
            .collect();

        ConcreteScene { children }
    }
}

impl Playable for Stagger {
    fn name(&self) -> &str {
        Stagger::name(self)
    }

    fn duration(&self) -> f32 {
        Stagger::duration(self)
    }

    fn resolve(&self, t: f32) -> ConcreteScene {
        Stagger::resolve(self, t)
    }
}
