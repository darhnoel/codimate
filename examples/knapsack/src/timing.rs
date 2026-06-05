#[derive(Clone, Copy)]
pub struct KnapsackTiming {
    /// Showing the items and the empty table.
    pub init: f32,
    /// Computing a single cell.
    pub fill: f32,
    /// Holding on the finished table with the solution path.
    pub done: f32,
}

impl Default for KnapsackTiming {
    fn default() -> Self {
        Self {
            init: 1.8,
            fill: 0.42,
            done: 2.4,
        }
    }
}
