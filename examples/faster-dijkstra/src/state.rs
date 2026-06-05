pub const NODE_COUNT: usize = 8;
pub const EDGE_COUNT: usize = 10;

#[derive(Clone, Copy)]
pub struct DemoEdge {
    pub(crate) from: usize,
    pub(crate) to: usize,
    pub(crate) weight: u32,
}

#[derive(Clone, Copy)]
pub struct FasterDijkstra {
    pub(crate) labels: [&'static str; NODE_COUNT],
    pub(crate) edges: [DemoEdge; EDGE_COUNT],
}

impl FasterDijkstra {
    pub fn paper_demo() -> Self {
        Self {
            labels: ["s", "a", "b", "c", "d", "e", "f", "t"],
            edges: [
                DemoEdge {
                    from: 0,
                    to: 1,
                    weight: 2,
                },
                DemoEdge {
                    from: 0,
                    to: 2,
                    weight: 5,
                },
                DemoEdge {
                    from: 1,
                    to: 3,
                    weight: 2,
                },
                DemoEdge {
                    from: 1,
                    to: 4,
                    weight: 6,
                },
                DemoEdge {
                    from: 2,
                    to: 4,
                    weight: 1,
                },
                DemoEdge {
                    from: 3,
                    to: 5,
                    weight: 3,
                },
                DemoEdge {
                    from: 4,
                    to: 5,
                    weight: 1,
                },
                DemoEdge {
                    from: 4,
                    to: 6,
                    weight: 2,
                },
                DemoEdge {
                    from: 5,
                    to: 7,
                    weight: 2,
                },
                DemoEdge {
                    from: 6,
                    to: 7,
                    weight: 1,
                },
            ],
        }
    }
}

impl Default for FasterDijkstra {
    fn default() -> Self {
        Self::paper_demo()
    }
}
