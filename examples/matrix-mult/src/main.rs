use codimate_example_matrix_mult::{
    explain, matrix_mult_algorithm, matrix_mult_motion, matrix_mult_view, MatrixMultTiming,
    MatrixMultiplication, DEFAULT_A, DEFAULT_B,
};

fn main() {
    explain("Matrix Multiplication")
        .state(MatrixMultiplication::new(DEFAULT_A, DEFAULT_B))
        .view(matrix_mult_view)
        .algorithm(matrix_mult_algorithm)
        .motion(matrix_mult_motion)
        .timing(MatrixMultTiming::default())
        .render("results/matrix-mult.mp4");
}
