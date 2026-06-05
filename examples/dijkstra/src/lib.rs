mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate_animation::Playable;
use codimate_layout::Viewport;

pub use algorithm::{dijkstra_algorithm, DijkstraAction, DijkstraStep, DijkstraTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{dijkstra_motion, DijkstraMotion};
pub use state::{Dijkstra, EDGE_COUNT, NODE_COUNT, START};
pub use timing::DijkstraTiming;
pub use view::{dijkstra_view, DijkstraView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Dijkstra")
        .state(Dijkstra::new())
        .view(dijkstra_view)
        .algorithm(dijkstra_algorithm)
        .motion(dijkstra_motion)
        .timing(DijkstraTiming::default())
        .build()
}
