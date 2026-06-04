use codimate_core::{ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct MergeSortMotion;

pub fn merge_sort_motion() -> MergeSortMotion {
    MergeSortMotion
}

impl MergeSortMotion {
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
