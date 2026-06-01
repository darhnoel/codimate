use codimate_animation::{animation, sequence};
use codimate_core::{rect, scene, tween, Color, IntoAnimated, Scene, Text};
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;

// ── Data ────────────────────────────────────────────────────────────────

struct Dims {
    a_rows: usize,
    a_cols: usize,
    b_cols: usize,
}

const DIMS: Dims = Dims {
    a_rows: 2,
    a_cols: 3,
    b_cols: 2,
};

const A_DATA: [[i32; 3]; 2] = [[1, 2, 3], [4, 5, 6]];
const B_DATA: [[i32; 2]; 3] = [[7, 8], [9, 10], [11, 12]];
const C_DATA: [[i32; 2]; 2] = [[58, 64], [139, 154]];

const CELL_W: f32 = 50.0;
const CELL_H: f32 = 34.0;
const GAP: f32 = 6.0;

const A_X: f32 = 40.0;
const A_Y: f32 = 180.0;
const B_X: f32 = 310.0;
const B_Y: f32 = 180.0;
const C_X: f32 = 560.0;
const C_Y: f32 = 180.0;

// ── Colors ──────────────────────────────────────────────────────────────

const BG_DARK: Color = Color {
    r: 0.15,
    g: 0.15,
    b: 0.15,
    a: 1.0,
};
const BG_CELL: Color = Color {
    r: 0.85,
    g: 0.85,
    b: 0.85,
    a: 1.0,
};
const BG_HIGHLIGHT: Color = Color {
    r: 0.4,
    g: 0.85,
    b: 0.4,
    a: 1.0,
};
const BG_RESULT: Color = Color {
    r: 0.35,
    g: 0.8,
    b: 0.35,
    a: 1.0,
};
const TEXT_NORMAL: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
const TEXT_DIM: Color = Color {
    r: 0.3,
    g: 0.3,
    b: 0.3,
    a: 1.0,
};
const TEXT_LABEL: Color = Color {
    r: 0.9,
    g: 0.9,
    b: 0.9,
    a: 1.0,
};
const TITLE_COLOR: Color = Color {
    r: 0.1,
    g: 0.6,
    b: 1.0,
    a: 1.0,
};

// ── Phase definition ────────────────────────────────────────────────────

struct Phase {
    highlight_row: Option<usize>,    // A row to highlight
    highlight_col: Option<usize>,    // B col to highlight
    compute: Option<(usize, usize)>, // C cell being computed (None = all done)
    formula: &'static str,
}

const PHASES: &[Phase] = &[
    Phase {
        highlight_row: None,
        highlight_col: None,
        compute: None,
        formula: "A × B = C",
    },
    Phase {
        highlight_row: Some(0),
        highlight_col: Some(0),
        compute: Some((0, 0)),
        formula: "C[0][0] = 1·7 + 2·9 + 3·11 = 7 + 18 + 33 = 58",
    },
    Phase {
        highlight_row: Some(0),
        highlight_col: Some(1),
        compute: Some((0, 1)),
        formula: "C[0][1] = 1·8 + 2·10 + 3·12 = 8 + 20 + 36 = 64",
    },
    Phase {
        highlight_row: Some(1),
        highlight_col: Some(0),
        compute: Some((1, 0)),
        formula: "C[1][0] = 4·7 + 5·9 + 6·11 = 28 + 45 + 66 = 139",
    },
    Phase {
        highlight_row: Some(1),
        highlight_col: Some(1),
        compute: Some((1, 1)),
        formula: "C[1][1] = 4·8 + 5·10 + 6·12 = 32 + 50 + 72 = 154",
    },
    Phase {
        highlight_row: None,
        highlight_col: None,
        compute: None,
        formula: "A × B = C  ✓  done!",
    },
];

// ── Phase scene builder ─────────────────────────────────────────────────

enum ACellState {
    Normal,
    Highlight,
}

enum BCellState {
    Normal,
    Highlight,
}

enum CState {
    Dim,
    Compute,
    Result,
}

fn cell_x(origin_x: f32, col: usize) -> f32 {
    origin_x + col as f32 * (CELL_W + GAP)
}

