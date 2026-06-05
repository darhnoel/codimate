//! Layer 1 — Generic explanation builder.
//!
//! Eliminates 16 copies of `Option`-field + `expect()` boilerplate in examples.
//! Each example wraps this as a thin newtype that adds domain-specific
//! [`build`] and [`render`] methods.

/// A generic builder for the standard explanation pattern:
/// `(name, state, algorithm, motion, timing) → Playable`.
///
/// All fields are optional setters except `name`. Call [`take`](Self::take)
/// at the end to retrieve the values (as a `Result`), or use this inside a
/// newtype wrapper that provides a typed [`build`] method.
///
/// The type parameter `A` is either a function pointer
/// `fn(State) -> Trace` (deferred — call in `build`) or the trace type
/// directly (eager — pre-computed in the setter).
///
/// # Example
///
/// ```ignore
/// pub type MyBuilder = ExplanationBuilder<MyState, fn(MyState) -> MyTrace, MyMotion, MyTiming>;
/// ```
pub struct ExplanationBuilder<S, A, M, T = ()> {
    pub name: &'static str,
    pub state: Option<S>,
    pub algorithm: Option<A>,
    pub motion: Option<M>,
    pub timing: T,
}

impl<S, A, M, T: Default> ExplanationBuilder<S, A, M, T> {
    /// All optional fields start as `None`; timing uses `Default`.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            state: None,
            algorithm: None,
            motion: None,
            timing: T::default(),
        }
    }

    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    pub fn algorithm(mut self, algorithm: A) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    pub fn motion(mut self, motion: M) -> Self {
        self.motion = Some(motion);
        self
    }

    pub fn timing(mut self, timing: T) -> Self {
        self.timing = timing;
        self
    }

    /// No-op — kept for backward compatibility.
    /// View code is typically called from other build helpers.
    pub fn view<V>(self, _view: fn() -> V) -> Self {
        self
    }

    /// Consume the builder and return all four values, or the first missing
    /// field name. This is the only place `Option` unwrapping happens.
    pub fn take(self) -> Result<(S, A, M, T), &'static str> {
        Ok((
            self.state.ok_or("state")?,
            self.algorithm.ok_or("algorithm")?,
            self.motion.ok_or("motion")?,
            self.timing,
        ))
    }
}
