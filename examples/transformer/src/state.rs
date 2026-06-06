#[derive(Clone, Copy)]
pub enum TransformerPhase {
    Encoder,
    Decoder,
    Cross,
}

impl TransformerPhase {
    pub fn name(&self) -> &'static str {
        match self {
            TransformerPhase::Encoder => "encoder",
            TransformerPhase::Decoder => "decoder",
            TransformerPhase::Cross => "cross",
        }
    }
}

pub type TransformerTrace = Vec<TransformerPhase>;

#[derive(Clone, Copy)]
pub struct TransformerState {
    pub show_pulses: bool,
}

impl Default for TransformerState {
    fn default() -> Self {
        Self { show_pulses: true }
    }
}

pub fn transformer_algorithm(_state: TransformerState) -> TransformerTrace {
    vec![
        TransformerPhase::Encoder,
        TransformerPhase::Decoder,
        TransformerPhase::Cross,
    ]
}
