use crate::{Knapsack, CAPACITY, COLS, ITEM_COUNT, ROWS};

/// What a single step of the trace is doing.
#[derive(Clone, Copy)]
pub enum KnapsackAction {
    /// Show the items and the empty table (row 0 is the base case, all zeros).
    Init,
    /// Compute one cell `dp[item][cap]` from the two cells above it.
    Fill { item: usize, cap: usize },
    /// Table complete — backtrack to reveal which items were chosen.
    Done,
}

/// A full snapshot of the DP table after one step.
#[derive(Clone, Copy)]
pub struct KnapsackStep {
    pub(crate) index: usize,
    pub(crate) action: KnapsackAction,
    /// The table values computed so far.
    pub(crate) dp: [[u32; COLS]; ROWS],
    /// Which cells hold a real value yet.
    pub(crate) filled: [[bool; COLS]; ROWS],
    /// `dp[item-1][cap]` — the value if we skip the current item.
    pub(crate) skip: u32,
    /// `value + dp[item-1][cap-weight]` — the value if we take it (if it fits).
    pub(crate) take: Option<u32>,
    /// Whether taking beat skipping for the cell filled this step.
    pub(crate) took: bool,
    /// Cells on the backtracked solution path (set on the Done step).
    pub(crate) path: [[bool; COLS]; ROWS],
    /// Which items ended up in the optimal knapsack (set on the Done step).
    pub(crate) chosen: [bool; ITEM_COUNT],
}

pub struct KnapsackTrace {
    pub(crate) steps: Vec<KnapsackStep>,
    pub(crate) weights: [usize; ITEM_COUNT],
    pub(crate) values: [u32; ITEM_COUNT],
}

pub fn knapsack_algorithm(state: Knapsack) -> KnapsackTrace {
    let weights = state.weights;
    let values = state.values;

    let mut dp = [[0u32; COLS]; ROWS];
    let mut filled = [[false; COLS]; ROWS];
    // Base case: with zero items, every capacity yields value 0.
    for c in 0..COLS {
        filled[0][c] = true;
    }

    let mut steps = Vec::new();
    let mut index = 0;

    steps.push(KnapsackStep {
        index,
        action: KnapsackAction::Init,
        dp,
        filled,
        skip: 0,
        take: None,
        took: false,
        path: [[false; COLS]; ROWS],
        chosen: [false; ITEM_COUNT],
    });
    index += 1;

    for item in 1..=ITEM_COUNT {
        let wt = weights[item - 1];
        let val = values[item - 1];
        for cap in 0..COLS {
            let skip = dp[item - 1][cap];
            let take = if cap >= wt {
                Some(val + dp[item - 1][cap - wt])
            } else {
                None
            };
            let took = matches!(take, Some(t) if t > skip);
            dp[item][cap] = take.map_or(skip, |t| t.max(skip));
            filled[item][cap] = true;

            steps.push(KnapsackStep {
                index,
                action: KnapsackAction::Fill { item, cap },
                dp,
                filled,
                skip,
                take,
                took,
                path: [[false; COLS]; ROWS],
                chosen: [false; ITEM_COUNT],
            });
            index += 1;
        }
    }

    // Backtrack from dp[N][CAPACITY]: an item was taken whenever its row's value
    // differs from the row above it.
    let mut path = [[false; COLS]; ROWS];
    let mut chosen = [false; ITEM_COUNT];
    let mut cap = CAPACITY;
    for item in (1..=ITEM_COUNT).rev() {
        path[item][cap] = true;
        if dp[item][cap] != dp[item - 1][cap] {
            chosen[item - 1] = true;
            cap -= weights[item - 1];
        }
    }
    path[0][cap] = true;

    steps.push(KnapsackStep {
        index,
        action: KnapsackAction::Done,
        dp,
        filled,
        skip: 0,
        take: None,
        took: false,
        path,
        chosen,
    });

    KnapsackTrace {
        steps,
        weights,
        values,
    }
}
