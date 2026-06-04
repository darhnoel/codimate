mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate_animation::Playable;
use codimate_layout::Viewport;

pub use algorithm::{swap_algorithm, SwapMove};
pub use builder::{explain, ExplainBuilder};
pub use motion::{swap_motion, SwapMotion};
pub use state::{BallKey, Swap};
pub use timing::SwapTiming;
pub use view::{swap_view, SwapView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Swap")
        .state(Swap::new("A", "B"))
        .view(swap_view)
        .algorithm(swap_algorithm)
        .motion(swap_motion)
        .timing(SwapTiming::default())
        .build()
}
