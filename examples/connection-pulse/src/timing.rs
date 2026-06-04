#[derive(Clone, Copy)]
pub struct ConnectionPulseTiming {
    pub pulse: f32,
}

impl Default for ConnectionPulseTiming {
    fn default() -> Self {
        Self { pulse: 2.0 }
    }
}
