use crate::TransformerArchitecture;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformerArchitectureAction {
    TranslationProblem,
    EncoderReadsDecoderWrites,
    OriginalLayout,
    InputEmbedding,
    PositionalEncoding,
    EncoderBlock,
    SelfAttentionIntuition,
    MultiHeadAttentionIntuition,
    AddNorm,
    FeedForward,
    EncoderRepeats,
    EncoderMemory,
    DecoderInput,
    MaskedSelfAttention,
    CrossAttention,
    DecoderBlock,
    LinearSoftmax,
    FullArchitecture,
}

#[derive(Clone, Copy)]
pub struct TransformerArchitectureStep {
    pub(crate) index: usize,
    pub(crate) action: TransformerArchitectureAction,
    pub(crate) input: &'static [&'static str],
    pub(crate) output: &'static str,
}

pub struct TransformerArchitectureTrace {
    pub(crate) steps: Vec<TransformerArchitectureStep>,
}

pub fn transformer_architecture_algorithm(
    state: TransformerArchitecture,
) -> TransformerArchitectureTrace {
    use TransformerArchitectureAction::*;

    let actions = [
        TranslationProblem,
        EncoderReadsDecoderWrites,
        OriginalLayout,
        InputEmbedding,
        PositionalEncoding,
        EncoderBlock,
        SelfAttentionIntuition,
        MultiHeadAttentionIntuition,
        AddNorm,
        FeedForward,
        EncoderRepeats,
        EncoderMemory,
        DecoderInput,
        MaskedSelfAttention,
        CrossAttention,
        DecoderBlock,
        LinearSoftmax,
        FullArchitecture,
    ];

    TransformerArchitectureTrace {
        steps: actions
            .into_iter()
            .enumerate()
            .map(|(index, action)| TransformerArchitectureStep {
                index,
                action,
                input: state.input,
                output: state.output,
            })
            .collect(),
    }
}
