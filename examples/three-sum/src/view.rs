use crate::{
    style::*, ThreeSumAction, ThreeSumMotion, ThreeSumStep, ThreeSumTiming, ThreeSumTrace, N,
};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::Viewport;

const VIEW_W: f32 = 1000.0;
const VIEW_H: f32 = 620.0;
const TILE_W: f32 = 58.0;
const TILE_H: f32 = 42.0;
const TILE_GAP: f32 = 18.0;
const TILE_FONT: f32 = 17.0;
const TILE_START_X: f32 = (VIEW_W - (N as f32 * TILE_W + (N - 1) as f32 * TILE_GAP)) / 2.0;
const TILE_RADIUS: f32 = 7.0;

const INPUT_Y: f32 = 166.0;
const ARRAY_Y: f32 = 276.0;
const POINTER_Y: f32 = ARRAY_Y + TILE_H + 28.0;
const RESULT_Y: f32 = 466.0;
const RESULT_START_X: f32 = 278.0;
const RESULT_GAP: f32 = 42.0;

#[derive(Clone, Copy)]
pub struct ThreeSumView;

pub fn three_sum_view() -> ThreeSumView {
    ThreeSumView
}

pub(crate) fn build_three_sum(
    name: &'static str,
    trace: ThreeSumTrace,
    motion: ThreeSumMotion,
    timing: ThreeSumTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();

    for (index, step) in trace.steps.iter().enumerate() {
        let duration = step_duration(step, timing);
        let scene = match step.action {
            ThreeSumAction::IntroduceInput => intro_scene(&trace),
            ThreeSumAction::Sort => sort_scene(&trace, motion),
            ThreeSumAction::Done => final_scene(&trace),
            _ => step_scene(&trace, step, motion),
        };
        anims.push(animation(
            format!("three-sum-step-{index:02}"),
            duration,
            scene,
        ));
    }

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn step_duration(step: &ThreeSumStep, timing: ThreeSumTiming) -> f32 {
    match step.action {
        ThreeSumAction::IntroduceInput => timing.intro,
        ThreeSumAction::Sort => timing.sort,
        ThreeSumAction::FixAnchor { .. } => timing.fix_anchor,
        ThreeSumAction::SetPointers { .. } => timing.set_pointers,
        ThreeSumAction::Compare { .. } => timing.compare,
        ThreeSumAction::MoveLeft { .. } | ThreeSumAction::MoveRight { .. } => timing.move_pointer,
        ThreeSumAction::Found { .. } => timing.found,
        ThreeSumAction::SkipDuplicate { .. } => timing.skip_duplicate,
        ThreeSumAction::Done => timing.final_hold,
    }
}

fn tile_x(index: usize) -> f32 {
    TILE_START_X + index as f32 * (TILE_W + TILE_GAP)
}

fn result_x(result_index: usize, item_index: usize) -> f32 {
    RESULT_START_X
        + result_index as f32 * (TILE_W * 3.0 + TILE_GAP * 2.0 + RESULT_GAP)
        + item_index as f32 * (TILE_W + TILE_GAP)
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

fn box_path(x: impl IntoAnimated<f32>, y: impl IntoAnimated<f32>) -> Animated<Path> {
    let x = x.into_animated();
    let y = y.into_animated();
    Animated::new(move |t| {
        rounded_rect_path(x.resolve(t), y.resolve(t), TILE_W, TILE_H, TILE_RADIUS)
    })
}

fn style(fill: Color, stroke_width: f32, stroke_color: Color) -> Style {
    Style::new().fill(fill).stroke(stroke_width, stroke_color)
}

fn tile_style(fill: Color) -> Style {
    style(fill, 1.8, TILE_STROKE)
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

fn add_background(mut sc: Scene, subtitle: impl Into<String>) -> Scene {
    sc = sc.node(
        path_node()
            .path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
            .style(style(BG, 0.0, BG)),
    );
    sc = sc.node(label(54.0, 52.0, "3Sum", 30.0, INK));
    sc.node(label(54.0, 88.0, subtitle, 16.0, MUTED))
}

fn add_slots(mut sc: Scene, y: f32) -> Scene {
    for index in 0..N {
        sc = sc.node(
            path_node()
                .path(rounded_rect_path(
                    tile_x(index),
                    y,
                    TILE_W,
                    TILE_H,
                    TILE_RADIUS,
                ))
                .style(style(PANEL, 1.0, PANEL_BORDER)),
        );
    }
    sc
}

fn add_tile(
    mut sc: Scene,
    value: i32,
    x: impl IntoAnimated<f32>,
    y: impl IntoAnimated<f32>,
    tile_style: impl IntoAnimated<Style>,
) -> Scene {
    let x = x.into_animated();
    let y = y.into_animated();
    let text_x = x.clone();
    let text_y = y.clone();
    let content = value.to_string();
    let offset_x = (TILE_W - text_width(&content, TILE_FONT)) / 2.0;
    let offset_y = (TILE_H + TILE_FONT * 0.45) / 2.0;

    sc = sc.node(path_node().path(box_path(x, y)).style(tile_style));
    sc.node(
        text()
            .x(Animated::new(move |t| text_x.resolve(t) + offset_x))
            .y(Animated::new(move |t| text_y.resolve(t) + offset_y))
            .text(content)
            .font_size(TILE_FONT)
            .fill(INK),
    )
}

fn add_array(mut sc: Scene, sorted: &[i32; N], step: &ThreeSumStep) -> Scene {
    let (anchor, left, right) = active_indices(step);
    for (index, value) in sorted.iter().enumerate() {
        let fill = if Some(index) == anchor {
            ANCHOR_FILL
        } else if Some(index) == left {
            LEFT_FILL
        } else if Some(index) == right {
            RIGHT_FILL
        } else {
            PANEL
        };
        sc = add_tile(sc, *value, tile_x(index), ARRAY_Y, tile_style(fill));
    }
    sc
}

fn active_indices(step: &ThreeSumStep) -> (Option<usize>, Option<usize>, Option<usize>) {
    match step.action {
        ThreeSumAction::FixAnchor { i } => (Some(i), None, None),
        ThreeSumAction::SetPointers { i, left, right }
        | ThreeSumAction::Compare { i, left, right, .. }
        | ThreeSumAction::Found { i, left, right, .. } => (Some(i), Some(left), Some(right)),
        ThreeSumAction::MoveLeft { i, from, .. } => (Some(i), Some(from), None),
        ThreeSumAction::MoveRight { i, from, .. } => (Some(i), None, Some(from)),
        ThreeSumAction::SkipDuplicate { from, .. } => (Some(from), None, None),
        ThreeSumAction::IntroduceInput | ThreeSumAction::Sort | ThreeSumAction::Done => {
            (None, None, None)
        }
    }
}

fn add_pointers(mut sc: Scene, step: &ThreeSumStep, motion: ThreeSumMotion) -> Scene {
    match step.action {
        ThreeSumAction::FixAnchor { i } => {
            sc = add_pointer(sc, "i", tile_x(i) + TILE_W / 2.0, ANCHOR_FILL);
        }
        ThreeSumAction::SetPointers { i, left, right }
        | ThreeSumAction::Compare { i, left, right, .. }
        | ThreeSumAction::Found { i, left, right, .. } => {
            sc = add_pointer(sc, "i", tile_x(i) + TILE_W / 2.0, ANCHOR_FILL);
            sc = add_pointer(sc, "L", tile_x(left) + TILE_W / 2.0, LEFT_FILL);
            sc = add_pointer(sc, "R", tile_x(right) + TILE_W / 2.0, RIGHT_FILL);
        }
        ThreeSumAction::MoveLeft { i, from, to, .. } => {
            sc = add_pointer(sc, "i", tile_x(i) + TILE_W / 2.0, ANCHOR_FILL);
            sc = add_pointer(
                sc,
                "L",
                motion.move_value(tile_x(from) + TILE_W / 2.0, tile_x(to) + TILE_W / 2.0),
                LEFT_FILL,
            );
        }
        ThreeSumAction::MoveRight { i, from, to, .. } => {
            sc = add_pointer(sc, "i", tile_x(i) + TILE_W / 2.0, ANCHOR_FILL);
            sc = add_pointer(
                sc,
                "R",
                motion.move_value(tile_x(from) + TILE_W / 2.0, tile_x(to) + TILE_W / 2.0),
                RIGHT_FILL,
            );
        }
        ThreeSumAction::SkipDuplicate { from, to, .. } => {
            sc = add_pointer(
                sc,
                "i",
                motion.move_value(tile_x(from) + TILE_W / 2.0, tile_x(to) + TILE_W / 2.0),
                ANCHOR_FILL,
            );
        }
        ThreeSumAction::IntroduceInput | ThreeSumAction::Sort | ThreeSumAction::Done => {}
    }
    sc
}

fn add_pointer(
    sc: Scene,
    text_value: &'static str,
    x: impl IntoAnimated<f32>,
    fill: Color,
) -> Scene {
    let x = x.into_animated();
    let label_x = x.clone();
    let mut sc = sc.node(circle().x(x).y(POINTER_Y).radius(15.0).fill(fill));
    sc = sc.node(
        text()
            .x(Animated::new(move |t| {
                label_x.resolve(t) - text_width(text_value, 13.0) / 2.0
            }))
            .y(POINTER_Y + 5.0)
            .text(text_value)
            .font_size(13.0)
            .fill(INK),
    );
    sc
}

fn add_compare_bracket(mut sc: Scene, step: &ThreeSumStep) -> Scene {
    if let ThreeSumAction::Compare { left, right, .. } = step.action {
        let lx = tile_x(left) + TILE_W / 2.0;
        let rx = tile_x(right) + TILE_W / 2.0;
        let top = ARRAY_Y - 24.0;
        let bottom = ARRAY_Y - 44.0;
        sc = sc.node(
            connection(Vec2::new(lx, top), Vec2::new(rx, top))
                .via([Vec2::new(lx, bottom), Vec2::new(rx, bottom)])
                .stroke(1.8, ACCENT)
                .arrow(4.0),
        );
        sc = sc.node(centered_label(
            (lx + rx) / 2.0,
            bottom - 8.0,
            "compare",
            12.0,
            ACCENT,
        ));
    }
    sc
}

fn add_results(mut sc: Scene, results: &[[i32; 3]]) -> Scene {
    sc = sc.node(label(94.0, RESULT_Y + 27.0, "Triplets", 15.0, MUTED));
    for (result_index, triplet) in results.iter().enumerate() {
        for (item_index, value) in triplet.iter().enumerate() {
            sc = add_tile(
                sc,
                *value,
                result_x(result_index, item_index),
                RESULT_Y,
                tile_style(FOUND_FILL),
            );
        }
    }
    sc
}

fn add_found_motion(
    mut sc: Scene,
    sorted: &[i32; N],
    motion: ThreeSumMotion,
    i: usize,
    left: usize,
    right: usize,
    result_index: usize,
) -> Scene {
    for (item_index, source_index) in [i, left, right].into_iter().enumerate() {
        sc = add_tile(
            sc,
            sorted[source_index],
            motion.move_value(tile_x(source_index), result_x(result_index, item_index)),
            motion.move_value(ARRAY_Y, RESULT_Y),
            tile_style(MOVING_FILL),
        );
    }
    sc
}

fn sort_scene(trace: &ThreeSumTrace, motion: ThreeSumMotion) -> Scene {
    let mut sc = add_background(scene(), "Sort first, then use two pointers");
    sc = sc.node(label(94.0, INPUT_Y + 27.0, "Original input", 15.0, MUTED));
    sc = sc.node(label(
        94.0,
        ARRAY_Y + 27.0,
        "Sorted copy for two pointers",
        15.0,
        MUTED,
    ));
    sc = sc.node(centered_label(
        VIEW_W / 2.0,
        236.0,
        "sort ascending",
        15.0,
        ACCENT,
    ));
    sc = sc.node(
        connection(
            Vec2::new(VIEW_W / 2.0, INPUT_Y + TILE_H + 18.0),
            Vec2::new(VIEW_W / 2.0, ARRAY_Y - 18.0),
        )
        .stroke(1.6, ACCENT)
        .arrow(5.0),
    );
    sc = add_slots(sc, INPUT_Y);
    sc = add_slots(sc, ARRAY_Y);
    for sorted_index in 0..N {
        let original_index = trace.sort_order[sorted_index];
        sc = add_tile(
            sc,
            trace.input[original_index],
            motion.move_value(tile_x(original_index), tile_x(sorted_index)),
            motion.move_value(INPUT_Y, ARRAY_Y),
            tile_style(MOVING_FILL),
        );
    }
    sc
}

fn intro_scene(trace: &ThreeSumTrace) -> Scene {
    let mut sc = add_background(scene(), "We need all unique triplets whose sum is zero");
    sc = sc.node(label(94.0, INPUT_Y + 27.0, "Original input", 15.0, MUTED));
    sc = add_slots(sc, INPUT_Y);
    for (index, value) in trace.input.iter().enumerate() {
        sc = add_tile(sc, *value, tile_x(index), INPUT_Y, tile_style(PANEL));
    }
    sc = sc.node(centered_label(
        VIEW_W / 2.0,
        ARRAY_Y + 18.0,
        "Next: sort the array so left and right pointers can move with meaning",
        16.0,
        MUTED,
    ));
    sc
}

fn step_scene(trace: &ThreeSumTrace, step: &ThreeSumStep, motion: ThreeSumMotion) -> Scene {
    let mut sc = add_background(scene(), step.title(&trace.sorted));
    sc = sc.node(label(94.0, ARRAY_Y + 27.0, "Sorted nums", 15.0, MUTED));
    sc = add_slots(sc, ARRAY_Y);
    sc = add_array(sc, &trace.sorted, step);
    sc = add_pointers(sc, step, motion);
    sc = add_compare_bracket(sc, step);
    sc = add_results(sc, &step.results_before);

    if let ThreeSumAction::Found {
        i,
        left,
        right,
        result_index,
        ..
    } = step.action
    {
        sc = add_found_motion(sc, &trace.sorted, motion, i, left, right, result_index);
    }

    sc
}

fn final_scene(trace: &ThreeSumTrace) -> Scene {
    let mut sc = add_background(scene(), "Unique triplets that sum to zero");
    sc = sc.node(label(94.0, ARRAY_Y + 27.0, "Sorted nums", 15.0, MUTED));
    sc = add_slots(sc, ARRAY_Y);
    for (index, value) in trace.sorted.iter().enumerate() {
        sc = add_tile(sc, *value, tile_x(index), ARRAY_Y, tile_style(PANEL));
    }
    add_results(sc, &trace.results)
}
