use codimate_core::{ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct TransformerArchitectureMotion;

pub fn transformer_architecture_motion() -> TransformerArchitectureMotion {
    TransformerArchitectureMotion
}

impl TransformerArchitectureMotion {
    pub(crate) fn ease<T>(self, from: impl IntoAnimated<T>, to: impl IntoAnimated<T>) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(ease_in_out)
    }

    pub(crate) fn pulse(self, low: f32, high: f32) -> Animated<f32> {
        Animated::new(move |t| {
            let wave = (std::f32::consts::TAU * t).sin().abs();
            low + (high - low) * wave
        })
    }
}
