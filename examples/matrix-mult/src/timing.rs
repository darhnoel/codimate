#[derive(Clone, Copy)]
pub struct MatrixMultTiming {
    pub intro: f32,
    pub compute_cell: f32,
    pub final_hold: f32,
}

impl Default for MatrixMultTiming {
    fn default() -> Self {
        Self {
            intro: 1.0,
            compute_cell: 1.6,
            final_hold: 1.2,
        }
    }
}
