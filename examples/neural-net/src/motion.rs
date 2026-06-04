use codimate_core::{ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct NeuralNetMotion;

pub fn neural_net_motion() -> NeuralNetMotion {
    NeuralNetMotion
}

impl NeuralNetMotion {
    pub(crate) fn fire<T>(self, from: impl IntoAnimated<T>, to: impl IntoAnimated<T>) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(ease_in_out)
    }
}
