#[derive(Clone, Copy)]
pub struct MergeSortTiming {
    pub overview: f32,
    pub step: f32,
    pub transition: f32,
    pub final_hold: f32,
}

impl Default for MergeSortTiming {
    fn default() -> Self {
        Self {
            overview: 1.4,
            step: 0.72,
            transition: 1.0,
            final_hold: 1.6,
        }
    }
}
