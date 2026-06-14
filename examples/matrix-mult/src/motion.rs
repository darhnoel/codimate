use codimate::{ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct MatrixMultMotion;

pub fn matrix_mult_motion() -> MatrixMultMotion {
    MatrixMultMotion
}

impl MatrixMultMotion {
    pub(crate) fn reveal<T>(
        self,
        from: impl IntoAnimated<T>,
        to: impl IntoAnimated<T>,
    ) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(ease_in_out)
    }
}
