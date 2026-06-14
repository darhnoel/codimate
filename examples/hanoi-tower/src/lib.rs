mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate::Playable;
use codimate::Viewport;

pub use algorithm::{hanoi_algorithm, HanoiMove, HanoiTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{hanoi_motion, HanoiMotion};
pub use state::{HanoiTower, Peg, DISK_COUNT, PEG_COUNT};
pub use timing::HanoiTiming;
pub use view::{hanoi_view, HanoiView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Tower of Hanoi")
        .state(HanoiTower::new(DISK_COUNT))
        .view(hanoi_view)
        .algorithm(hanoi_algorithm)
        .motion(hanoi_motion)
        .timing(HanoiTiming::default())
        .build()
}
