use crate::{
    style::*, Knapsack as KnapsackState, KnapsackAction, KnapsackMotion, KnapsackStep,
    KnapsackTiming, KnapsackTrace, CAPACITY, COLS, ITEM_COUNT, ROWS,
};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::Viewport;

const VIEW_W: f32 = 960.0;
const VIEW_H: f32 = 600.0;

const CELL_W: f32 = 58.0;
const CELL_H: f32 = 50.0;
const GAP: f32 = 6.0;
const CELL_RADIUS: f32 = 8.0;
const FONT: f32 = 18.0;

const GRID_X0: f32 = 360.0;
const GRID_Y0: f32 = 178.0;

#[derive(Clone, Copy)]
pub struct KnapsackView;

pub fn knapsack_view() -> KnapsackView {
    KnapsackView
}

pub(crate) fn build_knapsack(
    name: &'static str,
    _state: KnapsackState,
    trace: KnapsackTrace,
    motion: KnapsackMotion,
    timing: KnapsackTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();
    for step in &trace.steps {
        anims.push(animation(
            format!("knapsack-step-{:02}", step.index),
            step_duration(step, timing),
            step_scene(step, &trace, motion),
        ));
    }

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn step_duration(step: &KnapsackStep, timing: KnapsackTiming) -> f32 {
    match step.action {
        KnapsackAction::Init => timing.init,
        KnapsackAction::Fill { .. } => timing.fill,
        KnapsackAction::Done => timing.done,
    }
}

// ----- geometry -----

fn cell_x(c: usize) -> f32 {
    GRID_X0 + c as f32 * (CELL_W + GAP)
}

fn cell_y(i: usize) -> f32 {
    GRID_Y0 + i as f32 * (CELL_H + GAP)
}

// ----- small text/style helpers (mirrors the other examples) -----

fn text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.30
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

fn style(fill: Color, stroke_width: f32, stroke_color: Color) -> Style {
    Style::new().fill(fill).stroke(stroke_width, stroke_color)
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

fn cell_style(fill: impl IntoAnimated<Color>, active: bool) -> Animated<Style> {
    let fill = fill.into_animated();
    Animated::new(move |t| {
        let stroke = if active { BORDER } else { SUBTLE_BORDER };
        let width = if active { 2.4 } else { 1.0 };
        style(fill.resolve(t), width, stroke)
    })
}

// ----- cells -----

#[derive(Clone, Copy)]
enum CellRole {
    Empty,
    Filled,
    Current,
    SkipCand,
    TakeCand,
    Path,
}

fn cell_role(i: usize, c: usize, trace: &KnapsackTrace, current: &KnapsackStep) -> CellRole {
    match current.action {
        KnapsackAction::Fill { item, cap } => {
            if i == item && c == cap {
                CellRole::Current
            } else if i + 1 == item && c == cap {
                CellRole::SkipCand
            } else if current.take.is_some()
                && i + 1 == item
                && c + current_weight(item, trace) == cap
            {
                CellRole::TakeCand
            } else if current.filled[i][c] {
                CellRole::Filled
            } else {
                CellRole::Empty
            }
        }
        KnapsackAction::Done => {
            if current.path[i][c] {
                CellRole::Path
            } else if current.filled[i][c] {
                CellRole::Filled
            } else {
                CellRole::Empty
            }
        }
        KnapsackAction::Init => {
            if current.filled[i][c] {
                CellRole::Filled
            } else {
                CellRole::Empty
            }
        }
    }
}

fn current_weight(item: usize, trace: &KnapsackTrace) -> usize {
    trace.weights[item - 1]
}

fn add_cell(
    sc: Scene,
    i: usize,
    c: usize,
    step: &KnapsackStep,
    trace: &KnapsackTrace,
    motion: KnapsackMotion,
) -> Scene {
    let x = cell_x(c);
    let y = cell_y(i);
    let role = cell_role(i, c, trace, step);

    let (fill, active): (Animated<Color>, bool) = match role {
        CellRole::Empty => (CELL_EMPTY.into_animated(), false),
        CellRole::Filled => (CELL_FILLED.into_animated(), false),
        CellRole::Current => (motion.ease(CELL_FILLED, CELL_CURRENT), true),
        CellRole::SkipCand => (CAND_SKIP.into_animated(), true),
        CellRole::TakeCand => (CAND_TAKE.into_animated(), true),
        CellRole::Path => (CELL_PATH.into_animated(), true),
    };

    let mut sc = sc.add(
        primitive_path(rounded_rect_path(x, y, CELL_W, CELL_H, CELL_RADIUS))
            .style(cell_style(fill, active)),
    );

    if step.filled[i][c] {
        sc = sc.add(centered_label(
            x + CELL_W / 2.0,
            y + CELL_H / 2.0 + FONT * 0.33,
            step.dp[i][c].to_string(),
            FONT,
            INK,
        ));
    }
    sc
}

// ----- panels -----

fn add_background(mut sc: Scene, step: &KnapsackStep, trace: &KnapsackTrace) -> Scene {
    sc = sc.add(
        primitive_path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
            .style(style(BG, 0.0, BG)),
    );
    sc = sc.add(label(54.0, 52.0, "0/1 Knapsack (DP)", 30.0, INK));
    sc = sc.add(label(54.0, 88.0, subtitle(step, trace), 16.0, MUTED));
    sc.add(label(54.0, VIEW_H - 26.0, answer_line(step), 16.0, MUTED))
}

fn item_name(k: usize) -> String {
    // Items shown to the viewer as A, B, C, ... (k is 1-based).
    char::from(b'A' + (k as u8 - 1)).to_string()
}

fn add_items_panel(mut sc: Scene, step: &KnapsackStep, trace: &KnapsackTrace) -> Scene {
    sc = sc.add(label(54.0, 150.0, "Items (weight, value)", 17.0, MUTED));

    let active_item = match step.action {
        KnapsackAction::Fill { item, .. } => Some(item),
        _ => None,
    };

    for k in 1..=ITEM_COUNT {
        let y = 186.0 + (k as f32 - 1.0) * 40.0;
        let color = if active_item == Some(k) {
            ITEM_ACTIVE
        } else if step.chosen[k - 1] {
            ITEM_CHOSEN
        } else {
            INK
        };
        let mark = if step.chosen[k - 1] { " ✓" } else { "" };
        sc = sc.add(label(
            54.0,
            y,
            format!(
                "{}:  w={}  v={}{}",
                item_name(k),
                trace.weights[k - 1],
                trace.values[k - 1],
                mark
            ),
            18.0,
            color,
        ));
    }

    sc.add(label(
        54.0,
        186.0 + ITEM_COUNT as f32 * 40.0 + 6.0,
        format!("Capacity W = {}", CAPACITY),
        18.0,
        MUTED,
    ))
}

fn add_headers(mut sc: Scene) -> Scene {
    sc = sc.add(centered_label(
        GRID_X0 + (COLS as f32 * (CELL_W + GAP)) / 2.0,
        GRID_Y0 - 40.0,
        "capacity  c  \u{2192}",
        15.0,
        MUTED,
    ));
    for c in 0..COLS {
        sc = sc.add(centered_label(
            cell_x(c) + CELL_W / 2.0,
            GRID_Y0 - 14.0,
            c.to_string(),
            15.0,
            MUTED,
        ));
    }
    for i in 0..ROWS {
        let row_label = if i == 0 {
            "\u{2205}".to_string() // empty prefix
        } else {
            item_name(i)
        };
        sc = sc.add(centered_label(
            GRID_X0 - 26.0,
            cell_y(i) + CELL_H / 2.0 + FONT * 0.33,
            row_label,
            16.0,
            MUTED,
        ));
    }
    sc
}

fn step_scene(step: &KnapsackStep, trace: &KnapsackTrace, motion: KnapsackMotion) -> Scene {
    let mut sc = add_background(scene(), step, trace);
    sc = add_items_panel(sc, step, trace);
    sc = add_headers(sc);

    for i in 0..ROWS {
        for c in 0..COLS {
            sc = add_cell(sc, i, c, step, trace, motion);
        }
    }
    sc
}

// ----- captions -----

fn subtitle(step: &KnapsackStep, trace: &KnapsackTrace) -> String {
    match step.action {
        KnapsackAction::Init => {
            "dp[i][c] = best value using the first i items within capacity c".to_string()
        }
        KnapsackAction::Fill { item, cap } => {
            let result = step.dp[item][cap];
            match step.take {
                Some(t) => {
                    let choice = if step.took { "take" } else { "skip" };
                    format!(
                        "Item {}: max(skip={}, take={}) = {}  \u{2192} {}",
                        item_name(item),
                        step.skip,
                        t,
                        result,
                        choice
                    )
                }
                None => format!(
                    "Item {} (w={}) doesn't fit in capacity {} \u{2192} skip, dp = {}",
                    item_name(item),
                    trace.weights[item - 1],
                    cap,
                    result
                ),
            }
        }
        KnapsackAction::Done => {
            let chosen: Vec<String> = (1..=ITEM_COUNT)
                .filter(|&k| step.chosen[k - 1])
                .map(item_name)
                .collect();
            format!(
                "Best value {} — take items {} (green = backtrack path)",
                step.dp[ITEM_COUNT][CAPACITY],
                chosen.join(" + ")
            )
        }
    }
}

fn answer_line(step: &KnapsackStep) -> String {
    let cell = if step.filled[ITEM_COUNT][CAPACITY] {
        step.dp[ITEM_COUNT][CAPACITY].to_string()
    } else {
        "\u{2026}".to_string()
    };
    format!("answer  dp[{}][{}] = {}", ITEM_COUNT, CAPACITY, cell)
}
