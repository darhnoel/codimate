use codimate::{ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct DemoMotion;

pub fn demo_motion() -> DemoMotion {
    DemoMotion
}

impl DemoMotion {
    pub(crate) fn travel<T>(
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
