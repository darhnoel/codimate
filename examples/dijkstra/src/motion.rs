use codimate_core::{ease_in_out, tween, Animated, IntoAnimated, Lerp};

#[derive(Clone, Copy)]
pub struct DijkstraMotion;

pub fn dijkstra_motion() -> DijkstraMotion {
    DijkstraMotion
}

impl DijkstraMotion {
    /// Smoothly move from `from` to `to` over the step. Used for node fills,
    /// edge highlights, and the radius pulse on the node being settled.
    pub(crate) fn ease<T>(self, from: impl IntoAnimated<T>, to: impl IntoAnimated<T>) -> Animated<T>
    where
        T: Lerp + 'static,
    {
        tween(from, to).ease(ease_in_out)
    }
}
