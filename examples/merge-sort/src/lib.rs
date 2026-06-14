mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate::Playable;
use codimate::Viewport;

pub use algorithm::{merge_sort_algorithm, MergeStep, MergeTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{merge_sort_motion, MergeSortMotion};
pub use state::{MergeSort, DEFAULT_VALUES, N};
pub use timing::MergeSortTiming;
pub use view::{merge_sort_view, MergeSortView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Merge Sort")
        .state(MergeSort::new(DEFAULT_VALUES))
        .view(merge_sort_view)
        .algorithm(merge_sort_algorithm)
        .motion(merge_sort_motion)
        .timing(MergeSortTiming::default())
        .build()
}
