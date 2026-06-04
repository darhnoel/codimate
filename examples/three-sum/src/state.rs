pub const N: usize = 6;
pub const DEFAULT_VALUES: [i32; N] = [-1, 0, 1, 2, -1, -4];

#[derive(Clone, Copy)]
pub struct ThreeSum {
    values: [i32; N],
}

impl ThreeSum {
    pub fn new(values: [i32; N]) -> Self {
        Self { values }
    }

    pub(crate) fn values(self) -> [i32; N] {
        self.values
    }
}
