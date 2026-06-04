use codimate_core::{tween, Animated};

#[derive(Clone, Copy)]
pub struct ConnectionPulseMotion;

pub fn connection_pulse_motion() -> ConnectionPulseMotion {
    ConnectionPulseMotion
}

impl ConnectionPulseMotion {
    pub(crate) fn pulse_progress(self) -> Animated<f32> {
        tween(0.0, 1.0)
    }
}
