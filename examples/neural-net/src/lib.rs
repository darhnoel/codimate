mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate_animation::Playable;
use codimate_layout::Viewport;

pub use algorithm::{neural_net_algorithm, Edge, NeuralAction, NeuralStep, NeuralTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{neural_net_motion, NeuralNetMotion};
pub use state::{NeuralNet, HIDDEN_COUNT, INPUT_COUNT, OUTPUT_COUNT};
pub use timing::NeuralNetTiming;
pub use view::{neural_net_view, NeuralNetView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Neural Net")
        .state(NeuralNet::new())
        .view(neural_net_view)
        .algorithm(neural_net_algorithm)
        .motion(neural_net_motion)
        .timing(NeuralNetTiming::default())
        .build()
}
