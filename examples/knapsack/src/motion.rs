use codimate::{ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct KnapsackMotion;

pub fn knapsack_motion() -> KnapsackMotion {
    KnapsackMotion
}

impl KnapsackMotion {
    /// Smoothly move from `from` to `to` over the step. Used to fade the active
    /// cell's highlight in as it is computed.
    pub(crate) fn ease<T>(self, from: impl IntoAnimated<T>, to: impl IntoAnimated<T>) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(ease_in_out)
    }
}
