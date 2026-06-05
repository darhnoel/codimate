use codimate_core::{ease_in_out, tween, Animated, IntoAnimated, Lerp, Vec2};

#[derive(Clone, Copy)]
pub struct NewtonLawsMotion;

pub fn newton_laws_motion() -> NewtonLawsMotion {
    NewtonLawsMotion
}

impl NewtonLawsMotion {
    pub(crate) fn linear<T>(
        self,
        from: impl IntoAnimated<T>,
        to: impl IntoAnimated<T>,
    ) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to)
    }

    pub(crate) fn ease<T>(self, from: impl IntoAnimated<T>, to: impl IntoAnimated<T>) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(ease_in_out)
    }

    pub(crate) fn accelerate(self, from: Vec2, to: Vec2) -> Animated<Vec2> {
        tween(from, to).ease(|t| t * t)
    }

    pub(crate) fn pulse(self, low: f32, high: f32) -> Animated<f32> {
        Animated::new(move |t| {
            let wave = (std::f32::consts::TAU * t).sin().abs();
            low + (high - low) * wave
        })
    }
}
