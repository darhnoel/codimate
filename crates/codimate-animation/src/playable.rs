use codimate_core::ConcreteScene;

/// Shared Layer 3 sampling contract for preview/export code.
///
/// A `Playable` has an authored name, a duration in seconds, and resolves from
/// normalized `t` into a concrete Scene.
pub trait Playable {
    /// The name this playable was given.
    fn name(&self) -> &str;
    /// How long it lasts, in seconds.
    fn duration(&self) -> f32;
    /// The scene at normalized time `t`, from 0 at the start to 1 at the end.
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

impl Playable for Box<dyn Playable> {
    fn name(&self) -> &str {
        (**self).name()
    }
    fn duration(&self) -> f32 {
        (**self).duration()
    }
    fn resolve(&self, t: f32) -> ConcreteScene {
        (**self).resolve(t)
    }
}
