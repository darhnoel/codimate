/// Number of items the thief is choosing from.
pub const ITEM_COUNT: usize = 4;
/// Capacity of the knapsack (max total weight).
pub const CAPACITY: usize = 7;
/// DP table rows: one per "first i items" prefix, plus the empty prefix.
pub const ROWS: usize = ITEM_COUNT + 1;
/// DP table columns: one per capacity 0..=CAPACITY.
pub const COLS: usize = CAPACITY + 1;

/// The input: a small set of items (each with a weight and a value) and the
/// knapsack capacity. Chosen so the answer genuinely combines two items
/// (B + C, weight 3+4 = 7, value 4+5 = 9) rather than just grabbing the
/// single most valuable one.
#[derive(Clone, Copy)]
pub struct Knapsack {
    pub(crate) weights: [usize; ITEM_COUNT],
    pub(crate) values: [u32; ITEM_COUNT],
}

impl Knapsack {
    pub fn new() -> Self {
        Self {
            weights: [1, 3, 4, 5],
            values: [1, 4, 5, 7],
        }
    }
}

impl Default for Knapsack {
    fn default() -> Self {
        Self::new()
    }
}
