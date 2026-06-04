pub const DEFAULT_VALUES: [i32; N] = [38, 27, 43, 3, 9, 82, 10, 15];
pub const N: usize = 8;

#[derive(Clone, Copy)]
pub struct MergeSort {
    values: [i32; N],
}

impl MergeSort {
    pub fn new(values: [i32; N]) -> Self {
        Self { values }
    }

    pub(crate) fn values(self) -> [i32; N] {
        self.values
    }
}
