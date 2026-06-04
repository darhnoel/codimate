use crate::{ThreeSum, N};

#[derive(Clone)]
pub enum ThreeSumAction {
    IntroduceInput,
    Sort,
    FixAnchor {
        i: usize,
    },
    SetPointers {
        i: usize,
        left: usize,
        right: usize,
    },
    Compare {
        i: usize,
        left: usize,
        right: usize,
        sum: i32,
    },
    MoveLeft {
        i: usize,
        from: usize,
        to: usize,
        sum: i32,
    },
    MoveRight {
        i: usize,
        from: usize,
        to: usize,
        sum: i32,
    },
    Found {
        i: usize,
        left: usize,
        right: usize,
        triplet: [i32; 3],
        result_index: usize,
    },
    SkipDuplicate {
        from: usize,
        to: usize,
        value: i32,
    },
    Done,
}

#[derive(Clone)]
pub struct ThreeSumStep {
    pub(crate) action: ThreeSumAction,
    pub(crate) results_before: Vec<[i32; 3]>,
}

impl ThreeSumStep {
    pub(crate) fn title(&self, sorted: &[i32; N]) -> String {
        match &self.action {
            ThreeSumAction::IntroduceInput => "Start with the original input array".to_string(),
            ThreeSumAction::Sort => "Sort first so two pointers can move predictably".to_string(),
            ThreeSumAction::FixAnchor { i } => {
                format!("Fix anchor {} at index {}", sorted[*i], i)
            }
            ThreeSumAction::SetPointers { i, .. } => {
                format!("Find two numbers that sum to {}", -sorted[*i])
            }
            ThreeSumAction::Compare {
                i,
                left,
                right,
                sum,
            } => format!(
                "{} + {} + {} = {}",
                sorted[*i], sorted[*left], sorted[*right], sum
            ),
            ThreeSumAction::MoveLeft { sum, .. } => {
                format!("Sum {sum} is too small, move left pointer right")
            }
            ThreeSumAction::MoveRight { sum, .. } => {
                format!("Sum {sum} is too large, move right pointer left")
            }
            ThreeSumAction::Found { triplet, .. } => {
                format!("Found [{}, {}, {}]", triplet[0], triplet[1], triplet[2])
            }
            ThreeSumAction::SkipDuplicate { value, .. } => {
                format!("Skip duplicate anchor {value}")
            }
            ThreeSumAction::Done => "All unique triplets found".to_string(),
        }
    }
}

pub struct ThreeSumTrace {
    pub(crate) input: [i32; N],
    pub(crate) sorted: [i32; N],
    pub(crate) sort_order: [usize; N],
    pub(crate) steps: Vec<ThreeSumStep>,
    pub(crate) results: Vec<[i32; 3]>,
}

pub fn three_sum_algorithm(state: ThreeSum) -> ThreeSumTrace {
    let input = state.values();
    let mut pairs = input
        .iter()
        .copied()
        .enumerate()
        .map(|(index, value)| (value, index))
        .collect::<Vec<_>>();
    pairs.sort_by_key(|&(value, index)| (value, index));

    let mut sorted = [0; N];
    let mut sort_order = [0; N];
    for (sorted_index, (value, original_index)) in pairs.into_iter().enumerate() {
        sorted[sorted_index] = value;
        sort_order[sorted_index] = original_index;
    }

    let mut steps = Vec::new();
    let mut results = Vec::new();
    steps.push(ThreeSumStep {
        action: ThreeSumAction::IntroduceInput,
        results_before: results.clone(),
    });
    steps.push(ThreeSumStep {
        action: ThreeSumAction::Sort,
        results_before: results.clone(),
    });

    for i in 0..N - 2 {
        if i > 0 && sorted[i] == sorted[i - 1] {
            steps.push(ThreeSumStep {
                action: ThreeSumAction::SkipDuplicate {
                    from: i,
                    to: i + 1,
                    value: sorted[i],
                },
                results_before: results.clone(),
            });
            continue;
        }

        steps.push(ThreeSumStep {
            action: ThreeSumAction::FixAnchor { i },
            results_before: results.clone(),
        });

        let mut left = i + 1;
        let mut right = N - 1;
        steps.push(ThreeSumStep {
            action: ThreeSumAction::SetPointers { i, left, right },
            results_before: results.clone(),
        });

        while left < right {
            let sum = sorted[i] + sorted[left] + sorted[right];
            steps.push(ThreeSumStep {
                action: ThreeSumAction::Compare {
                    i,
                    left,
                    right,
                    sum,
                },
                results_before: results.clone(),
            });

            match sum.cmp(&0) {
                std::cmp::Ordering::Less => {
                    let from = left;
                    left += 1;
                    steps.push(ThreeSumStep {
                        action: ThreeSumAction::MoveLeft {
                            i,
                            from,
                            to: left,
                            sum,
                        },
                        results_before: results.clone(),
                    });
                }
                std::cmp::Ordering::Greater => {
                    let from = right;
                    right -= 1;
                    steps.push(ThreeSumStep {
                        action: ThreeSumAction::MoveRight {
                            i,
                            from,
                            to: right,
                            sum,
                        },
                        results_before: results.clone(),
                    });
                }
                std::cmp::Ordering::Equal => {
                    let triplet = [sorted[i], sorted[left], sorted[right]];
                    steps.push(ThreeSumStep {
                        action: ThreeSumAction::Found {
                            i,
                            left,
                            right,
                            triplet,
                            result_index: results.len(),
                        },
                        results_before: results.clone(),
                    });
                    results.push(triplet);

                    let left_value = sorted[left];
                    let right_value = sorted[right];
                    while left < right && sorted[left] == left_value {
                        left += 1;
                    }
                    while left < right && sorted[right] == right_value {
                        right -= 1;
                    }
                    if left < right {
                        steps.push(ThreeSumStep {
                            action: ThreeSumAction::SetPointers { i, left, right },
                            results_before: results.clone(),
                        });
                    }
                }
            }
        }
    }

    steps.push(ThreeSumStep {
        action: ThreeSumAction::Done,
        results_before: results.clone(),
    });

    ThreeSumTrace {
        input,
        sorted,
        sort_order,
        steps,
        results,
    }
}
