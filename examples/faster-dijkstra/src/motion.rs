use codimate_core::{ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct FasterDijkstraMotion;

pub fn faster_dijkstra_motion() -> FasterDijkstraMotion {
    FasterDijkstraMotion
}

impl FasterDijkstraMotion {
    pub(crate) fn ease<T>(self, from: impl IntoAnimated<T>, to: impl IntoAnimated<T>) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(|t| ease_in_out((t / 0.40).clamp(0.0, 1.0)))
    }
}
