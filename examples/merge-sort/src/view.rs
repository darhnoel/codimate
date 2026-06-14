use crate::{style::*, MergeSort, MergeSortMotion, MergeSortTiming, MergeStep, MergeTrace, N};
use codimate::*;
use codimate::{animation, sequence, Animation, Playable};
use codimate::{box_at, Viewport};

const VIEW_W: f32 = 1000.0;
const VIEW_H: f32 = 640.0;
const TILE_W: f32 = 58.0;
const TILE_H: f32 = 42.0;
const TILE_GAP: f32 = 18.0;
const TILE_FONT: f32 = 17.0;
const TILE_START_X: f32 = (VIEW_W - (N as f32 * TILE_W + (N - 1) as f32 * TILE_GAP)) / 2.0;
const TILE_RADIUS: f32 = 7.0;

const HEADER_Y: f32 = 42.0;
const SOURCE_Y: f32 = 176.0;
const FINAL_Y: f32 = 250.0;
const OUTPUT_Y: f32 = 404.0;
const ROW_LABEL_X: f32 = 72.0;

#[derive(Clone, Copy)]
pub struct MergeSortView;

pub fn merge_sort_view() -> MergeSortView {
    MergeSortView
}

pub(crate) fn build_merge_sort(
    name: &'static str,
    state: MergeSort,
    trace: MergeTrace,
    motion: MergeSortMotion,
    timing: MergeSortTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();

    anims.push(animation(
        "overview",
        timing.overview,
        overview_scene(&state.values()),
    ));
    for (i, step) in trace.steps.iter().enumerate() {
        anims.push(animation(
            format!("merge-step-{i:02}"),
            timing.step,
            step_scene(step, motion),
        ));
        let next_pass = trace.steps.get(i + 1).map(|next| next.pass);
        if next_pass.is_some_and(|next| next != step.pass) {
            anims.push(animation(
                format!("pass-{}-copy-up", step.pass + 1),
                timing.transition,
                pass_transition_scene(step.pass, &trace.pass_results[step.pass], motion),
            ));
        }
    }
    anims.push(animation(
        "final-output-to-sorted-array",
        timing.transition,
        final_transition_scene(&trace.sorted, motion),
    ));
    anims.push(animation(
        "sorted",
        timing.final_hold,
        final_scene(&trace.sorted),
    ));

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn tile_x(index: usize) -> f32 {
    TILE_START_X + index as f32 * (TILE_W + TILE_GAP)
}

fn text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.30
}

fn style(fill: Color, stroke_width: f32, stroke_color: Color) -> Style {
    Style::new().fill(fill).stroke(stroke_width, stroke_color)
}

