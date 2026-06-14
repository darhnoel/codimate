mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate::Playable;
use codimate::Viewport;

pub use algorithm::{swap_a_b_algorithm, SwapABEvent, SwapABTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{swap_a_b_motion, SwapABMotion};
pub use state::{ItemId, SlotId, SwapAB};
pub use timing::SwapABTiming;
pub use view::{swap_a_b_view, SwapABView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Swap A B")
        .state(SwapAB::new())
        .view(swap_a_b_view)
        .algorithm(swap_a_b_algorithm)
        .motion(swap_a_b_motion)
        .timing(SwapABTiming::default())
        .build()
}
