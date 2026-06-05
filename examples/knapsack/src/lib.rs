mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate_animation::Playable;
use codimate_layout::Viewport;

pub use algorithm::{knapsack_algorithm, KnapsackAction, KnapsackStep, KnapsackTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{knapsack_motion, KnapsackMotion};
pub use state::{Knapsack, CAPACITY, COLS, ITEM_COUNT, ROWS};
pub use timing::KnapsackTiming;
pub use view::{knapsack_view, KnapsackView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Knapsack")
        .state(Knapsack::new())
        .view(knapsack_view)
        .algorithm(knapsack_algorithm)
        .motion(knapsack_motion)
        .timing(KnapsackTiming::default())
        .build()
}
