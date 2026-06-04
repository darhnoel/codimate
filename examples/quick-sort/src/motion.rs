use codimate_core::{ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct QuickSortMotion {
    pub(crate) lift_height: f32,
}

pub fn quick_sort_motion() -> QuickSortMotion {
    QuickSortMotion { lift_height: 72.0 }
}

impl QuickSortMotion {
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
