mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate::Playable;
use codimate::Viewport;

pub use algorithm::{newton_laws_algorithm, NewtonLawAction, NewtonLawStep, NewtonLawTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{newton_laws_motion, NewtonLawsMotion};
pub use state::NewtonLaws;
pub use timing::NewtonLawsTiming;
pub use view::{newton_laws_view, NewtonLawsView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Newton's Three Laws")
        .state(NewtonLaws::new())
        .view(newton_laws_view)
        .algorithm(newton_laws_algorithm)
        .motion(newton_laws_motion)
        .timing(NewtonLawsTiming::default())
        .build()
}
