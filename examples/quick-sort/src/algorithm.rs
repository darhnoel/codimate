use crate::{QuickSort, N};

#[derive(Clone, Copy)]
pub enum QuickAction {
    ChoosePivot,
    Compare { index: usize },
    Swap { left: usize, right: usize },
    PlacePivot { from: usize, to: usize },
}

#[derive(Clone, Copy)]
pub struct QuickStep {
    pub(crate) depth: usize,
    pub(crate) low: usize,
    pub(crate) high: usize,
    pub(crate) values_before: [i32; N],
    pub(crate) pivot_index: usize,
    pub(crate) pivot_value: i32,
    pub(crate) store_index: usize,
    pub(crate) action: QuickAction,
}

impl QuickStep {
    pub(crate) fn title(&self) -> String {
        match self.action {
            QuickAction::ChoosePivot => {
                format!(
                    "Partition [{}..{}], choose pivot {}",
                    self.low, self.high, self.pivot_value
                )
            }
            QuickAction::Compare { index } => {
                format!(
                    "Compare {} with pivot {}",
                    self.values_before[index], self.pivot_value
                )
            }
            QuickAction::Swap { left, right } => {
                format!(
                    "Swap {} and {}",
                    self.values_before[left], self.values_before[right]
                )
            }
            QuickAction::PlacePivot { to, .. } => {
                format!("Place pivot {} at index {}", self.pivot_value, to)
            }
        }
    }
}

pub struct QuickTrace {
    pub(crate) steps: Vec<QuickStep>,
    pub(crate) sorted: [i32; N],
}

pub fn quick_sort_algorithm(state: QuickSort) -> QuickTrace {
    let mut values = state.values();
    let mut steps = Vec::new();
    quick_sort_range(&mut values, 0, N - 1, 0, &mut steps);
    QuickTrace {
        steps,
        sorted: values,
    }
}

fn quick_sort_range(
    values: &mut [i32; N],
    low: usize,
    high: usize,
    depth: usize,
    steps: &mut Vec<QuickStep>,
) {
    if low >= high {
        return;
    }

    let pivot = partition(values, low, high, depth, steps);
    if pivot > low {
        quick_sort_range(values, low, pivot - 1, depth + 1, steps);
    }
    quick_sort_range(values, pivot + 1, high, depth + 1, steps);
}

fn partition(
    values: &mut [i32; N],
    low: usize,
    high: usize,
    depth: usize,
    steps: &mut Vec<QuickStep>,
) -> usize {
    let pivot_value = values[high];
    let mut store = low;

    push_step(
        steps,
        QuickStep {
            depth,
            low,
            high,
            values_before: *values,
            pivot_index: high,
            pivot_value,
            store_index: store,
            action: QuickAction::ChoosePivot,
        },
    );

    for index in low..high {
        push_step(
            steps,
            QuickStep {
                depth,
                low,
                high,
                values_before: *values,
                pivot_index: high,
                pivot_value,
                store_index: store,
                action: QuickAction::Compare { index },
            },
        );

        if values[index] <= pivot_value {
            let before = *values;
            values.swap(store, index);
            if store != index {
                push_step(
                    steps,
                    QuickStep {
                        depth,
                        low,
                        high,
                        values_before: before,
                        pivot_index: high,
                        pivot_value,
                        store_index: store,
                        action: QuickAction::Swap {
                            left: store,
                            right: index,
                        },
                    },
                );
            }
            store += 1;
        }
    }

    let before = *values;
    values.swap(store, high);
    push_step(
        steps,
        QuickStep {
            depth,
            low,
            high,
            values_before: before,
            pivot_index: high,
            pivot_value,
            store_index: store,
            action: QuickAction::PlacePivot {
                from: high,
                to: store,
            },
        },
    );

    store
}

fn push_step(steps: &mut Vec<QuickStep>, step: QuickStep) {
    steps.push(step);
}
