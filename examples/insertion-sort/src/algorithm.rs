use crate::{InsertionSort, N};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisualItem {
    pub(crate) id: usize,
    pub(crate) value: i32,
    pub(crate) origin_slot: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeldKey {
    pub(crate) item: VisualItem,
    pub(crate) origin_slot: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertionMovement {
    ShiftRight {
        item: VisualItem,
        from: usize,
        to: usize,
    },
    InsertKey {
        item: VisualItem,
        from: usize,
        to: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertionAction {
    ChooseKey { index: usize, key: i32 },
    Compare { left: usize, key: i32 },
    Shift { from: usize, to: usize, key: i32 },
    Insert { from: usize, to: usize, key: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InsertionStep {
    pub(crate) slots: [Option<VisualItem>; N],
    pub(crate) held: Option<HeldKey>,
    pub(crate) hole: Option<usize>,
    pub(crate) compare_left: Option<usize>,
    pub(crate) movement: Option<InsertionMovement>,
    pub(crate) action: InsertionAction,
}

impl InsertionStep {
    pub(crate) fn title(self) -> String {
        match self.action {
            InsertionAction::ChooseKey { index, key } => {
                format!("លើកលេខ {key} ចេញពីសន្ទស្សន៍ទីតាំង {index}")
            }
            InsertionAction::Compare { left, key } => {
                format!("ប្រៀបធៀបលេខគន្លឹះ {key} ជាមួយលេខនៅសន្ទស្សន៍ទីតាំង {left}")
            }
            InsertionAction::Shift { from, to, key } => {
                format!("រំកិលសន្ទស្សន៍ទីតាំងពី {from} ទៅសន្ទស្សន៍ទីតាំង {to} ដែលមានតម្លៃ {key}")
            }
            InsertionAction::Insert { from, to, key } => {
                format!("បញ្ចូលលេខគន្លឹះ {key} ពីសន្ទស្សន៍ទីតាំង {from} ចូលក្នុងសន្ទស្សន៍ទីតាំង {to}")
            }
        }
    }
}

pub struct InsertionTrace {
    pub(crate) steps: Vec<InsertionStep>,
    pub(crate) sorted: [i32; N],
}

pub fn insertion_sort_algorithm(state: InsertionSort) -> InsertionTrace {
    let mut slots = initial_slots(state.values());
    let mut steps = Vec::new();

    for i in 1..N {
        let key_item = slots[i].expect("key slot is occupied before lift");
        let key = key_item.value;
        slots[i] = None;
        let held = HeldKey {
            item: key_item,
            origin_slot: i,
        };
        let mut hole = i;

        steps.push(InsertionStep {
            slots,
            held: Some(held),
            hole: Some(hole),
            compare_left: None,
            movement: None,
            action: InsertionAction::ChooseKey { index: i, key },
        });

        while hole > 0 {
            let left = hole - 1;
            steps.push(InsertionStep {
                slots,
                held: Some(held),
                hole: Some(hole),
                compare_left: Some(left),
                movement: None,
                action: InsertionAction::Compare { left, key },
            });

            let left_item = slots[left].expect("left slot is occupied during compare");
            if left_item.value <= key {
                break;
            }

            steps.push(InsertionStep {
                slots,
                held: Some(held),
                hole: Some(hole),
                compare_left: None,
                movement: Some(InsertionMovement::ShiftRight {
                    item: left_item,
                    from: left,
                    to: hole,
                }),
                action: InsertionAction::Shift {
                    from: left,
                    to: hole,
                    key,
                },
            });

            slots[hole] = Some(left_item);
            slots[left] = None;
            hole = left;
        }

        steps.push(InsertionStep {
            slots,
            held: Some(held),
            hole: Some(hole),
            compare_left: None,
            movement: Some(InsertionMovement::InsertKey {
                item: key_item,
                from: held.origin_slot,
                to: hole,
            }),
            action: InsertionAction::Insert {
                from: held.origin_slot,
                to: hole,
                key,
            },
        });

        slots[hole] = Some(key_item);
    }

    InsertionTrace {
        steps,
        sorted: values_from_slots(slots),
    }
}

fn initial_slots(values: [i32; N]) -> [Option<VisualItem>; N] {
    std::array::from_fn(|idx| {
        Some(VisualItem {
            id: idx,
            value: values[idx],
            origin_slot: idx,
        })
    })
}

fn values_from_slots(slots: [Option<VisualItem>; N]) -> [i32; N] {
    std::array::from_fn(|idx| slots[idx].expect("final slot is occupied").value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_VALUES;

    #[test]
    fn first_step_lifts_key_and_leaves_hole() {
        let trace = insertion_sort_algorithm(InsertionSort::new(DEFAULT_VALUES));
        let first = trace.steps[0];

        assert_eq!(first.hole, Some(1));
        assert_eq!(first.held.unwrap().item.value, 3);
        assert_eq!(first.slots[1], None);
    }

    #[test]
    fn shift_moves_one_larger_item_right_while_key_stays_held() {
        let trace = insertion_sort_algorithm(InsertionSort::new(DEFAULT_VALUES));
        let shift = trace
            .steps
            .iter()
            .copied()
            .find(|step| matches!(step.movement, Some(InsertionMovement::ShiftRight { .. })))
            .expect("trace should contain a shift");

        assert_eq!(shift.held.unwrap().item.value, 3);
        assert_eq!(shift.hole, Some(1));
        assert_eq!(shift.slots[1], None);
        assert_eq!(shift.slots[0].unwrap().value, 8);
        assert_eq!(
            shift.movement,
            Some(InsertionMovement::ShiftRight {
                item: VisualItem {
                    id: 0,
                    value: 8,
                    origin_slot: 0,
                },
                from: 0,
                to: 1,
            })
        );
    }

    #[test]
    fn insert_drops_held_key_into_current_hole() {
        let trace = insertion_sort_algorithm(InsertionSort::new(DEFAULT_VALUES));
        let insert = trace
            .steps
            .iter()
            .copied()
            .find(|step| matches!(step.movement, Some(InsertionMovement::InsertKey { .. })))
            .expect("trace should contain an insert");

        assert_eq!(insert.held.unwrap().item.value, 3);
        assert_eq!(insert.hole, Some(0));
        assert_eq!(insert.slots[0], None);
        assert_eq!(
            insert.movement,
            Some(InsertionMovement::InsertKey {
                item: VisualItem {
                    id: 1,
                    value: 3,
                    origin_slot: 1,
                },
                from: 1,
                to: 0,
            })
        );
    }

    #[test]
    fn final_values_are_sorted() {
        let trace = insertion_sort_algorithm(InsertionSort::new(DEFAULT_VALUES));
        assert_eq!(trace.sorted, [1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
