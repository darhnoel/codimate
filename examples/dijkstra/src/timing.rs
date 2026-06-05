#[derive(Clone, Copy)]
pub struct DijkstraTiming {
    /// Showing the graph and the starting distances.
    pub init: f32,
    /// Settling one node and relaxing its neighbours.
    pub settle: f32,
    /// Holding on the finished shortest-path tree.
    pub done: f32,
}

impl Default for DijkstraTiming {
    fn default() -> Self {
        Self {
            init: 1.1,
            settle: 1.2,
            done: 1.8,
        }
    }
}
