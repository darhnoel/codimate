use crate::{Dijkstra, EDGE_COUNT, NODE_COUNT};

/// What a single step of the trace is doing.
#[derive(Clone, Copy)]
pub enum DijkstraAction {
    /// Show the graph; the source has distance 0, everyone else ∞.
    Init,
    /// Pop the closest unsettled node and relax its neighbours.
    Settle { node: usize },
    /// Every node settled — the shortest-path tree is complete.
    Done,
}

/// A full snapshot of the algorithm after one step.
///
/// The view renders the whole graph from this snapshot, so each field describes
/// the state *as it stands now*, not an editing command.
#[derive(Clone, Copy)]
pub struct DijkstraStep {
    pub(crate) index: usize,
    pub(crate) action: DijkstraAction,
    /// Tentative shortest distance from the source; `None` means ∞.
    pub(crate) dist: [Option<u32>; NODE_COUNT],
    /// Whether a node has been settled (its distance is final).
    pub(crate) visited: [bool; NODE_COUNT],
    /// Predecessor on the current best path — the shortest-path tree.
    pub(crate) pred: [Option<usize>; NODE_COUNT],
    /// The node being settled this step, if any.
    pub(crate) current: Option<usize>,
    /// Nodes whose distance was lowered this step (for the relax highlight).
    pub(crate) improved: [bool; NODE_COUNT],
}

pub struct DijkstraTrace {
    pub(crate) steps: Vec<DijkstraStep>,
    pub(crate) edges: [(usize, usize, u32); EDGE_COUNT],
}

pub fn dijkstra_algorithm(state: Dijkstra) -> DijkstraTrace {
    let edges = state.edges;
    let start = state.start;

    let mut dist: [Option<u32>; NODE_COUNT] = [None; NODE_COUNT];
    let mut visited = [false; NODE_COUNT];
    let mut pred: [Option<usize>; NODE_COUNT] = [None; NODE_COUNT];
    dist[start] = Some(0);

    let mut steps = Vec::new();
    let mut index = 0;

    steps.push(DijkstraStep {
        index,
        action: DijkstraAction::Init,
        dist,
        visited,
        pred,
        current: None,
        improved: [false; NODE_COUNT],
    });
    index += 1;

    loop {
        // Extract the unsettled node with the smallest tentative distance.
        let mut best: Option<(usize, u32)> = None;
        for n in 0..NODE_COUNT {
            if visited[n] {
                continue;
            }
            if let Some(d) = dist[n] {
                if best.map_or(true, |(_, bd)| d < bd) {
                    best = Some((n, d));
                }
            }
        }
        let (u, du) = match best {
            Some(found) => found,
            None => break,
        };

        visited[u] = true;
        let mut improved = [false; NODE_COUNT];

        // Relax every edge incident to u that leads to an unsettled neighbour.
        for &(a, b, w) in &edges {
            let v = if a == u {
                Some(b)
            } else if b == u {
                Some(a)
            } else {
                None
            };
            if let Some(v) = v {
                if !visited[v] {
                    let candidate = du + w;
                    if dist[v].map_or(true, |d| candidate < d) {
                        dist[v] = Some(candidate);
                        pred[v] = Some(u);
                        improved[v] = true;
                    }
                }
            }
        }

        steps.push(DijkstraStep {
            index,
            action: DijkstraAction::Settle { node: u },
            dist,
            visited,
            pred,
            current: Some(u),
            improved,
        });
        index += 1;
    }

    steps.push(DijkstraStep {
        index,
        action: DijkstraAction::Done,
        dist,
        visited,
        pred,
        current: None,
        improved: [false; NODE_COUNT],
    });

    DijkstraTrace { steps, edges }
}
