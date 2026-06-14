use codimate::{ease_in, ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct SymSpellMotion;

pub fn symspell_motion() -> SymSpellMotion {
    SymSpellMotion
}

impl SymSpellMotion {
    pub fn delete_letter(&self) -> Animated<f32> {
        tween(1.0, 0.0).ease(ease_in)
    }

    pub fn slide_together(&self) -> Animated<f32> {
        tween(0.0, 1.0).ease(ease_in_out)
    }

    pub fn travel<T>(self, from: impl IntoAnimated<T>, to: impl IntoAnimated<T>) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(ease_in_out)
    }

    pub fn pulse_progress(&self) -> Animated<f32> {
        tween(0.0, 1.0)
    }

    pub fn hit_pulse_progress(&self) -> Animated<f32> {
        Animated::new(|t| (t / 0.45).clamp(0.0, 1.0))
    }

    pub fn highlight<T>(self, from: impl IntoAnimated<T>, to: impl IntoAnimated<T>) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(ease_in_out)
    }

    pub fn fade_in<T>(self, from: impl IntoAnimated<T>, to: impl IntoAnimated<T>) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(ease_in_out)
    }
}
