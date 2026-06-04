use crate::{NeuralNet, HIDDEN_COUNT, INPUT_COUNT, OUTPUT_COUNT};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    InputHidden { input: usize, hidden: usize },
    HiddenOutput { hidden: usize, output: usize },
}

#[derive(Clone, Copy)]
pub enum NeuralAction {
    ShowInputs,
    FireToHidden { hidden: usize },
    FireToOutput { output: usize },
    Hold,
}

#[derive(Clone, Copy)]
pub struct NeuralStep {
    pub(crate) index: usize,
    pub(crate) action: NeuralAction,
}

pub struct NeuralTrace {
    pub(crate) steps: Vec<NeuralStep>,
    pub(crate) hidden_group_count: usize,
}

pub fn neural_net_algorithm(state: NeuralNet) -> NeuralTrace {
    debug_assert_eq!(state.input_count, INPUT_COUNT);
    debug_assert_eq!(state.hidden_count, HIDDEN_COUNT);
    debug_assert_eq!(state.output_count, OUTPUT_COUNT);

    let mut steps = Vec::new();
    steps.push(NeuralStep {
        index: 0,
        action: NeuralAction::ShowInputs,
    });

    let mut index = 1;
    for hidden in 0..HIDDEN_COUNT {
        steps.push(NeuralStep {
            index,
            action: NeuralAction::FireToHidden { hidden },
        });
        index += 1;
    }
    let hidden_group_count = HIDDEN_COUNT;

    for output in 0..OUTPUT_COUNT {
        steps.push(NeuralStep {
            index,
            action: NeuralAction::FireToOutput { output },
        });
        index += 1;
    }

    steps.push(NeuralStep {
        index,
        action: NeuralAction::Hold,
    });

    NeuralTrace {
        steps,
        hidden_group_count,
    }
}
