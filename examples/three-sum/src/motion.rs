use codimate::{ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct ThreeSumMotion;

pub fn three_sum_motion() -> ThreeSumMotion {
    ThreeSumMotion
}

impl ThreeSumMotion {
    pub(crate) fn move_value<T>(
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