fn cell_y(origin_y: f32, row: usize) -> f32 {
    origin_y + row as f32 * (CELL_H + GAP)
}

fn a_state(phase: &Phase, row: usize, _col: usize) -> ACellState {
    if phase.highlight_row == Some(row) {
        ACellState::Highlight
    } else {
        ACellState::Normal
    }
}

fn b_state(phase: &Phase, _row: usize, col: usize) -> BCellState {
    if phase.highlight_col == Some(col) {
        BCellState::Highlight
    } else {
        BCellState::Normal
    }
}

fn c_state(phase_idx: usize, row: usize, col: usize) -> CState {
    let phase = &PHASES[phase_idx];
    match phase.compute {
        Some((cr, cc)) if cr == row && cc == col => CState::Compute,
        _ if (row * DIMS.b_cols + col) < computed_count(phase_idx) => CState::Result,
        _ => CState::Dim,
    }
}

fn computed_count(phase_idx: usize) -> usize {
    let phase = &PHASES[phase_idx];
    match phase.compute {
        Some((r, c)) => r * DIMS.b_cols + c,
        None if phase_idx == 0 => 0,
        None => DIMS.a_rows * DIMS.b_cols,
    }
}

/// Cell background fill
fn a_bg(state: &ACellState) -> impl IntoAnimated<Color> {
    match state {
        ACellState::Normal => BG_CELL,
        ACellState::Highlight => BG_HIGHLIGHT,
    }
}

fn b_bg(state: &BCellState) -> impl IntoAnimated<Color> {
    match state {
        BCellState::Normal => BG_CELL,
        BCellState::Highlight => BG_HIGHLIGHT,
    }
}

fn c_bg(state: &CState) -> impl IntoAnimated<Color> {
    match state {
        CState::Dim => BG_DARK.into_animated(),
        CState::Compute => tween(BG_DARK, BG_RESULT),
        CState::Result => BG_RESULT.into_animated(),
    }
}

fn c_text_fill(state: &CState) -> impl IntoAnimated<Color> {
    match state {
        CState::Dim => TEXT_DIM.into_animated(),
        CState::Compute => tween(TEXT_DIM, TEXT_NORMAL),
        CState::Result => TEXT_NORMAL.into_animated(),
    }
}

/// Roughly center text in a cell for monospace font.
fn text_x(cell_x: f32, value: i32) -> f32 {
    let digits = if value == 0 {
        1
    } else {
        (value.abs() as f32).log10().floor() as usize + 1
    };
    cell_x + (CELL_W - digits as f32 * 7.5) / 2.0
}

fn text_y(cell_y: f32) -> f32 {
    cell_y + (CELL_H - 14.0) / 2.0
}

