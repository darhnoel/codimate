/// Number of nodes in the teaching graph (A..E).
pub const NODE_COUNT: usize = 5;
/// Number of weighted, undirected edges.
pub const EDGE_COUNT: usize = 7;
/// Index of the source node we run Dijkstra from (A).
pub const START: usize = 0;

/// The input: a tiny weighted undirected graph and the source node.
///
/// Edges are `(from, to, weight)`. The graph is fixed and small on purpose —
/// just big enough to show a tentative distance being *improved* (B is first
/// reached via A at cost 4, then re-relaxed via C down to 3).
#[derive(Clone, Copy)]
pub struct Dijkstra {
    pub(crate) edges: [(usize, usize, u32); EDGE_COUNT],
    pub(crate) start: usize,
}

impl Dijkstra {
    pub fn new() -> Self {
        Self {
            edges: [
                (0, 1, 4),  // A-B
                (0, 2, 1),  // A-C
                (1, 2, 2),  // B-C
                (1, 3, 5),  // B-D
                (2, 3, 8),  // C-D
                (3, 4, 3),  // D-E
                (1, 4, 10), // B-E
            ],
            start: START,
        }
    }
}

impl Default for Dijkstra {
    fn default() -> Self {
        Self::new()
    }
}
