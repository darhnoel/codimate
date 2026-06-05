mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate_animation::Playable;
use codimate_layout::Viewport;

pub use algorithm::{
    faster_dijkstra_algorithm, FasterDijkstraAction, FasterDijkstraStep, FasterDijkstraTrace,
};
pub use builder::{explain, ExplainBuilder};
pub use motion::{faster_dijkstra_motion, FasterDijkstraMotion};
pub use state::{FasterDijkstra, EDGE_COUNT, NODE_COUNT};
pub use timing::FasterDijkstraTiming;
pub use view::{faster_dijkstra_view, FasterDijkstraView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Faster Dijkstra")
        .state(FasterDijkstra::paper_demo())
        .view(faster_dijkstra_view)
        .algorithm(faster_dijkstra_algorithm)
        .motion(faster_dijkstra_motion)
        .timing(FasterDijkstraTiming::default())
        .build()
}
