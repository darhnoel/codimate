mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate::Playable;
use codimate::Viewport;

pub use algorithm::{matrix_mult_algorithm, MatrixStep, MatrixTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{matrix_mult_motion, MatrixMultMotion};
pub use state::{MatrixMultiplication, A_COLS, A_ROWS, B_COLS, DEFAULT_A, DEFAULT_B};
pub use timing::MatrixMultTiming;
pub use view::{matrix_mult_view, MatrixMultView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Matrix Multiplication")
        .state(MatrixMultiplication::new(DEFAULT_A, DEFAULT_B))
        .view(matrix_mult_view)
        .algorithm(matrix_mult_algorithm)
        .motion(matrix_mult_motion)
        .timing(MatrixMultTiming::default())
        .build()
}
