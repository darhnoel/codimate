#[derive(Clone, Copy)]
pub struct FasterDijkstraTiming {
    pub(crate) intro: f32,
    pub(crate) concept: f32,
    pub(crate) result: f32,
}

impl Default for FasterDijkstraTiming {
    fn default() -> Self {
        Self {
            intro: 2.2,
            concept: 3.0,
            result: 3.2,
        }
    }
}