#[allow(clippy::needless_range_loop)]
fn build_phase_scene(phase_idx: usize) -> Scene {
    let phase = &PHASES[phase_idx];
    let mut sc = scene();

    // Title
    sc = sc.node(
        Text::new()
            .text("A")
            .x(A_X + 30.0)
            .y(A_Y - 40.0)
            .font_size(20.0)
            .fill(TITLE_COLOR),
    );
    sc = sc.node(
        Text::new()
            .text("B")
            .x(B_X + 30.0)
            .y(B_Y - 40.0)
            .font_size(20.0)
            .fill(TITLE_COLOR),
    );
    sc = sc.node(
        Text::new()
            .text("C")
            .x(C_X + 30.0)
            .y(C_Y - 40.0)
            .font_size(20.0)
            .fill(TITLE_COLOR),
    );

    // Operators
    sc = sc.node(
        Text::new()
            .text("×")
            .x(270.0)
            .y(195.0)
            .font_size(28.0)
            .fill(TEXT_LABEL),
    );
    sc = sc.node(
        Text::new()
            .text("=")
            .x(530.0)
            .y(195.0)
            .font_size(28.0)
            .fill(TEXT_LABEL),
    );

    // Dimensions
    sc = sc.node(
        Text::new()
            .text("2×3")
            .x(A_X + 30.0)
            .y(A_Y + 2.0 * (CELL_H + GAP) + 10.0)
            .font_size(12.0)
            .fill(Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            }),
    );
    sc = sc.node(
        Text::new()
            .text("3×2")
            .x(B_X + 30.0)
            .y(B_Y + 3.0 * (CELL_H + GAP) + 10.0)
            .font_size(12.0)
            .fill(Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            }),
    );
    sc = sc.node(
        Text::new()
            .text("2×2")
            .x(C_X + 30.0)
            .y(C_Y + 2.0 * (CELL_H + GAP) + 10.0)
            .font_size(12.0)
            .fill(Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            }),
    );

    // Matrix A cells (2 rows × 3 cols)
    for row in 0..DIMS.a_rows {
        for col in 0..DIMS.a_cols {
            let cx = cell_x(A_X, col);
            let cy = cell_y(A_Y, row);
            let state = a_state(phase, row, col);
            let val = A_DATA[row][col];

            sc = sc
                .node(
                    rect()
                        .x(cx)
                        .y(cy)
                        .width(CELL_W)
                        .height(CELL_H)
                        .fill(a_bg(&state)),
                )
                .node(
                    Text::new()
                        .text(val.to_string())
                        .x(text_x(cx, val))
                        .y(text_y(cy))
                        .font_size(14.0)
                        .fill(TEXT_NORMAL),
                );
        }
    }

    // Matrix B cells (3 rows × 2 cols)
    for row in 0..DIMS.a_cols {
        for col in 0..DIMS.b_cols {
            let cx = cell_x(B_X, col);
            let cy = cell_y(B_Y, row);
            let state = b_state(phase, row, col);
            let val = B_DATA[row][col];

            sc = sc
                .node(
                    rect()
                        .x(cx)
                        .y(cy)
                        .width(CELL_W)
                        .height(CELL_H)
                        .fill(b_bg(&state)),
                )
                .node(
                    Text::new()
                        .text(val.to_string())
                        .x(text_x(cx, val))
                        .y(text_y(cy))
                        .font_size(14.0)
                        .fill(TEXT_NORMAL),
                );
        }
    }

    // Matrix C cells (2 rows × 2 cols)
    for row in 0..DIMS.a_rows {
        for col in 0..DIMS.b_cols {
            let cx = cell_x(C_X, col);
            let cy = cell_y(C_Y, row);
            let state = c_state(phase_idx, row, col);
            let val = C_DATA[row][col];

            sc = sc
                .node(
                    rect()
                        .x(cx)
                        .y(cy)
                        .width(CELL_W)
                        .height(CELL_H)
                        .fill(c_bg(&state)),
                )
                .node(
                    Text::new()
                        .text(val.to_string())
                        .x(text_x(cx, val))
                        .y(text_y(cy))
                        .font_size(14.0)
                        .fill(c_text_fill(&state)),
                );
        }
    }

    // Formula text at bottom
    sc = sc.node(
        Text::new()
            .text(phase.formula)
            .x(40.0)
            .y(350.0)
            .font_size(16.0)
            .fill(TEXT_LABEL),
    );

    sc
}

// ── Main ────────────────────────────────────────────────────────────────

fn main() {
    // Phase durations: intro 1s, each compute 2s, done 1s
    let durations: &[f32] = &[1.0, 2.0, 2.0, 2.0, 2.0, 1.0];
    let names: &[&str] = &["intro", "c00", "c01", "c10", "c11", "done"];

    let anims: Vec<_> = (0..PHASES.len())
        .map(|i| {
            let sc = build_phase_scene(i);
            animation(names[i], durations[i], sc)
        })
        .collect();

    let timeline = sequence("matrix-mult", anims);

    let viewport = Viewport::new(800.0, 450.0);
    let config = ExportConfig::new(30.0, viewport);

    std::fs::create_dir_all("results").ok();
    println!("Exporting results/matrix-mult.mp4 …");
    match export_mp4(&timeline, &config, "results/matrix-mult.mp4") {
        Ok(()) => println!("Written results/matrix-mult.mp4"),
        Err(e) => eprintln!("export skipped: {e}"),
    }
}
