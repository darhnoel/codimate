#[derive(Clone, Copy)]
pub enum ConnectionPulseStep {
    Pulse,
}

pub struct ConnectionPulseTrace {
    pub steps: Vec<ConnectionPulseStep>,
}

pub fn connection_pulse_algorithm(_state: crate::ConnectionPulse) -> ConnectionPulseTrace {
    ConnectionPulseTrace {
        steps: vec![ConnectionPulseStep::Pulse],
    }
}
