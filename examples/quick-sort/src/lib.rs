mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate_animation::Playable;
use codimate_layout::Viewport;

pub use algorithm::{quick_sort_algorithm, QuickAction, QuickStep, QuickTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{quick_sort_motion, QuickSortMotion};
pub use state::{QuickSort, DEFAULT_VALUES, N};
pub use timing::QuickSortTiming;
pub use view::{quick_sort_view, QuickSortView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Quick Sort")
        .state(QuickSort::new(DEFAULT_VALUES))
        .view(quick_sort_view)
        .algorithm(quick_sort_algorithm)
        .motion(quick_sort_motion)
        .timing(QuickSortTiming::default())
        .build()
}
