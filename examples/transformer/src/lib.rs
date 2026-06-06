mod builder;
mod layout;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate_animation::Playable;
use codimate_layout::Viewport;

pub use builder::{explain, ExplainBuilder};
pub use motion::{transformer_motion, TransformerMotion};
pub use state::{transformer_algorithm, TransformerPhase, TransformerState, TransformerTrace};
pub use timing::TransformerTiming;

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Transformer")
        .state(TransformerState::default())
        .algorithm(transformer_algorithm)
        .motion(transformer_motion)
        .timing(TransformerTiming::default())
        .build()
}
