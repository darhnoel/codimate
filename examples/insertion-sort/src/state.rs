pub const N: usize = 8;
pub const DEFAULT_VALUES: [i32; N] = [8, 3, 5, 1, 7, 4, 6, 2];

#[derive(Clone, Copy)]
pub struct InsertionSort {
    values: [i32; N],
}

impl InsertionSort {
    pub fn new(values: [i32; N]) -> Self {
        Self { values }
    }

    pub fn values(self) -> [i32; N] {
        self.values
    }
}
