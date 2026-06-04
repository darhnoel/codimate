use crate::{
    style::*, MatrixMultMotion, MatrixMultTiming, MatrixMultiplication, MatrixStep, MatrixTrace,
    A_COLS, A_ROWS, B_COLS,
};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::Viewport;

const VIEW_W: f32 = 900.0;
const VIEW_H: f32 = 520.0;
const CELL_W: f32 = 62.0;
const CELL_H: f32 = 44.0;
const GAP: f32 = 8.0;
const CELL_RADIUS: f32 = 8.0;
const FONT_SIZE: f32 = 17.0;

const A_X: f32 = 78.0;
const A_Y: f32 = 202.0;
const B_X: f32 = 362.0;
const B_Y: f32 = 178.0;
const C_X: f32 = 650.0;
const C_Y: f32 = 202.0;
const FORMULA_Y: f32 = 410.0;

#[derive(Clone, Copy)]
pub struct MatrixMultView;

#[derive(Clone, Copy)]
enum MatrixKind {
    A,
    B,
    C,
}

pub fn matrix_mult_view() -> MatrixMultView {
    MatrixMultView
}

pub(crate) fn build_matrix_mult(
    name: &'static str,
    state: MatrixMultiplication,
    trace: MatrixTrace,
    motion: MatrixMultMotion,
    timing: MatrixMultTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();
    anims.push(animation("intro", timing.intro, overview_scene(state)));

    for (i, step) in trace.steps.iter().enumerate() {
        anims.push(animation(
            format!("cell-{}-{}", step.row, step.col),
            timing.compute_cell,
            step_scene(state, step, motion, i),
        ));
    }

    anims.push(animation(
        "done",
        timing.final_hold,
        final_scene(state, &trace.result),
    ));

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.30
}

fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = 0.552_284_8 * r;
    Path {
        segments: vec![
            Segment::Line(Vec2::new(x + r, y), Vec2::new(x + w - r, y)),
            Segment::Cubic(
                Vec2::new(x + w - r, y),
                Vec2::new(x + w - r + k, y),
                Vec2::new(x + w, y + r - k),
                Vec2::new(x + w, y + r),
            ),
            Segment::Line(Vec2::new(x + w, y + r), Vec2::new(x + w, y + h - r)),
            Segment::Cubic(
                Vec2::new(x + w, y + h - r),
                Vec2::new(x + w, y + h - r + k),
                Vec2::new(x + w - r + k, y + h),
                Vec2::new(x + w - r, y + h),
            ),
            Segment::Line(Vec2::new(x + w - r, y + h), Vec2::new(x + r, y + h)),
            Segment::Cubic(
                Vec2::new(x + r, y + h),
                Vec2::new(x + r - k, y + h),
                Vec2::new(x, y + h - r + k),
                Vec2::new(x, y + h - r),
            ),
            Segment::Line(Vec2::new(x, y + h - r), Vec2::new(x, y + r)),
            Segment::Cubic(
                Vec2::new(x, y + r),
                Vec2::new(x, y + r - k),
                Vec2::new(x + r - k, y),
                Vec2::new(x + r, y),
            ),
        ],
        closed: true,
    }
}

fn style(fill: Color, stroke_width: f32, stroke_color: Color) -> Style {
    Style::new().fill(fill).stroke(stroke_width, stroke_color)
}

fn label(x: f32, y: f32, content: impl Into<String>, font_size: f32, fill: Color) -> Text {
    text()
        .x(x)
        .y(y)
        .text(content.into())
        .font_size(font_size)
        .fill(fill)
}

fn centered_label(x: f32, y: f32, content: impl Into<String>, font_size: f32, fill: Color) -> Text {
    let content = content.into();
    label(
        x - text_width(&content, font_size) / 2.0,
        y,
        content,
        font_size,
        fill,
    )
}

fn cell_x(kind: MatrixKind, col: usize) -> f32 {
    let origin = match kind {
        MatrixKind::A => A_X,
        MatrixKind::B => B_X,
        MatrixKind::C => C_X,
    };
    origin + col as f32 * (CELL_W + GAP)
}

fn cell_y(kind: MatrixKind, row: usize) -> f32 {
    let origin = match kind {
        MatrixKind::A => A_Y,
        MatrixKind::B => B_Y,
        MatrixKind::C => C_Y,
    };
    origin + row as f32 * (CELL_H + GAP)
}

fn add_background(mut sc: Scene, subtitle: impl Into<String>) -> Scene {
    sc = sc.node(
        path_node()
            .path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
            .style(style(BG, 0.0, BG)),
    );
    sc = sc.node(label(54.0, 48.0, "Matrix Multiplication", 28.0, INK));
    sc.node(label(54.0, 82.0, subtitle, 16.0, MUTED))
}

fn add_matrix_labels(mut sc: Scene) -> Scene {
    sc = sc.node(centered_label(
        A_X + CELL_W * 1.5 + GAP,
        A_Y - 34.0,
        "A",
        22.0,
        INK,
    ));
    sc = sc.node(centered_label(
        B_X + CELL_W + GAP / 2.0,
        B_Y - 34.0,
        "B",
        22.0,
        INK,
    ));
    sc = sc.node(centered_label(
        C_X + CELL_W + GAP / 2.0,
        C_Y - 34.0,
        "C",
        22.0,
        INK,
    ));
    sc = sc.node(centered_label(304.0, A_Y + 42.0, "x", 25.0, MUTED));
    sc.node(centered_label(594.0, A_Y + 42.0, "=", 25.0, MUTED))
}

