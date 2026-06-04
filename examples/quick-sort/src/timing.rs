#[derive(Clone, Copy)]
pub struct QuickSortTiming {
    pub overview: f32,
    pub choose_pivot: f32,
    pub compare: f32,
    pub swap: f32,
    pub place_pivot: f32,
    pub final_hold: f32,
}

impl Default for QuickSortTiming {
    fn default() -> Self {
        Self {
            overview: 1.2,
            choose_pivot: 0.8,
            compare: 0.55,
            swap: 0.85,
            place_pivot: 0.9,
            final_hold: 1.5,
        }
    }
}
