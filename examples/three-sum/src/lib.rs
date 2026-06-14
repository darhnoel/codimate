mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate::Playable;
use codimate::Viewport;

pub use algorithm::{three_sum_algorithm, ThreeSumAction, ThreeSumStep, ThreeSumTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{three_sum_motion, ThreeSumMotion};
pub use state::{ThreeSum, DEFAULT_VALUES, N};
pub use timing::ThreeSumTiming;
pub use view::{three_sum_view, ThreeSumView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("3Sum")
        .state(ThreeSum::new(DEFAULT_VALUES))
        .view(three_sum_view)
        .algorithm(three_sum_algorithm)
        .motion(three_sum_motion)
        .timing(ThreeSumTiming::default())
        .build()
}