fn add_dimensions(mut sc: Scene) -> Scene {
    sc = sc.node(centered_label(
        A_X + CELL_W * 1.5 + GAP,
        A_Y + A_ROWS as f32 * (CELL_H + GAP) + 16.0,
        "2 x 3",
        12.0,
        MUTED,
    ));
    sc = sc.node(centered_label(
        B_X + CELL_W + GAP / 2.0,
        B_Y + A_COLS as f32 * (CELL_H + GAP) + 16.0,
        "3 x 2",
        12.0,
        MUTED,
    ));
    sc.node(centered_label(
        C_X + CELL_W + GAP / 2.0,
        C_Y + A_ROWS as f32 * (CELL_H + GAP) + 16.0,
        "2 x 2",
        12.0,
        MUTED,
    ))
}

fn cell_style(fill: impl IntoAnimated<Color>, active: bool) -> Animated<Style> {
    let fill = fill.into_animated();
    Animated::new(move |t| {
        let stroke = if active { BORDER } else { SUBTLE_BORDER };
        let width = if active { 2.2 } else { 1.2 };
        style(fill.resolve(t), width, stroke)
    })
}

fn add_cell(
    mut sc: Scene,
    kind: MatrixKind,
    row: usize,
    col: usize,
    value: impl Into<String>,
    fill: impl IntoAnimated<Color>,
    active: bool,
) -> Scene {
    let x = cell_x(kind, col);
    let y = cell_y(kind, row);
    let text = value.into();
    sc = sc.node(
        path_node()
            .path(rounded_rect_path(x, y, CELL_W, CELL_H, CELL_RADIUS))
            .style(cell_style(fill, active)),
    );
    sc.node(centered_label(
        x + CELL_W / 2.0,
        y + CELL_H / 2.0 + FONT_SIZE * 0.33,
        text,
        FONT_SIZE,
        INK,
    ))
}

fn add_matrices(
    mut sc: Scene,
    state: MatrixMultiplication,
    step: Option<&MatrixStep>,
    motion: MatrixMultMotion,
) -> Scene {
    let a = state.a();
    let b = state.b();

    for (row, row_values) in a.iter().enumerate() {
        for (col, value) in row_values.iter().enumerate() {
            let active = step.is_some_and(|step| step.row == row);
            let fill = if active { ROW_FILL } else { CELL };
            sc = add_cell(sc, MatrixKind::A, row, col, value.to_string(), fill, active);
        }
    }

    for (row, row_values) in b.iter().enumerate() {
        for (col, value) in row_values.iter().enumerate() {
            let active = step.is_some_and(|step| step.col == col);
            let fill = if active { COL_FILL } else { CELL };
            sc = add_cell(sc, MatrixKind::B, row, col, value.to_string(), fill, active);
        }
    }

    for row in 0..A_ROWS {
        for col in 0..B_COLS {
            let (value, fill, active) = match step {
                Some(step) if step.row == row && step.col == col => (
                    step.value.to_string(),
                    motion.reveal(CELL_DIM, ACTIVE_RESULT),
                    true,
                ),
                Some(step) => match step.result_before[row][col] {
                    Some(value) => (value.to_string(), RESULT_FILL.into_animated(), false),
                    None => ("".to_string(), CELL_DIM.into_animated(), false),
                },
                None => ("".to_string(), CELL_DIM.into_animated(), false),
            };
            sc = add_cell(sc, MatrixKind::C, row, col, value, fill, active);
        }
    }

    sc
}

fn add_formula(sc: Scene, content: impl Into<String>) -> Scene {
    let content = content.into();
    sc.node(centered_label(VIEW_W / 2.0, FORMULA_Y, content, 18.0, INK))
}

fn overview_scene(state: MatrixMultiplication) -> Scene {
    let mut sc = add_background(scene(), "Each output cell is one row-column dot product");
    sc = add_matrix_labels(sc);
    sc = add_dimensions(sc);
    sc = add_matrices(sc, state, None, MatrixMultMotion);
    add_formula(sc, "A x B = C")
}

fn step_scene(
    state: MatrixMultiplication,
    step: &MatrixStep,
    motion: MatrixMultMotion,
    index: usize,
) -> Scene {
    let mut sc = add_background(
        scene(),
        format!("Compute output cell {} of {}", index + 1, A_ROWS * B_COLS),
    );
    sc = add_matrix_labels(sc);
    sc = add_dimensions(sc);
    sc = add_matrices(sc, state, Some(step), motion);
    add_formula(sc, step.formula())
}

fn final_scene(state: MatrixMultiplication, result: &[[i32; B_COLS]; A_ROWS]) -> Scene {
    let mut sc = add_background(scene(), "All row-column products are complete");
    sc = add_matrix_labels(sc);
    sc = add_dimensions(sc);
    sc = add_matrices(sc, state, None, MatrixMultMotion);
    for (row, row_values) in result.iter().enumerate() {
        for (col, value) in row_values.iter().enumerate() {
            sc = add_cell(
                sc,
                MatrixKind::C,
                row,
                col,
                value.to_string(),
                RESULT_FILL,
                false,
            );
        }
    }
    add_formula(sc, "A x B = C")
}
