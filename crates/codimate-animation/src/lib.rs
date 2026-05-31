//! codimate-animation — Layer 3 (Composition).
//!
//! Duration lives here, not in `codimate-core`.

use codimate_core::{ConcreteScene, Scene};

/// Shared Layer 3 sampling contract for preview/export code.
///
/// A `Playable` has an authored name, a duration in seconds, and resolves from
/// normalized `t` into a concrete Scene.
pub trait Playable {
    fn name(&self) -> &str;
    fn duration(&self) -> f32;
    fn resolve(&self, t: f32) -> ConcreteScene;

    /// Resolve by elapsed seconds, clamped into this Playable's duration.
    ///
    /// This is still pure: callers own clocks/playback loops and pass elapsed
    /// time in explicitly.
    fn resolve_at(&self, elapsed_seconds: f32) -> ConcreteScene {
        let duration = self.duration();
        let t = if duration == 0.0 {
            1.0
        } else {
            (elapsed_seconds / duration).clamp(0.0, 1.0)
        };
        self.resolve(t)
    }
}

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

impl Playable for Animation {
    fn name(&self) -> &str {
        Animation::name(self)
    }

    fn duration(&self) -> f32 {
        Animation::duration(self)
    }

    fn resolve(&self, t: f32) -> ConcreteScene {
        Animation::resolve(self, t)
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

impl Playable for Sequence {
    fn name(&self) -> &str {
        Sequence::name(self)
    }

    fn duration(&self) -> f32 {
        Sequence::duration(self)
    }

    fn resolve(&self, t: f32) -> ConcreteScene {
        Sequence::resolve(self, t)
    }
}

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
