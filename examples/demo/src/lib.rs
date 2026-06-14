mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate::Playable;
use codimate::Viewport;

pub use algorithm::{demo_algorithm, DemoStep, DemoTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{demo_motion, DemoMotion};
pub use state::Demo;
pub use timing::DemoTiming;
pub use view::{demo_view, DemoView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Demo")
        .state(Demo::new())
        .view(demo_view)
        .algorithm(demo_algorithm)
        .motion(demo_motion)
        .timing(DemoTiming::default())
        .build()
}
