use crate::{MatrixMultiplication, A_COLS, A_ROWS, B_COLS};

#[derive(Clone, Copy)]
pub struct MatrixStep {
    pub(crate) row: usize,
    pub(crate) col: usize,
    pub(crate) result_before: [[Option<i32>; B_COLS]; A_ROWS],
    pub(crate) value: i32,
    pub(crate) terms: [(i32, i32); A_COLS],
}

impl MatrixStep {
    pub(crate) fn formula(self) -> String {
        let products = self
            .terms
            .iter()
            .map(|(a, b)| format!("{a}*{b}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let values = self
            .terms
            .iter()
            .map(|(a, b)| (a * b).to_string())
            .collect::<Vec<_>>()
            .join(" + ");
        format!(
            "C[{}][{}] = {} = {} = {}",
            self.row, self.col, products, values, self.value
        )
    }
}

pub struct MatrixTrace {
    pub(crate) steps: Vec<MatrixStep>,
    pub(crate) result: [[i32; B_COLS]; A_ROWS],
}

pub fn matrix_mult_algorithm(state: MatrixMultiplication) -> MatrixTrace {
    let a = state.a();
    let b = state.b();
    let mut result = [[0; B_COLS]; A_ROWS];
    let mut result_before = [[None; B_COLS]; A_ROWS];
    let mut steps = Vec::new();

    for row in 0..A_ROWS {
        for col in 0..B_COLS {
            let mut value = 0;
            let mut terms = [(0, 0); A_COLS];
            for k in 0..A_COLS {
                terms[k] = (a[row][k], b[k][col]);
                value += a[row][k] * b[k][col];
            }
            steps.push(MatrixStep {
                row,
                col,
                result_before,
                value,
                terms,
            });
            result[row][col] = value;
            result_before[row][col] = Some(value);
        }
    }

    MatrixTrace { steps, result }
}
