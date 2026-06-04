use crate::{MergeSort, N};

#[derive(Clone, Copy)]
pub struct MergeStep {
    pub(crate) pass: usize,
    pub(crate) width: usize,
    pub(crate) start: usize,
    pub(crate) mid: usize,
    pub(crate) end: usize,
    pub(crate) source: [i32; N],
    pub(crate) output_before: [Option<i32>; N],
    pub(crate) consumed_before: [bool; N],
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
    pub(crate) winner: usize,
    pub(crate) output: usize,
}

impl MergeStep {
    pub(crate) fn pass_label(&self) -> String {
        format!(
            "Pass {}: merge runs of {} into {}",
            self.pass + 1,
            self.width,
            self.width * 2
        )
    }
}

pub struct MergeTrace {
    pub(crate) steps: Vec<MergeStep>,
    pub(crate) pass_results: Vec<[i32; N]>,
    pub(crate) sorted: [i32; N],
}

pub fn merge_sort_algorithm(state: MergeSort) -> MergeTrace {
    let mut steps = Vec::new();
    let mut pass_results = Vec::new();
    let mut current = state.values();
    let mut width = 1;
    let mut pass = 0;

    while width < N {
        let source = current;
        let mut next = current;
        let mut output_before = [None; N];
        let mut consumed_before = [false; N];

        for start in (0..N).step_by(width * 2) {
            let mid = (start + width).min(N);
            let end = (start + width * 2).min(N);
            let mut left = start;
            let mut right = mid;
            let mut output = start;

            while left < mid || right < end {
                let left_candidate = (left < mid).then_some(left);
                let right_candidate = (right < end).then_some(right);
                let winner = match (left_candidate, right_candidate) {
                    (Some(l), Some(r)) if source[l] <= source[r] => l,
                    (Some(_), Some(r)) => r,
                    (Some(l), None) => l,
                    (None, Some(r)) => r,
                    (None, None) => unreachable!(),
                };

                steps.push(MergeStep {
                    pass,
                    width,
                    start,
                    mid,
                    end,
                    source,
                    output_before,
                    consumed_before,
                    left: left_candidate,
                    right: right_candidate,
                    winner,
                    output,
                });

                output_before[output] = Some(source[winner]);
                consumed_before[winner] = true;
                next[output] = source[winner];
                if Some(winner) == left_candidate {
                    left += 1;
                } else {
                    right += 1;
                }
                output += 1;
            }
        }

        current = next;
        pass_results.push(current);
        width *= 2;
        pass += 1;
    }

    MergeTrace {
        steps,
        pass_results,
        sorted: current,
    }
}
