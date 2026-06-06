mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate_animation::Playable;
use codimate_layout::Viewport;

pub use algorithm::{
    transformer_architecture_algorithm, TransformerArchitectureAction, TransformerArchitectureStep,
    TransformerArchitectureTrace,
};
pub use builder::{explain, ExplainBuilder};
pub use motion::{transformer_architecture_motion, TransformerArchitectureMotion};
pub use state::TransformerArchitecture;
pub use timing::TransformerArchitectureTiming;
pub use view::{transformer_architecture_view, TransformerArchitectureView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Attention Is All You Need Architecture")
        .state(TransformerArchitecture::new())
        .view(transformer_architecture_view)
        .algorithm(transformer_architecture_algorithm)
        .motion(transformer_architecture_motion)
        .timing(TransformerArchitectureTiming::default())
        .build()
}