fn translucent(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

fn background_style() -> Style {
    style(BG, 0.0, BG)
}

fn slot_style() -> Style {
    style(SLOT_FILL, 1.0, PANEL_BORDER)
}

fn tile_style(fill: Color) -> Style {
    style(fill, 1.8, TILE_STROKE)
}

fn ghost_tile_style(fill: Color) -> Style {
    style(translucent(fill, 0.14), 1.2, translucent(TILE_STROKE, 0.35))
}

fn band_style(fill: Color) -> Style {
    style(translucent(fill, 0.22), 1.2, translucent(fill, 0.60))
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

fn center_from_top_left(
    x: impl IntoAnimated<f32>,
    y: impl IntoAnimated<f32>,
    size: Vec2,
) -> Animated<Vec2> {
    let x = x.into_animated();
    let y = y.into_animated();
    Animated::new(move |t| Vec2::new(x.resolve(t) + size.x / 2.0, y.resolve(t) + size.y / 2.0))
}

fn add_panel(sc: Scene) -> Scene {
    sc.add(primitive_path(rect_path(0.0, 0.0, VIEW_W, VIEW_H)).style(background_style()))
}

fn add_header(mut sc: Scene, title: impl Into<String>) -> Scene {
    sc = sc.add(label(54.0, HEADER_Y, "Merge Sort", 28.0, INK));
    sc.add(label(54.0, 76.0, title, 16.0, MUTED))
}

fn add_lane_labels(mut sc: Scene) -> Scene {
    sc = sc.add(label(
        ROW_LABEL_X,
        SOURCE_Y + 25.0,
        "Source runs",
        15.0,
        MUTED,
    ));
    sc = sc.add(label(
        ROW_LABEL_X,
        OUTPUT_Y + 25.0,
        "Output buffer",
        15.0,
        MUTED,
    ));
    sc
}

fn add_slots(mut sc: Scene, y: f32) -> Scene {
    for i in 0..N {
        sc = sc.add(
            box_at(
                Vec2::new(tile_x(i) + TILE_W / 2.0, y + TILE_H / 2.0),
                Vec2::new(TILE_W, TILE_H),
            )
            .radius(TILE_RADIUS)
            .style(slot_style()),
        );
    }
    sc
}

fn add_band(mut sc: Scene, start: usize, end: usize, y: f32, fill: Color) -> Scene {
    if start >= end {
        return sc;
    }
    let pad_x = 5.0;
    let pad_y = 7.0;
    let x = tile_x(start) - pad_x;
    let w = tile_x(end - 1) + TILE_W - x + pad_x;
    let h = TILE_H + pad_y * 2.0;
    sc = sc.add(
        box_at(Vec2::new(x + w / 2.0, y - pad_y + h / 2.0), Vec2::new(w, h))
            .radius(TILE_RADIUS + 1.0)
            .style(band_style(fill)),
    );
    sc
}

fn add_tile(
    sc: Scene,
    value: i32,
    x: impl IntoAnimated<f32>,
    y: impl IntoAnimated<f32>,
    tile_style: impl IntoAnimated<Style>,
) -> Scene {
    add_tile_with_text(sc, value, x, y, tile_style, INK)
}

fn add_tile_with_text(
    mut sc: Scene,
    value: i32,
    x: impl IntoAnimated<f32>,
    y: impl IntoAnimated<f32>,
    tile_style: impl IntoAnimated<Style>,
    text_fill: Color,
) -> Scene {
    let x = x.into_animated();
    let y = y.into_animated();
    let text_x = x.clone();
    let text_y = y.clone();
    let content = value.to_string();
    let offset_x = (TILE_W - text_width(&content, TILE_FONT)) / 2.0;
    let offset_y = (TILE_H + TILE_FONT * 0.45) / 2.0;

    sc = sc.add(
        box_at(
            center_from_top_left(x, y, Vec2::new(TILE_W, TILE_H)),
            Vec2::new(TILE_W, TILE_H),
        )
        .radius(TILE_RADIUS)
        .style(tile_style),
    );
    sc.add(
        text()
            .x(Animated::new(move |t| text_x.resolve(t) + offset_x))
            .y(Animated::new(move |t| text_y.resolve(t) + offset_y))
            .text(content)
            .font_size(TILE_FONT)
            .fill(text_fill),
    )
}

fn source_tile_style(step: &MergeStep, index: usize) -> Style {
    if Some(index) == step.left || Some(index) == step.right {
        tile_style(CANDIDATE_FILL)
    } else if index >= step.start && index < step.mid {
        tile_style(LEFT_FILL)
    } else if index >= step.mid && index < step.end {
        tile_style(RIGHT_FILL)
    } else {
        tile_style(PANEL)
    }
}

fn add_source_row(mut sc: Scene, step: &MergeStep) -> Scene {
    for i in 0..N {
        if step.consumed_before[i] {
            continue;
        }
        if i == step.winner {
            sc = add_tile_with_text(
                sc,
                step.source[i],
                tile_x(i),
                SOURCE_Y,
                ghost_tile_style(source_tile_style(step, i).fill),
                translucent(MUTED, 0.70),
            );
            continue;
        }
        sc = add_tile(
            sc,
            step.source[i],
            tile_x(i),
            SOURCE_Y,
            source_tile_style(step, i),
        );
    }
    sc
}

fn add_output_row(mut sc: Scene, output_before: &[Option<i32>; N]) -> Scene {
    for (i, value) in output_before.iter().enumerate() {
        if let Some(value) = value {
            sc = add_tile(sc, *value, tile_x(i), OUTPUT_Y, tile_style(PLACED_FILL));
        }
    }
    sc
}

fn add_winner(mut sc: Scene, step: &MergeStep, motion: MergeSortMotion) -> Scene {
    let rest = tile_style(MOVING_FILL);
    let active = tile_style(PLACED_FILL);
    sc = sc.add(
        box_at(
            Vec2::new(tile_x(step.output) + TILE_W / 2.0, OUTPUT_Y + TILE_H / 2.0),
            Vec2::new(TILE_W + 8.0, TILE_H + 8.0),
        )
        .radius(TILE_RADIUS + 2.0)
        .style(style(Color::TRANSPARENT, 2.5, ACCENT)),
    );
    add_tile(
        sc,
        step.source[step.winner],
        motion.move_value(tile_x(step.winner), tile_x(step.output)),
        motion.move_value(SOURCE_Y, OUTPUT_Y),
        motion.move_value(rest, active),
    )
}

fn add_comparison_guide(mut sc: Scene, step: &MergeStep) -> Scene {
    if let (Some(left), Some(right)) = (step.left, step.right) {
        let lx = tile_x(left) + TILE_W / 2.0;
        let rx = tile_x(right) + TILE_W / 2.0;
        let top_y = SOURCE_Y + TILE_H + 6.0;
        let bottom_y = SOURCE_Y + TILE_H + 28.0;
        sc = sc.add(
            connection(Vec2::new(lx, top_y), Vec2::new(rx, top_y))
                .via([Vec2::new(lx, bottom_y), Vec2::new(rx, bottom_y)])
                .stroke(1.8, ACCENT)
                .arrow(4.0),
        );
        sc = sc.add(centered_label(
            (lx + rx) / 2.0,
            bottom_y + 22.0,
            "compare",
            13.0,
            ACCENT,
        ));
    }
    sc
}

fn step_scene(step: &MergeStep, motion: MergeSortMotion) -> Scene {
    let mut sc = add_panel(scene());
    sc = add_header(sc, step.pass_label());
    sc = add_lane_labels(sc);

    sc = add_band(sc, step.start, step.mid, SOURCE_Y, LEFT_FILL);
    sc = add_band(sc, step.mid, step.end, SOURCE_Y, RIGHT_FILL);
    sc = add_slots(sc, SOURCE_Y);
    sc = add_slots(sc, OUTPUT_Y);
    sc = add_source_row(sc, step);
    sc = add_output_row(sc, &step.output_before);
    sc = add_comparison_guide(sc, step);
    add_winner(sc, step, motion)
}

fn overview_scene(values: &[i32; N]) -> Scene {
    let mut sc = add_panel(scene());
    sc = add_header(sc, "Bottom-up merge sort");
    sc = add_lane_labels(sc);
    sc = add_slots(sc, SOURCE_Y);
    sc = add_slots(sc, OUTPUT_Y);
    for (i, value) in values.iter().enumerate() {
        sc = add_tile(sc, *value, tile_x(i), SOURCE_Y, tile_style(PANEL));
    }
    sc
}

fn pass_transition_scene(pass: usize, values: &[i32; N], motion: MergeSortMotion) -> Scene {
    let mut sc = add_panel(scene());
    sc = add_header(sc, format!("Pass {} complete", pass + 1));
    sc = sc.add(label(
        ROW_LABEL_X,
        SOURCE_Y + 25.0,
        "Next source runs",
        15.0,
        MUTED,
    ));
    sc = sc.add(label(
        ROW_LABEL_X,
        OUTPUT_Y + 25.0,
        "Completed output",
        15.0,
        MUTED,
    ));
    sc = add_slots(sc, SOURCE_Y);
    sc = add_slots(sc, OUTPUT_Y);

    let rest = tile_style(PLACED_FILL);
    let active = tile_style(PANEL);
    for (i, value) in values.iter().enumerate() {
        sc = add_tile(
            sc,
            *value,
            tile_x(i),
            motion.move_value(OUTPUT_Y, SOURCE_Y),
            motion.move_value(rest, active),
        );
    }

    sc
}

fn final_transition_scene(sorted: &[i32; N], motion: MergeSortMotion) -> Scene {
    let mut sc = add_panel(scene());
    sc = add_header(sc, "Final pass complete");
    sc = sc.add(label(
        ROW_LABEL_X,
        FINAL_Y + TILE_H / 2.0 + 5.0,
        "Sorted array",
        15.0,
        MUTED,
    ));
    sc = sc.add(label(
        ROW_LABEL_X,
        OUTPUT_Y + 25.0,
        "Final output",
        15.0,
        MUTED,
    ));
    sc = add_slots(sc, FINAL_Y);
    sc = add_slots(sc, OUTPUT_Y);

    let rest = tile_style(PLACED_FILL);
    let active = tile_style(PLACED_FILL);
    for (i, value) in sorted.iter().enumerate() {
        sc = add_tile(
            sc,
            *value,
            tile_x(i),
            motion.move_value(OUTPUT_Y, FINAL_Y),
            motion.move_value(rest, active),
        );
    }

    sc
}

fn final_scene(sorted: &[i32; N]) -> Scene {
    let mut sc = add_panel(scene());
    sc = add_header(sc, "Sorted output");
    sc = sc.add(label(
        ROW_LABEL_X,
        FINAL_Y + TILE_H / 2.0 + 5.0,
        "Sorted array",
        15.0,
        MUTED,
    ));
    sc = add_slots(sc, FINAL_Y);
    for (i, value) in sorted.iter().enumerate() {
        sc = add_tile(sc, *value, tile_x(i), FINAL_Y, tile_style(PLACED_FILL));
    }
    sc
}
