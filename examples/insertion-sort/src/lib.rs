mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate::Playable;
use codimate::Viewport;

pub use algorithm::{
    insertion_sort_algorithm, HeldKey, InsertionAction, InsertionMovement, InsertionStep,
    InsertionTrace, VisualItem,
};
pub use builder::{explain, ExplainBuilder};
pub use motion::{insertion_sort_motion, InsertionSortMotion};
pub use state::{InsertionSort, DEFAULT_VALUES, N};
pub use timing::InsertionSortTiming;
pub use view::{insertion_sort_view, InsertionSortView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("ការតម្រៀបដោយបញ្ចូល")
        .state(InsertionSort::new(DEFAULT_VALUES))
        .view(insertion_sort_view)
        .algorithm(insertion_sort_algorithm)
        .motion(insertion_sort_motion)
        .timing(InsertionSortTiming::default())
        .build()
}
