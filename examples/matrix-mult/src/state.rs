pub const A_ROWS: usize = 2;
pub const A_COLS: usize = 3;
pub const B_COLS: usize = 2;

pub const DEFAULT_A: [[i32; A_COLS]; A_ROWS] = [[1, 2, 3], [4, 5, 6]];
pub const DEFAULT_B: [[i32; B_COLS]; A_COLS] = [[7, 8], [9, 10], [11, 12]];

#[derive(Clone, Copy)]
pub struct MatrixMultiplication {
    a: [[i32; A_COLS]; A_ROWS],
    b: [[i32; B_COLS]; A_COLS],
}

impl MatrixMultiplication {
    pub fn new(a: [[i32; A_COLS]; A_ROWS], b: [[i32; B_COLS]; A_COLS]) -> Self {
        Self { a, b }
    }

    pub(crate) fn a(self) -> [[i32; A_COLS]; A_ROWS] {
        self.a
    }

    pub(crate) fn b(self) -> [[i32; B_COLS]; A_COLS] {
        self.b
    }
}
