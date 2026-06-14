mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate::Playable;
use codimate::Viewport;

pub use algorithm::{connection_pulse_algorithm, ConnectionPulseStep, ConnectionPulseTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{connection_pulse_motion, ConnectionPulseMotion};
pub use state::ConnectionPulse;
pub use timing::ConnectionPulseTiming;
pub use view::{connection_pulse_view, ConnectionPulseView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Connection Pulse")
        .state(ConnectionPulse::new())
        .view(connection_pulse_view)
        .algorithm(connection_pulse_algorithm)
        .motion(connection_pulse_motion)
        .timing(ConnectionPulseTiming::default())
        .build()
}
